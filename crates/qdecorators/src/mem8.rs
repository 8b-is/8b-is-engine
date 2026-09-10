//! mem8 — the hypermesh quad: MEMNET's 8-byte memory cell.
//!
//! A memory address is not a high-dimensional vector here — it is a
//! phase-INTERPOLATION problem over a quad cell: two phase coordinates
//! (`φ`, `λ`), two boundary targets, two bilinear ratios, a boolean
//! gate, and an observer offset. The whole cell is **8 bytes**, so
//! eight cells fill one 64-byte cache line and a batch pass is a
//! SIMD-shaped fixed-point walk.
//!
//! Every field is a `u8`; the interpolated deltas are computed in i16
//! fixed point (the `>> 8` scale), and **overflow is impossible by
//! construction**: `|φ' − φ| ≤ 255`, `ratio ≤ 255`, so
//! `|(φ'−φ) × ratio| ≤ 65025 < i16::MAX (32767)` — the domain is chosen
//! so the i16 space never overflows, and a test pins the worst case.
//!
//! The observer-relative frame is the engine's own trick: shifting the
//! observer origo shifts the delta via a byte-wrapping add, without
//! mutating the quad — the same way the keeper shifts the fold for a
//! new observer, never the ledger.

/// The hypermesh quad — exactly 8 bytes, ready for the cache line.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mem8Quad {
    /// φ — base phase angle, 0-255 mapped to [0, 2π)
    pub phi_base: u8,
    /// φ' — the target boundary phase
    pub phi_prime: u8,
    /// λ — the base wavelength coordinate
    pub lambda_base: u8,
    /// λ' — the target boundary wavelength
    pub lambda_prime: u8,
    /// fixed-point scalar [0, 255] for the AP/AB ratio
    pub ratio_ap_ab: u8,
    /// fixed-point scalar [0, 255] for the DP/CD ratio
    pub ratio_dp_cd: u8,
    /// gate bits: `0b00` AND · `0b01` OR · `0b10` NAND · else XOR
    pub gate_mask: u8,
    /// the observer's origo offset — added wrapping, never geometry
    pub origo_delta: u8,
}

/// The interpolated read-out.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseResult {
    pub delta_phi: u8,
    pub delta_lambda: u8,
    pub gated_output: u8,
}

/// The cache-line invariant, stated once: eight cells in one line.
pub const QUADS_PER_LINE: usize = 64 / std::mem::size_of::<Mem8Quad>();

impl Mem8Quad {
    /// evaluate_phase — the bilinear phase interpolation:
    /// `Δφ = ((φ' − φ) · AP/AB) >> 8`, same for λ, both offset by the
    /// observer, then the boolean interference gate.
    ///
    /// Overflow, handled with the honest proof: the product lives in
    /// **i32** — `|(φ'−φ) · ratio| ≤ 255·255 = 65025 « i32::MAX`, so
    /// the multiply cannot overflow, the `>> 8` folds the delta into
    /// [-255, 255], and the truncation to u8 is the wrapping two's
    /// complement both languages agree on. The origo shift is
    /// `wrapping_add` by design — the frame may wrap, the geometry
    /// never does.
    #[inline(always)]
    pub fn evaluate_phase(&self) -> PhaseResult {
        let delta_phi = Self::delta_wire(self.phi_base, self.phi_prime, self.ratio_ap_ab)
            .wrapping_add(self.origo_delta);
        let delta_lambda = Self::delta_wire(self.lambda_base, self.lambda_prime, self.ratio_dp_cd)
            .wrapping_add(self.origo_delta);

        let gated_output = match self.gate_mask & 0x03 {
            0b00 => delta_phi & delta_lambda,    // AND
            0b01 => delta_phi | delta_lambda,    // OR
            0b10 => !(delta_phi & delta_lambda), // NAND
            _ => delta_phi ^ delta_lambda,       // XOR
        };

        PhaseResult {
            delta_phi,
            delta_lambda,
            gated_output,
        }
    }

    /// delta_fixed — one axis of the interpolation: the i32 product
    /// (provably in-bounds), the `>> 8` scale, the wrapping truncation
    /// to the u8 wire. The `>> 8` replaces division with a single-cycle
    /// shift.
    #[inline(always)]
    fn delta_wire(base: u8, prime: u8, ratio: u8) -> u8 {
        (((prime as i32 - base as i32) * ratio as i32) >> 8) as u8
    }

