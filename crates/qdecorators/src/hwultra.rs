//! hwultra — the hardware-ultra abstraction contracts: the lanes the
//! 1.58-bit engine runs on, named as a type; the GEMM and pack
//! contracts those lanes satisfy — same i32 accumulation, same
//! tri-state alphabet, on every board.
//!
//! `world-core::gemm` implements these contracts; this module is the
//! interface the rest of the constellation programs against.

/// The transport lanes — the engine's boards, first-class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    /// the portable reference kernel
    Scalar,
    /// x86-64: AMD Zen and Intel alike
    Avx2,
    /// aarch64: Apple silicon and ARM servers
    Neon,
    /// wasm32: browser and sandbox guests
    Simd128,
    /// Vulkan compute: AMD RADV native, every vendor portable
    Vulkan,
    /// Apple's Metal
    Msl,
}

/// The gamma scale of the BitNet b1.58 recipe: γ = mean(|W|), weights
/// divided by γ + ε — the single float scale applied once, at the end.
#[derive(Debug, Clone, Copy)]
pub struct GammaScale {
    pub gamma: f32,
    pub eps: f32,
}

impl GammaScale {
    pub const EPS: f32 = 1e-5;

    pub fn of(weights: &[f32]) -> Self {
        let sum: f32 = weights.iter().map(|w| w.abs()).sum();
        GammaScale {
            gamma: sum / weights.len().max(1) as f32,
            eps: Self::EPS,
        }
    }

    /// scale — the one float multiply the contract allows.
    pub fn scale(&self, acc: i32, s_x: f32) -> f32 {
        acc as f32 * s_x * self.gamma
    }
}

/// GEMM — the integer-accumulation contract: `w` is i16 in {-1,0,+1},
/// `a` is i16 activations, the sum accumulates in i32 (associative ⇒
/// bit-exact across every lane), and the float scale is applied once —
/// by the caller, via `GammaScale::scale`.
pub trait GEMM {
    fn gemm_i32(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32>;
}

/// TernaryPack — the tri-state store: {-1,0,+1} packed two bits each,
/// four to a byte, `0b11` reserved (strict refuses, lenient zeroes).
pub trait TernaryPack {
    fn pack(trits: &[i8]) -> Vec<u8>;
    fn unpack(bytes: &[u8], n: usize) -> Result<Vec<i8>, String>;
    fn per_byte() -> usize {
        4
    }
}

/// The engine's board list, for the review eye and the dashboards.
pub const TRANSPORTS: &[Transport] = &[
    Transport::Scalar,
    Transport::Avx2,
    Transport::Neon,
    Transport::Simd128,
    Transport::Vulkan,
    Transport::Msl,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamma_is_the_absmean_and_scales_once() {
        let w = [1.0f32, -2.0, 0.0, 3.0];
        let g = GammaScale::of(&w);
        assert!((g.gamma - 1.5).abs() < 1e-6, "mean |w| = 6/4");
        let out = g.scale(100, 0.5);
        assert!((out - 75.0).abs() < 1e-4, "100 · 0.5 · 1.5");
    }

    #[test]
    fn the_transport_names_are_stable() {
        assert_eq!(TRANSPORTS.len(), 6);
        assert!(TRANSPORTS.contains(&Transport::Msl));
        assert!(TRANSPORTS.contains(&Transport::Vulkan));
    }

    /// the contract in miniature: an i32 accumulator that any lane must
    /// equal — the kit's own reference, that `world-core::gemm` mirrors
    fn reference_gemm(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
        let mut out = vec![0i32; n_out];
        for o in 0..n_out {
            let row = &w[o * n_in..(o + 1) * n_in];
            out[o] = row
                .iter()
                .zip(a.iter())
                .map(|(&x, &y)| x as i32 * y as i32)
                .sum();
        }
        out
    }

    #[test]
    fn the_contract_is_the_integer_sum() {
        let w: Vec<i16> = vec![1, -1, 0, 1, 1, 1, -1, 0];
        let a: Vec<i16> = vec![3, 4, 5, 6, 7, 8, 9, 1];
        let got = reference_gemm(&w, &a, 8, 1);
        assert_eq!(got, vec![3 - 4 + 6 + 7 + 8 - 9]);
        // and the same sum in the reverse grouping — the associativity
        // that makes every lane equal every other
        let reversed: i32 = w
            .iter()
            .rev()
            .zip(a.iter().rev())
            .map(|(&x, &y)| x as i32 * y as i32)
            .sum();
        assert_eq!(got[0], reversed);
    }
}
