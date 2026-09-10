//! checkpoint_tests — the committed base model is a citizen of the test
//! suite: it loads, its sha256 verifies, its forward is bit-stable, and
//! its first 32 logits are pinned to a golden hash so that ANY drift on
//! ANY architecture (AMD, ARM, WASM, debug, release) fails loudly here.

use sha2::{Digest, Sha256};

const TERN: &[u8] = include_bytes!("../../../assets/ternary/sanctuary-1.58.tern");

fn golden_hash() -> String {
    let cp = crate::format::load_checkpoint(TERN).expect("the committed checkpoint must load");
    assert!(cp.verified, "the committed checkpoint must verify");
    let m = crate::model::CharModel::from_checkpoint(&cp);
    let mut hidden = vec![0f32; m.dim];
    let mut hasher = Sha256::new();
    for i in 0..32usize {
        let tok = (i * 7 + 3) % m.vocab;
        let logits = m.forward(tok, &mut hidden);
        for l in logits {
            hasher.update(l.to_bits().to_le_bytes());
        }
    }
    format!("{:x}", hasher.finalize())
}

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
fn forward_logits_are_pinned_to_the_golden_hash() {
    assert_eq!(
        golden_hash(),
        "aff6dc2bc980c1a2275957b25ee075139670e3bbad4920a0aec4f9c6fc952060",
        "the base model's forward drifted on this architecture"
    );
}

#[test]
fn forward_is_repeatable_on_this_machine() {
    assert_eq!(golden_hash(), golden_hash());
}

/// Run with `-- --nocapture` to re-emit the golden after a deliberate
/// retrain; copy the value into the pinned test above. This ritual is
/// the lane's honesty: the golden changes only when the world changes.
#[test]
fn print_golden() {
    eprintln!("GOLDEN={}", golden_hash());
}