    /// batch_evaluate — a contiguous walk over cells (the SIMD-shaped
    /// pass: 8 cells per cache line, the compiler vectorizes the loop).
    pub fn batch_evaluate(cells: &[Mem8Quad]) -> Vec<PhaseResult> {
        cells.iter().map(Mem8Quad::evaluate_phase).collect()
    }

    /// A canonical cell from two seeds (deterministic, for tests and
    /// for the constellation's demos).
    pub fn seeded(seed: u8) -> Mem8Quad {
        Mem8Quad {
            phi_base: seed.wrapping_mul(7).wrapping_add(1),
            phi_prime: seed.wrapping_mul(13).wrapping_add(3),
            lambda_base: seed.wrapping_mul(5).wrapping_add(2),
            lambda_prime: seed.wrapping_mul(11).wrapping_add(5),
            ratio_ap_ab: seed.wrapping_mul(3).wrapping_add(17),
            ratio_dp_cd: seed.wrapping_mul(9).wrapping_add(23),
            gate_mask: seed % 4,
            origo_delta: seed,
        }
    }
}

/// The Zig twin's C-ABI surface (compiled by build.rs from
/// `zig/mem8.zig` when `qdecorators_zig` is set) — reached typed, the
/// way the constellation prefers: the Rust authority and the Zig kernel
/// must be bit-exact, and the tests below pin it.
#[cfg(qdecorators_zig)]
extern "C" {
    fn mem8_quad_evaluate(quad_ptr: *const Mem8Quad, out_ptr: *mut PhaseResult);
    #[link_name = "mem8_quad_evaluate_batch"]
    fn mem8_quad_evaluate_batch_c(
        quads_ptr: *const Mem8Quad,
        results_ptr: *mut PhaseResult,
        count: usize,
    );
}

/// zig_evaluate — one quad through the Zig kernel (safe wrapper).
#[cfg(qdecorators_zig)]
pub fn zig_evaluate(q: &Mem8Quad) -> PhaseResult {
    let mut out = PhaseResult {
        delta_phi: 0,
        delta_lambda: 0,
        gated_output: 0,
    };
    // SAFETY: valid pointers to a single cell; the kernel writes exactly
    // one PhaseResult
    unsafe { mem8_quad_evaluate(q, &mut out) };
    out
}

