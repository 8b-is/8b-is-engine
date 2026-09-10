//! gemm — the integer ternary GEMM, bit-exact across architectures.
//!
//! The lane's arithmetic contract, in one sentence: **the forward pass
//! accumulates in `i32` integer arithmetic, and the single float scale is
//! applied once, at the end.** Integer addition is associative — it does
//! not care which order you add in — so the scalar core, the AVX2 lane
//! (AMD / Intel x86-64), the NEON lane (ARM, Apple and server alike), the
//! WASM simd128 lane, and the GPU kernels in `shaders/` all land on the
//! *same* `i32` sum, and therefore the same `f32` output. Deterministic
//! by construction, not by effort.
//!
//! Overflow bound: with activations quantized to `[-16000, 16000]`, an
//! inner dimension `K ≤ 65536` keeps the accumulated `i32` under
//! `2.1e9`. The lane's models are tiny (K is a dim, ≤ 4096), far under
//! the bound; `gemm_i32` asserts it.

use crate::pack::{pack_trits, unpack_strict, PACK_PER_BYTE};

/// The activation quantization scale denominator: activations live in
/// [-16000, 16000] after scaling, i16 with headroom for residual sums.
pub const QUANT_HEADROOM: f32 = 16000.0;

/// The epsilon for the absmean ternary scale, from the BitNet b1.58
/// recipe (`γ = mean(|W|)`, weights divided by `γ + ε`).
pub const GAMMA_EPS: f32 = 1e-5;

/// Packed density: the file-truth density (2 bits / weight).
pub fn packed_bytes_for(n: usize) -> usize {
    n.div_ceil(PACK_PER_BYTE)
}

// ----------------------------------------------------------------------
// quantization
// ----------------------------------------------------------------------

/// absmax — the activation's largest magnitude; the quant scale's denom.
pub fn absmax(x: &[f32]) -> f32 {
    x.iter().fold(0.0f32, |m, v| m.max(v.abs())).max(1e-6)
}

/// quant_acts — float activations to i16 at the layer's dynamic scale.
/// `s_x = absmax(x) / QUANT_HEADROOM`, so `q = round(x / s_x)` stays
/// within the headroom band. Deterministic: same inputs, same q.
pub fn quant_acts(x: &[f32]) -> Vec<i16> {
    let s = absmax(x) / QUANT_HEADROOM;
    x.iter()
        .map(|&v| {
            let q = (v / s).round();
            q.clamp(-QUANT_HEADROOM, QUANT_HEADROOM) as i16
        })
        .collect()
}

/// The per-layer activation scale for a layer's activations (needed by
/// the forward to dequantize the i32 sum: `out = acc * s_x * γ`).
pub fn act_scale(x: &[f32]) -> f32 {
    absmax(x) / QUANT_HEADROOM
}

// ----------------------------------------------------------------------
// the reference core (always compiled, always the authority)
// ----------------------------------------------------------------------

/// gemm_i32_scalar — tri-state × i16 activations, accumulated in i32.
/// Row-major weights: `w[o * n_in + k]` is neuron `o`'s weight on input
/// `k`. This is the reference every SIMD lane must match bit-for-bit.
pub fn gemm_i32_scalar(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
    let max_acc = 16000i64 * n_in as i64;
    assert!(
        max_acc < i32::MAX as i64,
        "gemm: n_in {} exceeds the i32 accumulate bound (65536)",
        n_in
    );
    let mut out = vec![0i32; n_out];
    for o in 0..n_out {
        let mut acc: i32 = 0;
        let row = &w[o * n_in..(o + 1) * n_in];
        for k in 0..n_in {
            acc = acc.wrapping_add(row[k] as i32 * a[k] as i32);
        }
        out[o] = acc;
    }
    out
}

// ----------------------------------------------------------------------
// the AVX2 lane (x86-64 — AMD Zen and Intel alike)
// ----------------------------------------------------------------------

