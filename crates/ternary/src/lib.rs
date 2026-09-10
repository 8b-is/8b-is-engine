//! ternary — the 1.58-bit lane: BitNet b1.58 base models as first-class
//! citizens.
//!
//! Where `kompress` is the mesh's compression lane, this crate is the
//! engine's **model lane**: weights live in three states {-1, 0, +1},
//! packed two bits each, four to a byte, and the forward pass is an
//! integer GEMM that is **bit-exact across architectures** — the same i32
//! sum on an AMD Zen CPU (AVX2), an Apple/ARM Neoverse core (NEON), a WASM
//! sandbox (simd128), and the GPU lanes (Vulkan compute / MSL), because
//! integer addition does not care about the order you add in, and the
//! single float scale is applied once, at the end.
//!
//! The engine's doctrine holds here too: a seed is a seed everywhere
//! (the dream uses `world-core::tern::mulberry32`), the checkpoint is a
//! small public file (the `.tern` format, sections + sha256 trailer), and
//! the model is small so the world is portable.
//!
//! # The lane at a glance
//!
//! ```text
//! checkpoint (.tern) → load → embed → per layer:
//!     layernorm → quant (i16) → ternary GEMM (i32, bit-exact) → scale
//!     → relu → residual
//! → logits → seeded sample → the dream
//! ```
//!
//! AMD and 1-bit models are first-class here: there is no metal board and
//! no apple gate — the arithmetic contract is one, and the shader lanes in
//! `shaders/` are written to the same contract.

#[cfg(test)]
pub mod checkpoint_tests;
pub mod format;
pub mod gemm;
pub mod golden;
pub mod model;
pub mod pack;
pub mod sample;
#[cfg(target_arch = "wasm32")]
pub mod wasm_abi;

pub use format::{load_checkpoint, Checkpoint, SectionKind, TernLayer};
pub use gemm::{
    absmax, act_scale, gamma_of, gemm_f32_reference, gemm_i32, gemm_i32_scalar, packed_bytes_for,
    quant_acts, quantize_pack, ternary_linear, unpack_weights, GAMMA_EPS, QUANT_HEADROOM,
};
pub use golden::golden_hash_hex;
pub use model::{layernorm, CharModel};
pub use pack::{pack_trits, packed_len, unpack_lenient, unpack_strict, PACK_PER_BYTE};
pub use sample::{argmax, sample, softmax};

/// The lane's tagline — the engine's voice for the 1.58-bit lane.
pub const TAGLINE: &str =
    "the 1.58-bit lane :: add and add and add until the order does not matter";

/// The dream's PRNG, wrapped as an `FnMut` — the engine's own
/// `mulberry32`, so a seed is a seed everywhere. The lane keeps a local
/// copy so the wasm surface stays dependency-free (world-core and its
/// zstd lane never board the wasm ship); a test pins this copy
/// bit-identical to `world-core::tern::mulberry32`, the one truth.
pub fn pack_mulberry(seed: u32) -> impl FnMut() -> f64 {
    let mut a = seed;
    move || {
        a = a.wrapping_add(0x6D2B79F5);
        let mut t = a;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61)) ^ t;
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }
}

#[cfg(test)]
mod prng_tests {
    #[test]
    fn pack_mulberry_is_the_engine_family() {
        // world-core is a dev-dependency: assert bit-identity against the
        // authoritative implementation on every surface this test runs.
        let mut ours = super::pack_mulberry(41592);
        let mut theirs = world_core::tern::mulberry32(41592);
        for _ in 0..128 {
            assert_eq!(ours(), theirs(), "the dream's PRNG drifted from the family");
        }
    }

    #[test]
    fn pack_mulberry_is_seeded_and_repeatable() {
        let mut a = super::pack_mulberry(7);
        let first: Vec<f64> = (0..16).map(|_| a()).collect();
        let mut b = super::pack_mulberry(7);
        let again: Vec<f64> = (0..16).map(|_| b()).collect();
        assert_eq!(first, again);
        let mut c = super::pack_mulberry(8);
        assert_ne!(first[0], c(), "different seeds diverge immediately");
    }
}