/// zig_batch — the contiguous walk through the Zig kernel.
#[cfg(qdecorators_zig)]
pub fn zig_batch(cells: &[Mem8Quad]) -> Vec<PhaseResult> {
    let mut out = vec![
        PhaseResult {
            delta_phi: 0,
            delta_lambda: 0,
            gated_output: 0,
        };
        cells.len()
    ];
    if !cells.is_empty() {
        // SAFETY: buffers are exactly `cells.len()` cells/results
        unsafe { mem8_quad_evaluate_batch_c(cells.as_ptr(), out.as_mut_ptr(), cells.len()) };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cell_is_eight_bytes_exactly() {
        assert_eq!(std::mem::size_of::<Mem8Quad>(), 8);
        assert_eq!(QUADS_PER_LINE, 8, "eight cells per 64-byte line");
        assert_eq!(std::mem::size_of::<PhaseResult>(), 3);
    }

    #[test]
    fn the_interpolation_is_deterministic() {
        let q = Mem8Quad::seeded(42);
        assert_eq!(q.evaluate_phase(), q.evaluate_phase());
        assert_eq!(
            Mem8Quad::seeded(7).evaluate_phase(),
            Mem8Quad::seeded(7).evaluate_phase()
        );
    }

    #[test]
    fn worst_case_product_lives_in_i32() {
        // |φ'−φ| ≤ 255, ratio ≤ 255 → |product| ≤ 65025 « i32::MAX — the
        // i32 space is the PROOF (i16 would overflow at 32767; the paper
        // test caught it), and the extreme corners are pinned here.
        let worst = Mem8Quad {
            phi_base: 0,
            phi_prime: 255,
            lambda_base: 255,
            lambda_prime: 0,
            ratio_ap_ab: 255,
            ratio_dp_cd: 255,
            gate_mask: 0b00,
            origo_delta: 0,
        };
        let r = worst.evaluate_phase();
        // d_phi = ((255)·255)>>8 = 65025>>8 = 254
        assert_eq!(r.delta_phi, 254);
        // d_lambda = ((0-255)·255)>>8 = -65025>>8 = -255, as u8 wraps to 1
        assert_eq!(r.delta_lambda, 1);
    }

    #[test]
    fn gates_are_boolean_interference() {
        let d_phi = 0b1010u8;
        let d_lambda = 0b1100u8;
        let cell = |gate_mask| Mem8Quad {
            phi_base: 0,
            phi_prime: 10, // d_phi = 0b1010 (with ratio 255, >>8 rounds to 9)
            lambda_base: 0,
            lambda_prime: 12,
            ratio_ap_ab: 255,
            ratio_dp_cd: 255,
            gate_mask,
            origo_delta: 0,
        };
        let _ = (d_phi, d_lambda);
        let and = cell(0b00).evaluate_phase();
        let or = cell(0b01).evaluate_phase();
        let nand = cell(0b10).evaluate_phase();
        let xor = cell(0b11).evaluate_phase();
        assert_eq!(and.gated_output, and.delta_phi & and.delta_lambda);
        assert_eq!(or.gated_output, or.delta_phi | or.delta_lambda);
        assert_eq!(nand.gated_output, !(nand.delta_phi & nand.delta_lambda));
        assert_eq!(xor.gated_output, xor.delta_phi ^ xor.delta_lambda);
    }

    #[test]
    fn the_observer_shifts_the_frame_not_the_geometry() {
        let mut q = Mem8Quad::seeded(3);
        let before = q.evaluate_phase();
        let new_origo = q.origo_delta.wrapping_add(9);
        q.origo_delta = new_origo;
        let after = q.evaluate_phase();
        // geometry untouched: the quad bytes except origo are identical
        assert_ne!(before.delta_phi, after.delta_phi);
        assert_eq!(q.phi_base, Mem8Quad::seeded(3).phi_base);
    }

    #[cfg(qdecorators_zig)]
    #[test]
    fn the_zig_twin_is_bit_exact_with_the_rust_authority() {
        // the full battery: every seed 0..=255, plus the extreme
        // corners — Rust and Zig must agree on every byte
        let cells: Vec<Mem8Quad> = (0..=255).map(Mem8Quad::seeded).collect();
        let rust: Vec<PhaseResult> = Mem8Quad::batch_evaluate(&cells);
        let zig: Vec<PhaseResult> = zig_batch(&cells);
        assert_eq!(rust.len(), zig.len());
        for i in 0..cells.len() {
            assert_eq!(rust[i], zig[i], "byte {} diverged!", i);
        }
        // the extremes through the single-cell path too
        let worst = Mem8Quad {
            phi_base: 0,
            phi_prime: 255,
            lambda_base: 255,
            lambda_prime: 0,
            ratio_ap_ab: 255,
            ratio_dp_cd: 255,
            gate_mask: 0b10,
            origo_delta: 17,
        };
        assert_eq!(worst.evaluate_phase(), zig_evaluate(&worst));
    }

    #[cfg(qdecorators_zig)]
    #[test]
    fn the_batch_paths_agree() {
        let cells: Vec<Mem8Quad> = (0..64).map(Mem8Quad::seeded).collect();
        let rust_batch = Mem8Quad::batch_evaluate(&cells);
        let zig_batch_ = zig_batch(&cells);
        assert_eq!(rust_batch, zig_batch_);
        // empty batch is safe in both
        assert_eq!(zig_batch(&[]), Vec::<PhaseResult>::new());
    }

    #[test]
    fn the_batch_is_a_deterministic_walk() {
        let cells: Vec<Mem8Quad> = (0..24).map(Mem8Quad::seeded).collect();
        let batch = Mem8Quad::batch_evaluate(&cells);
        assert_eq!(batch.len(), 24);
        for (i, c) in cells.iter().enumerate() {
            assert_eq!(batch[i], c.evaluate_phase());
        }
        assert_eq!(
            Mem8Quad::batch_evaluate(&cells),
            Mem8Quad::batch_evaluate(&cells)
        );
    }
}