/// The AVX2 lane: 16 i16 input lanes, pairwise product-accumulation into
/// 8 i32 lanes via `madd`, horizontal sum at the end. Same integer sum —
/// grouping does not change integer addition.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn gemm_i32_avx2(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
    use std::arch::x86_64::*;
    let mut out = vec![0i32; n_out];
    for o in 0..n_out {
        let row = &w[o * n_in..(o + 1) * n_in];
        let mut acc = _mm256_setzero_si256();
        let mut k = 0usize;
        while k + 16 <= n_in {
            let wa = _mm256_loadu_si256(row.as_ptr().add(k) as *const __m256i);
            let aa = _mm256_loadu_si256(a.as_ptr().add(k) as *const __m256i);
            let prod = _mm256_madd_epi16(aa, wa); // 16 i16 products, pairwise → 8 i32
            acc = _mm256_add_epi32(acc, prod);
            k += 16;
        }
        let mut lanes = [0i32; 8];
        _mm256_storeu_si256(lanes.as_mut_ptr() as *mut __m256i, acc);
        let mut acc_s = 0i32;
        for &l in &lanes {
            acc_s = acc_s.wrapping_add(l);
        }
        // tail — scalar, exact
        for kk in k..n_in {
            acc_s = acc_s.wrapping_add(row[kk] as i32 * a[kk] as i32);
        }
        out[o] = acc_s;
    }
    out
}

// ----------------------------------------------------------------------
// the NEON lane (aarch64 — Apple silicon and ARM servers)
// ----------------------------------------------------------------------

/// The NEON lane: 8 i16 lanes, widened to i32 partial sums. Same
/// integer contract. NEON is the aarch64 baseline, but rustc's
/// `std::arch` still demands the feature attribute.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn gemm_i32_neon(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
    use std::arch::aarch64::*;
    let mut out = vec![0i32; n_out];
    for o in 0..n_out {
        let row = &w[o * n_in..(o + 1) * n_in];
        let mut acc_lo = vdupq_n_s32(0);
        let mut acc_hi = vdupq_n_s32(0);
        let mut k = 0usize;
        while k + 8 <= n_in {
            let wa = vld1q_s16(row.as_ptr().add(k));
            let aa = vld1q_s16(a.as_ptr().add(k));
            let prod = vmulq_s16(aa, wa); // i16x8 products ({0,±1} × i16, exact)
            acc_lo = vaddq_s32(acc_lo, vmovl_s16(vget_low_s16(prod)));
            acc_hi = vaddq_s32(acc_hi, vmovl_s16(vget_high_s16(prod)));
            k += 8;
        }
        let combined = vaddq_s32(acc_lo, acc_hi);
        let mut lanes = [0i32; 4];
        vst1q_s32(lanes.as_mut_ptr(), combined);
        let mut total = 0i32;
        for &l in &lanes {
            total = total.wrapping_add(l);
        }
        for kk in k..n_in {
            total = total.wrapping_add(row[kk] as i32 * a[kk] as i32);
        }
        out[o] = total;
    }
    out
}

// ----------------------------------------------------------------------
// the WASM simd128 lane (browser / sandbox guests)
// ----------------------------------------------------------------------

/// The simd128 lane: 8 i16 products pairwise-extended into 4 i32 lanes
/// (`extadd_pairwise` covers all 8 products in one call).
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
#[target_feature(enable = "simd128")]
unsafe fn gemm_i32_wasm(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
    use std::arch::wasm32::*;
    let mut out = vec![0i32; n_out];
    for o in 0..n_out {
        let row = &w[o * n_in..(o + 1) * n_in];
        let mut acc = 0i32;
        let mut k = 0usize;
        while k + 8 <= n_in {
            let wa = v128_load(row.as_ptr().add(k) as *const v128);
            let aa = v128_load(a.as_ptr().add(k) as *const v128);
            let prod = i16x8_mul(aa, wa); // 8 i16 products
                                          // exact i32 pairing via the lane-extract primitive (stable
                                          // across stdarch revisions); integer addition does not care
                                          // about grouping, so the total is the scalar total
            acc = acc.wrapping_add(i16x8_extract_lane::<0>(prod) as i32);
            acc = acc.wrapping_add(i16x8_extract_lane::<1>(prod) as i32);
            acc = acc.wrapping_add(i16x8_extract_lane::<2>(prod) as i32);
            acc = acc.wrapping_add(i16x8_extract_lane::<3>(prod) as i32);
            acc = acc.wrapping_add(i16x8_extract_lane::<4>(prod) as i32);
            acc = acc.wrapping_add(i16x8_extract_lane::<5>(prod) as i32);
            acc = acc.wrapping_add(i16x8_extract_lane::<6>(prod) as i32);
            acc = acc.wrapping_add(i16x8_extract_lane::<7>(prod) as i32);
            k += 8;
        }
        for kk in k..n_in {
            acc = acc.wrapping_add(row[kk] as i32 * a[kk] as i32);
        }
        out[o] = acc;
    }
    out
}

// ----------------------------------------------------------------------
// the dispatcher
// ----------------------------------------------------------------------

