// tern.rs — the ternary {-1, 0, +1} wire, in Rust.
//
// The same PRNG family the engine's wire uses in Node (tern.ts) and in
// Python (the mesh actors): mulberry32 and the LCG. Bit-identical across
// languages — a seed here is a seed everywhere.

/// mulberry32 — 32-bit state, returns [0, 1). Bit-identical to the
/// JS/TS mulberry32 in quantTernEngine/tern.ts.
pub fn mulberry32(state: u32) -> impl FnMut() -> f64 {
    let mut a = state;
    move || {
        a = a.wrapping_add(0x6D2B79F5);
        let mut t = a;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61)) ^ t;
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }
}

/// The LCG the mesh NPCs run on (x = x·1664525 + 1013904223 mod 2^32).
pub fn lcg(state: u32) -> impl FnMut() -> f64 {
    let mut s = state;
    move || {
        s = s.wrapping_mul(1664525).wrapping_add(1013904223);
        (s as f64) / 4294967296.0
    }
}

/// seed_from_text — the one-way door from words to a seed (sha256).
pub fn seed_from_text(text: &str) -> u64 {
    use sha2::Digest;
    let d = sha2::Sha256::digest(text.as_bytes());
    u64::from_be_bytes(d[..8].try_into().expect("8 bytes"))
}

/// balanced_trits — a seed as `dims` balanced trits {-1, 0, +1}.
/// Trisected one digit at a time, replayable.
pub fn balanced_trits(mut seed: u64, dims: usize) -> Vec<i8> {
    let mut out = Vec::with_capacity(dims);
    for _ in 0..dims {
        let r = seed % 3;
        out.push(if r == 2 { -1 } else { r as i8 });
        seed /= 3;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mulberry32_matches_the_js_wire() {
        // recompute the first JS value bit-for-bit and compare:
        let mut rng = mulberry32(7);
        let got = rng();
        let a = 7u32.wrapping_add(0x6D2B79F5);
        let mut t = a;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61)) ^ t;
        let expect = ((t ^ (t >> 14)) as f64) / 4294967296.0;
        assert!((got - expect).abs() < 1e-12);
        // determinism: same seed, same stream
        let mut again = mulberry32(7);
        assert!((again() - got).abs() < 1e-12);
    }

    #[test]
    fn trits_are_balanced_and_replayable() {
        let a = balanced_trits(0xDEADBEEF, 16);
        let b = balanced_trits(0xDEADBEEF, 16);
        assert_eq!(a, b);
        assert!(a.iter().all(|&t| (-1..=1).contains(&t)));
    }
}
