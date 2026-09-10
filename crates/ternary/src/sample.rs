//! sample — drawing the next token from logits, with the engine's own
//! PRNG. Determinism here means: a seed is a seed everywhere — the same
//! temperature and the same `mulberry32` sequence give the same dream on
//! an AMD Zen laptop, an Apple core, a WASM sandbox, and a server farm.

/// softmax — stable (max-subtracted), the dream's distribution.
pub fn softmax(logits: &[f32]) -> Vec<f32> {
    let max = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = logits.iter().map(|l| (l - max).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|e| e / sum).collect()
}

/// argmax — the greedy read, `temperature → 0`'s limit. Ties resolve to
/// the lowest index (deterministic by construction).
pub fn argmax(logits: &[f32]) -> usize {
    logits
        .iter()
        .enumerate()
        .max_by(|(i, a), (j, b)| a.partial_cmp(b).unwrap().then_with(|| j.cmp(i)))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// sample — one token from the temperature-scaled distribution.
/// `temperature` below `1e-3` degenerates to argmax (documented; the
/// engine's dreams use 0.6-1.0 where the full support matters).
pub fn sample<T: FnMut() -> f64>(logits: &[f32], temperature: f32, rng: &mut T) -> usize {
    if temperature < 1e-3 {
        return argmax(logits);
    }
    let scaled: Vec<f32> = logits.iter().map(|l| l / temperature).collect();
    let probs = softmax(&scaled);
    let u = rng() as f32;
    let mut c = 0f32;
    for (i, p) in probs.iter().enumerate() {
        c += p;
        if u < c {
            return i;
        }
    }
    probs.len() - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn softmax_sums_to_one_and_is_stable() {
        let logits = [1000.0f32, 1000.0, 0.0, -1000.0];
        let p = softmax(&logits);
        let sum: f32 = p.iter().sum();
        assert!((sum - 1.0).abs() < 1e-4);
        assert!(p[0] > 0.49 && p[1] > 0.49 && p[3] < 1e-6);
    }

    #[test]
    fn argmax_finds_the_peak() {
        assert_eq!(argmax(&[0.1, 9.9, -3.0, 2.0]), 1);
        // ties → lowest index
        assert_eq!(argmax(&[5.0, 5.0, 1.0]), 0);
    }

    #[test]
    fn near_zero_temperature_is_greedy() {
        let logits = [0.1f32, 9.0, 0.4, 0.2];
        for seed in 0u32..64 {
            let mut rng = crate::pack_mulberry(seed);
            assert_eq!(sample(&logits, 1e-4, &mut rng), 1);
        }
    }

    #[test]
    fn uniform_logits_reach_every_token() {
        let logits = [0.0f32; 16];
        let mut seen = std::collections::HashSet::new();
        let mut rng = crate::pack_mulberry(7);
        for _ in 0..256 {
            let t = sample(&logits, 1.0, &mut rng);
            seen.insert(t);
            assert!(t < 16);
        }
        assert_eq!(seen.len(), 16, "every token must be reachable");
    }

    #[test]
    fn sampling_is_seed_deterministic() {
        let logits = [0.1f32, 1.0, 2.0, 3.0, 0.5];
        let mut a = crate::pack_mulberry(42);
        let mut b = crate::pack_mulberry(42);
        let seq_a: Vec<usize> = (0..50).map(|_| sample(&logits, 0.8, &mut a)).collect();
        let seq_b: Vec<usize> = (0..50).map(|_| sample(&logits, 0.8, &mut b)).collect();
        assert_eq!(seq_a, seq_b);
    }
}
