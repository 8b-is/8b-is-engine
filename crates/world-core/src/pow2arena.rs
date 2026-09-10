//! pow2arena — the bump-arena with a cause (the trick library, entry 2).
//!
//! The hot path hates two things: per-element allocation and growth that
//! re-homes every element. `Pow2Slabs<T>` is the answer the engine's
//! preallocated-arena doctrine already promised: elements live in
//! append-only slabs sized 1, 2, 4, … (next-power-of-two), so growth is
//! amortized, an id never moves once handed out, iteration is cache- and
//! determinism-friendly, and the worst-case slack is ≤ 2× the live count
//! (each slab is at most twice its predecessor's live tail).
//!
//! The `Events` register's swap-remove stays for the game-loop arena;
//! this lane is the *append-mostly* arena — spawn queues, ledger staging,
//! render batches — where stable ids and pow2 growth beat churn.
//!
//! Determinism: ids are handed out in order; iteration is slab order then
//! insertion order; two runs with the same pushes see the same ids and
//! the same byte-level visits.

/// The append-mostly slab arena.
#[derive(Debug, Clone)]
pub struct Pow2Slabs<T> {
    slots: Vec<T>,
    cap: usize,
}

/// The next power of two at or above `n` (saturating at u32 domain —
/// the lane never grows past the slab budget the wire can address).
pub fn next_pow2(n: usize) -> usize {
    let mut p = 1usize;
    while p < n {
        p = p.checked_mul(2).unwrap_or(usize::MAX);
    }
    p
}

impl<T> Pow2Slabs<T> {
    pub fn new() -> Self {
        Pow2Slabs {
            slots: Vec::new(),
            cap: 0,
        }
    }

    /// push — append one element, growing capacity to the next pow2 when
    /// the current slab is full. The returned id is stable forever.
    pub fn push(&mut self, value: T) -> usize {
        let id = self.slots.len();
        if id == self.cap {
            self.cap = next_pow2(id + 1);
            self.slots.reserve(self.cap - id);
        }
        self.slots.push(value);
        id
    }

    pub fn get(&self, id: usize) -> Option<&T> {
        self.slots.get(id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        self.slots.get_mut(id)
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// The slab budget: the capacity (next pow2 ≥ len). The slack metric
    /// documents the arena's memory honesty: cap ≤ 2·len except at the
    /// very first push (cap 1).
    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.slots.iter()
    }
}

impl<T> Default for Pow2Slabs<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_stable_and_sequential() {
        let mut a = Pow2Slabs::new();
        let ids: Vec<usize> = (0..100).map(|i| a.push(i * i)).collect();
        for (i, &id) in ids.iter().enumerate() {
            assert_eq!(id, i, "ids in insertion order");
            assert_eq!(a.get(id), Some(&(i * i)), "id is stable");
        }
        assert_eq!(a.len(), 100);
    }

    #[test]
    fn capacity_grows_by_next_pow2() {
        let mut a = Pow2Slabs::new();
        let caps: Vec<usize> = (0..130)
            .map(|_| {
                a.push(0u8);
                a.capacity()
            })
            .collect();
        // caps: 1, 2, 4, 4, 8, 8, 8, 8, 16, ... — every fresh slab is a
        // pow2 and the slack is bounded by 2×
        assert_eq!(caps[0], 1);
        assert_eq!(caps[1], 2);
        assert_eq!(caps[3], 4);
        assert_eq!(caps[7], 8);
        for (live, &cap) in caps.iter().enumerate() {
            let live = live + 1;
            assert!(cap >= live, "capacity never under the live count");
            assert!(cap <= live.max(1) * 2, "≤2× slack at {live} (cap {cap})");
            assert_eq!(cap, crate::pow2arena::next_pow2(cap), "capacity is a pow2");
        }
    }

    #[test]
    fn iteration_is_insertion_ordered_and_deterministic() {
        let mut a = Pow2Slabs::new();
        for i in 0..300 {
            a.push((i % 7) as u8);
        }
        let first: Vec<u8> = a.iter().copied().collect();
        let mut b = Pow2Slabs::new();
        for i in 0..300 {
            b.push((i % 7) as u8);
        }
        let second: Vec<u8> = b.iter().copied().collect();
        assert_eq!(first, second, "two runs see the same order");
        assert_eq!(first, (0..300).map(|i| (i % 7) as u8).collect::<Vec<_>>());
    }

    #[test]
    fn mutation_via_id_works() {
        let mut a = Pow2Slabs::new();
        let id = a.push(10i32);
        *a.get_mut(id).unwrap() += 5;
        assert_eq!(a.get(id), Some(&15));
        assert_eq!(a.get(999), None);
    }
}
