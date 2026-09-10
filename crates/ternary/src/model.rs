//! model — the forward pass: a tiny BitNet b1.58-style base model.
//!
//! The architecture is deliberately small and honest — an embedding
//! table (full precision, as in the b1.58 recipe) over L-1 residual
//! blocks of `LayerNorm → ternary Linear → ReLU`, and a head
//! `LayerNorm → ternary Linear → V logits`. Every ternary linear uses
//! the lane's integer GEMM (see `gemm`), so the whole forward is
//! bit-exact across architectures; the weights live in the `.tern`
//! checkpoint at 2 bits apiece.
//!
//! The model carries a hidden state — the previous token's residual path
//! — so it is a tiny sequence model: each step conditions on the entire
//! dream so far, the same way the ledger conditions the world.

use crate::format::Checkpoint;
use crate::gemm::{ternary_linear, unpack_weights};

const LN_EPS: f32 = 1e-5;

/// The runtime model: unpacked weights, ready to dream.
#[derive(Debug, Clone)]
pub struct CharModel {
    pub vocab: usize,
    pub dim: usize,
    pub emb: Vec<f32>,
    /// Unpacked row-major i16 weights, one buffer per layer.
    pub layers: Vec<Vec<i16>>,
    /// The absmean scale per layer.
    pub gammas: Vec<f32>,
    /// The per-layer output widths.
    pub outs: Vec<usize>,
}

impl CharModel {
    pub fn from_checkpoint(cp: &Checkpoint) -> CharModel {
        let outs: Vec<usize> = cp.layers.iter().map(|l| l.out as usize).collect();
        let layers: Vec<Vec<i16>> = cp
            .layers
            .iter()
            .map(|l| unpack_weights(&l.w_packed, l.n_in(), l.out as usize))
            .collect();
        let gammas = cp.layers.iter().map(|l| l.gamma).collect();
        CharModel {
            vocab: cp.vocab,
            dim: cp.dim,
            emb: cp.emb.clone(),
            layers,
            gammas,
            outs,
        }
    }

    /// One step: token `tok` in, logits out (len `vocab`), the hidden
    /// carry updated in place (callers own a zero-initialized hidden).
    pub fn forward(&self, tok: usize, hidden: &mut Vec<f32>) -> Vec<f32> {
        debug_assert!(tok < self.vocab, "token out of vocab");
        debug_assert_eq!(hidden.len(), self.dim);
        let mut x = vec![0f32; self.dim];
        let e = &self.emb[tok * self.dim..(tok + 1) * self.dim];
        for i in 0..self.dim {
            x[i] = e[i] + hidden[i];
        }

        // residual blocks: LayerNorm → ternary linear → ReLU, added back.
        // The LN keeps the activation variance bounded, which is exactly
        // what the i16 quant's dynamic scale expects.
        for l in 0..self.layers.len().saturating_sub(1) {
            let ln = layernorm(&x, LN_EPS);
            let y = ternary_linear(&self.layers[l], &ln, self.dim, self.outs[l], self.gammas[l]);
            for i in 0..x.len() {
                x[i] += y[i].max(0.0); // ReLU then residual
            }
        }

        // the head projects to vocab; the hidden carry is the residual
        // path (pre-head), so the history survives the next step.
        hidden.copy_from_slice(&x);
        let last = self.layers.len() - 1;
        let ln = layernorm(&x, LN_EPS);
        ternary_linear(
            &self.layers[last],
            &ln,
            self.dim,
            self.outs[last],
            self.gammas[last],
        )
    }

    /// The model's parameter count in packed tri-states (informational).
    pub fn tri_state_params(&self) -> usize {
        self.layers.iter().map(|w| w.len()).sum()
    }
}

/// LayerNorm (mean/var, no affine — the recipe's minimal form).
pub fn layernorm(x: &[f32], eps: f32) -> Vec<f32> {
    let n = x.len() as f32;
    let mean = x.iter().sum::<f32>() / n;
    let var = x.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n;
    let inv = 1.0 / (var + eps).sqrt();
    x.iter().map(|v| (v - mean) * inv).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::tests::build_toy;

    #[test]
    fn forward_is_deterministic() {
        let bytes = build_toy(4, 8, 2);
        let cp = crate::format::load_checkpoint(&bytes).unwrap();
        let m = CharModel::from_checkpoint(&cp);
        let mut h1 = vec![0f32; m.dim];
        let mut h2 = vec![0f32; m.dim];
        let l1 = m.forward(1, &mut h1);
        let l2 = m.forward(1, &mut h2);
        assert_eq!(l1, l2);
        assert_eq!(h1, h2);
        assert_eq!(l1.len(), 4);
    }

    #[test]
    fn zero_weights_give_uniform_logits() {
        let bytes = build_toy(4, 8, 2); // all weights 0, emb 0.25
        let cp = crate::format::load_checkpoint(&bytes).unwrap();
        let m = CharModel::from_checkpoint(&cp);
        let mut h = vec![0f32; m.dim];
        let logits = m.forward(0, &mut h);
        // embedding 0.25 enters x, LN normalizes it → ternary linear of a
        // LN'd vector against zero weights → acc 0 → logits 0.
        assert_eq!(logits.iter().map(|l| l.abs()).sum::<f32>(), 0.0);
    }

    #[test]
    fn history_changes_the_future() {
        let bytes = crate::format::tests::build_toy_wired(4, 8, 2);
        let cp = crate::format::load_checkpoint(&bytes).unwrap();
        let m = CharModel::from_checkpoint(&cp);
        let mut h = vec![0f32; m.dim];
        m.forward(0, &mut h); // embed token 0
        let mut hb = vec![0f32; m.dim];
        m.forward(1, &mut hb); // embed token 1
        let l0a = m.forward(2, &mut h);
        let l1a = m.forward(2, &mut hb);
        assert_ne!(l0a, l1a, "different histories must diverge");
        assert!(l0a.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn hidden_carry_is_the_sequence_memory() {
        let bytes = crate::format::tests::build_toy_wired(4, 8, 2);
        let cp = crate::format::load_checkpoint(&bytes).unwrap();
        let m = CharModel::from_checkpoint(&cp);
        // Two different prefixes that share the final token.
        let mut h = vec![0f32; m.dim];
        m.forward(0, &mut h);
        m.forward(1, &mut h);
        let end_a = m.forward(2, &mut h);
        let mut h2 = vec![0f32; m.dim];
        m.forward(1, &mut h2);
        m.forward(0, &mut h2);
        let end_b = m.forward(2, &mut h2);
        assert_ne!(end_a, end_b, "the carry must remember the order");
    }

    #[test]
    fn model_matches_checkpoint_shape() {
        let bytes = build_toy(4, 8, 2);
        let cp = crate::format::load_checkpoint(&bytes).unwrap();
        let m = CharModel::from_checkpoint(&cp);
        assert_eq!(m.vocab, 4);
        assert_eq!(m.dim, 8);
        assert_eq!(m.outs, vec![8, 4]);
        assert_eq!(m.tri_state_params(), 8 * 8 + 8 * 4);
    }

    #[test]
    fn layernorm_reference_values() {
        let x = [2.0f32, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]; // variance 4.0
        let ln = layernorm(&x, LN_EPS);
        let mean: f32 = ln.iter().sum::<f32>() / ln.len() as f32;
        let var: f32 = ln.iter().map(|v| v * v).sum::<f32>() / ln.len() as f32;
        assert!((mean).abs() < 1e-4, "zero mean");
        assert!((var - 1.0).abs() < 1e-3, "unit variance");
    }
}
