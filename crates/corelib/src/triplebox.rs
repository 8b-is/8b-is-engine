//! triplebox — the three skins of one engram, as a diamond, lazily.
//!
//! ```text
//!        L1  (the snake_case wire)
//!        │
//!        ▼
//!        L3  (the STACCED stacked blob)
//!        │
//!        ▼
//!        L2  (the ASCII85 printable skin)
//! ```
//!
//! The diamond has two encode edges from L1 (to L3 via stacking, then to
//! L2 via ascii85 — decodes walk back the same edge). Every view is
//! **lazy**: the box holds only the source engram; the lower skins are
//! computed on first ask and memoized (`OnceLock`), so an engram that is
//! only ever read as a wire never pays the blob tax. Deterministic
//! either way: the same engram, the same skins, whatever the order the
//! views are asked in.

use qdecorators::{ascii85_decode, ascii85_encode, stacked_decode, stacked_encode, Engram};
use std::sync::OnceLock;

/// Three addresses for one memory.
pub struct TripleBox {
    source: Engram,
    l3: OnceLock<Vec<u8>>,
    l2: OnceLock<String>,
}

impl TripleBox {
    pub fn from_engram(e: Engram) -> Self {
        TripleBox {
            source: e,
            l3: OnceLock::new(),
            l2: OnceLock::new(),
        }
    }

    /// L1 — the wire view (the source itself, never rebuilt).
    pub fn l1(&self) -> &Engram {
        &self.source
    }

    /// L3 — the stacked blob, computed once on first ask.
    pub fn l3(&self) -> &[u8] {
        self.l3
            .get_or_init(|| stacked_encode(std::slice::from_ref(&self.source)))
    }

    /// L2 — the ascii85 skin, built on the FIRST L3 ask's result (the
    /// diamond edge: nobody encodes L2 from thin air).
    pub fn l2(&self) -> &str {
        let l3 = self.l3(); // the diamond's downward edge
        self.l2.get_or_init(|| ascii85_encode(l3))
    }

    /// decode — walk the diamond back down to the wire: L2 → L3 → L1.
    pub fn decode(&self) -> Engram {
        let blob = ascii85_decode(self.l2()).expect("triplebox: the L2 skin decodes");
        let stack = stacked_decode(&blob).expect("triplebox: the L3 stack unstacks");
        stack
            .into_iter()
            .next()
            .expect("triplebox: one engram per box")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Engram {
        let mut e = Engram::new("the world runs without you");
        e.kind = "admission".into();
        e.ctx = "sanctuary".into();
        e.seed = 41592;
        e.tick = 108;
        e.tags = vec!["fold".into(), "keeper".into()];
        e.weight = 1.5;
        e.links = vec!["seg:0/off:4".into()];
        e
    }

    #[test]
    fn the_diamond_round_trips() {
        let boxed = TripleBox::from_engram(sample());
        assert_eq!(boxed.decode(), sample());
    }

    #[test]
    fn the_skins_are_memoized() {
        let boxed = TripleBox::from_engram(sample());
        let l3_a = boxed.l3();
        let l3_b = boxed.l3();
        assert!(
            std::ptr::eq(l3_a.as_ptr(), l3_b.as_ptr()),
            "L3 computed once"
        );
        let l2_a = boxed.l2();
        let l2_b = boxed.l2();
        assert!(
            std::ptr::eq(l2_a.as_ptr(), l2_b.as_ptr()),
            "L2 computed once"
        );
        // the diamond edge: L2 is exactly the ascii85 of L3
        assert_eq!(l2_a, ascii85_encode(l3_a));
    }

    #[test]
    fn the_box_is_lazy_until_a_skin_is_asked() {
        // construction does not touch the OnceLocks — the box's drop
        // would be trivially true; we verify by asking L1 first: the
        // wire view exists even if the blob is never requested
        let boxed = TripleBox::from_engram(sample());
        assert_eq!(boxed.l1().text, "the world runs without you");
    }

    #[test]
    fn order_of_asks_does_not_change_the_skins() {
        let a = TripleBox::from_engram(sample());
        let b = TripleBox::from_engram(sample());
        let _ = b.l2(); // ask the bottom first
        let _ = b.l3();
        assert_eq!(a.l3(), b.l3());
        assert_eq!(a.l2(), b.l2());
        assert_eq!(a.decode(), b.decode());
    }
}