/// The one public GEMM: scalar + the fastest compiled lane, all producing
/// the same i32 by the integer-addition contract.
pub fn gemm_i32(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: avx2 detected; buffers are length-checked by the
            // lane (k + 16 <= n_in; rows from slice bounds).
            return unsafe { gemm_i32_avx2(w, a, n_in, n_out) };
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: NEON is aarch64's baseline; buffers are slice-bounds
        // checked by the lane (k + 8 <= n_in; rows from slice ranges).
        return unsafe { gemm_i32_neon(w, a, n_in, n_out) };
    }
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        // SAFETY: simd128 gated by the attribute and the cfg; buffers
        // slice-bounds checked (k + 16 <= n_in).
        return unsafe { gemm_i32_wasm(w, a, n_in, n_out) };
    }
    #[cfg(not(any(
        target_arch = "aarch64",
        all(target_arch = "wasm32", target_feature = "simd128")
    )))]
    {
        return gemm_i32_scalar(w, a, n_in, n_out);
    }
}

// ----------------------------------------------------------------------
// the linear op the model calls
// ----------------------------------------------------------------------

/// unpack weights from their packed 2-bit store into the i16 runtime
/// buffers the GEMM consumes (row-major: neuron o over inputs k).
pub fn unpack_weights(packed: &[u8], n_in: usize, n_out: usize) -> Vec<i16> {
    let trits = unpack_strict(packed, n_in * n_out)
        .expect("unpack_weights: the checkpoint's packed lane must be valid");
    trits.iter().map(|&t| t as i16).collect()
}

/// The reference fp32 dot product over float weights (the tolerance
/// authority for tests; never used on the runtime path).
pub fn gemm_f32_reference(w_trits: &[f32], a: &[f32], n_in: usize, n_out: usize) -> Vec<f32> {
    let mut out = vec![0f32; n_out];
    for o in 0..n_out {
        let row = &w_trits[o * n_in..(o + 1) * n_in];
        out[o] = row.iter().zip(a.iter()).map(|(wx, ax)| wx * ax).sum();
    }
    out
}

/// ternary_linear — the fully wired forward op: activations in, logits
/// per neuron out:
/// `q = quant_acts(x)`, `acc = gemm_i32(w_i16, q)`,
/// `out = acc * s_x * γ` — the scale applied exactly once.
pub fn ternary_linear(w_i16: &[i16], x: &[f32], n_in: usize, n_out: usize, gamma: f32) -> Vec<f32> {
    debug_assert_eq!(w_i16.len(), n_in * n_out);
    let s_x = act_scale(x);
    let q = quant_acts(x);
    let acc = gemm_i32(w_i16, &q, n_in, n_out);
    acc.iter().map(|&a| a as f32 * s_x * gamma).collect()
}

/// The tri-state constant-vector builder the trainer mirrors (γ-scale):
/// used by tests to build known-shape weight buffers.
pub fn gamma_of(weights: &[f32]) -> f32 {
    let sum: f32 = weights.iter().map(|w| w.abs()).sum();
    sum / weights.len() as f32
}

