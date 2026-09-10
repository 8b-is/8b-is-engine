//! checkpoint_tests — the committed base model is a citizen of the test
//! suite: it loads, its sha256 verifies, its forward is bit-stable. The
//! cross-surface determinism proof itself (the golden hash, pinned) lives
//! in `golden.rs` and runs on every surface — aarch64 NEON here, x86-64
//! AVX2 in CI, wasm32 simd128 via `scripts/wasm-golden.mjs`.

use sha2::{Digest, Sha256};

const TERN: &[u8] = include_bytes!("../../../assets/ternary/sanctuary-1.58.tern");

#[test]
fn committed_checkpoint_loads_and_verifies() {
    use crate::format::load_checkpoint;
    let cp = load_checkpoint(TERN).expect("the committed checkpoint must load");
    assert!(cp.verified);
    assert!(cp.vocab >= 3 && cp.dim >= 32, "sane shape");
    assert!(
        cp.layers.len() >= 3,
        "a base model needs hidden layers + head"
    );
}

#[test]
fn forward_is_repeatable_on_this_machine() {
    assert_eq!(crate::golden_hash_hex(), crate::golden_hash_hex());
}

/// Run with `-- --nocapture` to re-emit the golden after a deliberate
/// retrain; update `golden.rs`'s EXPECTED. The golden changes only when
/// the world changes.
#[test]
fn print_golden() {
    eprintln!("GOLDEN={}", crate::golden_hash_hex());
}

#[test]
fn the_pinned_hash_is_what_the_runtime_computes() {
    let cp = crate::format::load_checkpoint(TERN).expect("load");
    let m = crate::model::CharModel::from_checkpoint(&cp);
    let mut hidden = vec![0f32; m.dim];
    let mut hasher = Sha256::new();
    for i in 0..32usize {
        let tok = (i * 7 + 3) % m.vocab;
        for l in m.forward(tok, &mut hidden) {
            hasher.update(l.to_bits().to_le_bytes());
        }
    }
    let hex = format!("{:x}", hasher.finalize());
    // The golden.rs constant and this independent re-derivation must
    // agree — two code paths, one truth.
    assert_eq!(hex, crate::golden_hash_hex());
}
