//! golden — the cross-surface determinism proof.
//!
//! The committed base model's first 32 forward logits are hashed into 64
//! hex chars. That same string must come out of the NEON lane (aarch64),
//! the AVX2 lane (x86-64, CI), and the simd128 lane (wasm32, run by
//! `scripts/wasm-golden.mjs` from Node). A changed golden means the world
//! changed — deliberately retrain and re-pin, never patch to pass.

use sha2::Digest;

#[cfg(test)]
const EXPECTED: &str = "aff6dc2bc980c1a2275957b25ee075139670e3bbad4920a0aec4f9c6fc952060";

/// The golden hex string for the committed checkpoint.
pub fn golden_hash_hex() -> String {
    let cp = crate::format::load_checkpoint(crate::TERN_ASSET)
        .expect("the committed checkpoint must load");
    let m = crate::model::CharModel::from_checkpoint(&cp);
    let mut hidden = vec![0f32; m.dim];
    let mut hasher = sha2::Sha256::new();
    for i in 0..32usize {
        let tok = (i * 7 + 3) % m.vocab;
        for l in m.forward(tok, &mut hidden) {
            hasher.update(l.to_bits().to_le_bytes());
        }
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_golden_holds_on_this_surface() {
        let got = super::golden_hash_hex();
        assert_eq!(
            &got,
            super::EXPECTED,
            "the base model's forward drifted on this architecture"
        );
    }
}