/// Just the pack step of the BitNet b1.58 recipe over a float weight
/// matrix (row-major n_out × n_in) — used by tests and documented for
/// the trainer: `w' = roundclip(w / (γ + ε))`, then packed.
pub fn quantize_pack(weights: &[f32], gamma: f32) -> Vec<u8> {
    let trits: Vec<i8> = weights
        .iter()
        .map(|&w| {
            let s = (w / (gamma + GAMMA_EPS)).round();
            if s >= 1.0 {
                1
            } else if s <= -1.0 {
                -1
            } else {
                0
            }
        })
        .collect();
    pack_trits(&trits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::unpack_lenient;

    fn seeded_weights(n_out: usize, n_in: usize, seed: u64) -> Vec<f32> {
        let mut s = seed;
        let mut rng = move || {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((s >> 33) as f64 / std::u64::MAX as f64) as f32
        };
        (0..n_out * n_in).map(|_| rng() * 2.0 - 1.0).collect()
    }

    #[test]
    fn scalar_matches_fp32_reference_within_bound() {
        let n_in = 256;
        let n_out = 128;
        let wf = seeded_weights(n_out, n_in, 7);
        let gamma = gamma_of(&wf);
        let packed = quantize_pack(&wf, gamma);
        let w_i16 = unpack_weights(&packed, n_in, n_out);
        let x: Vec<f32> = (0..n_in).map(|k| ((k as f32) / 64.0 - 1.0) * 3.0).collect();
        let s_x = act_scale(&x);
        let q = quant_acts(&x);
        let acc = gemm_i32_scalar(&w_i16, &q, n_in, n_out);
        let got: Vec<f32> = acc.iter().map(|&a| a as f32 * s_x * gamma).collect();

        // fp32 reference over the *quantized-in-theory* weights: build the
        // float tri-state matrix, scale it by its own (per-threshold) γ.
        let w_trits: Vec<f32> = unpack_lenient(&packed, n_in * n_out)
            .iter()
            .map(|&t| t as f32)
            .collect();
        let want = gemm_f32_reference(&w_trits, &x, n_in, n_out);

        // The integer path's only loss is the i16 activation rounding.
        let max_err = s_x * 0.5 * n_in as f32; // ≤ half a quant step per k
        for o in 0..n_out {
            let err = (got[o] - want[o]).abs();
            assert!(err <= max_err, "neuron {o}: err {err} > bound {max_err}");
        }
    }

    #[test]
    fn quant_is_deterministic_and_bounded() {
        let x: Vec<f32> = (0..4096).map(|k| ((k % 257) as f32) * 0.1 - 12.8).collect();
        let q1 = quant_acts(&x);
        let q2 = quant_acts(&x);
        assert_eq!(q1, q2);
        assert!(q1.iter().all(|&v| v.abs() <= QUANT_HEADROOM as i16));
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx2_lane_is_bit_exact_with_scalar() {
        if !std::arch::is_x86_feature_detected!("avx2") {
            return; // the contract holds wherever the lane runs
        }
        for (n_in, n_out) in [(64, 32), (256, 128), (4096, 16), (1024, 65)] {
            let wf = seeded_weights(n_out, n_in, 99);
            let gamma = gamma_of(&wf);
            let w_i16 = unpack_weights(&quantize_pack(&wf, gamma), n_in, n_out);
            let x: Vec<f32> = (0..n_in).map(|k| ((k % 31) as f32) * 0.5 - 7.5).collect();
            let q = quant_acts(&x);
            let scalar = gemm_i32_scalar(&w_i16, &q, n_in, n_out);
            let simd = unsafe { gemm_i32_avx2(&w_i16, &q, n_in, n_out) };
            assert_eq!(scalar, simd, "({n_in},{n_out}) must be bit-exact");
            assert_eq!(scalar, gemm_i32(&w_i16, &q, n_in, n_out));
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn neon_lane_is_bit_exact_with_scalar() {
        for (n_in, n_out) in [(64, 32), (4096, 16), (4093, 8)] {
            let wf = seeded_weights(n_out, n_in, 41);
            let gamma = gamma_of(&wf);
            let w_i16 = unpack_weights(&quantize_pack(&wf, gamma), n_in, n_out);
            let x: Vec<f32> = (0..n_in).map(|k| ((k % 17) as f32) - 8.0).collect();
            let q = quant_acts(&x);
            assert_eq!(gemm_i32_scalar(&w_i16, &q, n_in, n_out), unsafe {
                gemm_i32_neon(&w_i16, &q, n_in, n_out)
            });
        }
    }

    #[test]
    fn i32_accumulate_bound_is_safe_to_4096() {
        // A full-scale adversarial layer: every activation at -16000 and
        // every weight +1 → |acc| ≈ 65,536,000 < i32::MAX. No overflow.
        let n_in = 4096;
        let n_out = 8;
        let w_i16 = vec![1i16; n_in * n_out];
        let a = vec![-16000i16; n_in];
        let acc = gemm_i32(&w_i16, &a, n_in, n_out);
        assert_eq!(acc[0], -(16000i32 * n_in as i32));
    }

    #[test]
    fn quantize_pack_round_trip_shape() {
        let wf = seeded_weights(10, 20, 3);
        let gamma = gamma_of(&wf);
        let packed = quantize_pack(&wf, gamma);
        assert_eq!(packed.len(), packed_bytes_for(200));
        assert_eq!(unpack_weights(&packed, 20, 10).len(), 200);
    }

    #[test]
    fn large_layer_does_not_overflow_i64_authority() {
        // Simulate n_in beyond the documented i32 bound (16000·n ≥ 2.1e9).
        let n_in = 200_000; // beyond the bound → the assert fires
        let w_i16 = vec![1i16; n_in * 2];
        let a = vec![16000i16; n_in];
        let result = std::panic::catch_unwind(|| gemm_i32_scalar(&w_i16, &a, n_in, 2));
        assert!(result.is_err(), "the bound must be enforced");
    }
}
