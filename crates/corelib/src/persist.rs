//! persist — the engram vault: a virtual address space over
//! content-addressed, de-duplicated blocks, written by exactly one
//! writer, decoded lazily and cached per session.
//!
//! * **the virtual address space** — every engram has an address
//!   `{seg}.{off}.{len}`: segment + offset + length, stable forever,
//!   independent of content (append-only).
//! * **de-dupe blocks** — a block IS its content hash; appending an
//!   engram whose blob is already seated returns the SEATED address and
//!   writes nothing. The vault grows by unique memories only.
//! * **single writer** — `append(&mut self, …)`: the borrow-checker is
//!   the writer's door. There is no way to write through a shared
//!   reference; concurrent readers never race the one author.
//! * **the lazy cache** — `LazyCache` decodes on first `get` and
//!   memoizes; sessions flush it wholesale (`forget`) — the junk of a
//!   session dies with the session, the vault keeps only what was
//!   SENT.

use qdecorators::{attest, stacked_decode, stacked_encode, Engram};
use std::collections::HashMap;

/// A virtual address: segment, offset, length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Addr {
    pub seg: u32,
    pub off: u32,
    pub len: u32,
}

impl Addr {
    pub fn display(self) -> String {
        format!("seg:{}/off:{}", self.seg, self.off)
    }
}

struct Seated {
    // the address alias: seated blocks remember where they live
    #[allow(dead_code)]
    addr: Addr,
    blob: Vec<u8>, // the STACCED blob (L3) — decode on demand
}

/// The single-writer vault.
pub struct EngramVault {
    segments: Vec<Vec<Seated>>,
    /// content hash → the seated address (de-dupe)
    by_hash: HashMap<String, Addr>,
    /// content hash → the existing block's tail (for aliasing reads)
    segment_cap: usize,
    unique: u64,
    writes: u64,
}

impl EngramVault {
    pub fn new(segment_cap: usize) -> Self {
        EngramVault {
            segments: vec![Vec::new()],
            by_hash: HashMap::new(),
            segment_cap: segment_cap.max(1),
            unique: 0,
            writes: 0,
        }
    }

    /// append — the single writer's door: `&mut self` or nothing. If
    /// the engram's block is already seated, the seated address returns
    /// and NOTHING is written (de-dupe).
    pub fn append(&mut self, e: &Engram) -> Addr {
        let blob = stacked_encode(std::slice::from_ref(e));
        let hash = attest(&blob);
        if let Some(&addr) = self.by_hash.get(&hash) {
            return addr; // the block already lives here
        }
        let seg = (self.segments.len() - 1) as u32;
        let off = self.segments[seg as usize].len() as u32;
        let addr = Addr {
            seg,
            off,
            len: blob.len() as u32,
        };
        self.segments[seg as usize].push(Seated { addr, blob });
        self.by_hash.insert(hash, addr);
        self.unique += 1;
        self.writes += 1;
        // segment split: a full segment closes, a fresh one opens
        if self.segments[seg as usize].len() >= self.segment_cap {
            self.segments.push(Vec::new());
        }
        addr
    }

    /// read — lazy decode of the seated blob (the cache in
    /// `LazyCache` memoizes this).
    pub fn read(&self, addr: Addr) -> Option<Engram> {
        let seg = self.segments.get(addr.seg as usize)?;
        let seated = seg.get(addr.off as usize)?;
        let stack = stacked_decode(&seated.blob).ok()?;
        stack.into_iter().next()
    }

    /// The vault's guarantees, counted.
    pub fn stats(&self) -> (u64, u64) {
        (self.unique, self.writes)
    }

    /// A virtual address's display line, for the wire.
    pub fn describe(&self, addr: Addr) -> Option<String> {
        self.read(addr).map(|e| {
            format!(
                "{} · {:?} · {} · ✓{}",
                addr.display(),
                e.kind,
                e.text.chars().take(24).collect::<String>(),
                e.tick
            )
        })
    }
}

/// The virtual cache: decode on first get, memoize, flush per session.
pub struct LazyCache {
    memo: HashMap<Addr, Engram>,
    reads_from_log: u64,
    cache_hits: u64,
}

impl LazyCache {
    pub fn new_session() -> Self {
        LazyCache {
            memo: HashMap::new(),
            reads_from_log: 0,
            cache_hits: 0,
        }
    }

    /// get — lazy: the first ask decodes from the vault, the rest hit
    /// the memo.
    pub fn get(&mut self, vault: &EngramVault, addr: Addr) -> Option<&Engram> {
        if !self.memo.contains_key(&addr) {
            if let Some(e) = vault.read(addr) {
                self.memo.insert(addr, e);
                self.reads_from_log += 1;
            } else {
                return None;
            }
        } else {
            self.cache_hits += 1;
        }
        self.memo.get(&addr)
    }

    /// forget — the session's junk dies with the session: the memo
    /// empties and the session-scoped counters reset.
    pub fn forget(&mut self) {
        self.memo.clear();
        self.reads_from_log = 0;
        self.cache_hits = 0;
    }

    /// memo_len — how many memories sit in the session right now.
    pub fn memo_len(&self) -> usize {
        self.memo.len()
    }

    pub fn stats(&self) -> (u64, u64) {
        (self.reads_from_log, self.cache_hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(text: &str, tick: u64) -> Engram {
        let mut e = Engram::new(text);
        e.tick = tick;
        e
    }

    #[test]
    fn the_single_writer_appends_and_reads() {
        let mut vault = EngramVault::new(4);
        let a = vault.append(&e("one", 1));
        let b = vault.append(&e("two", 2));
        assert_eq!(vault.read(a).unwrap().text, "one");
        assert_eq!(vault.read(b).unwrap().text, "two");
        assert_eq!(vault.stats(), (2, 2));
    }

    #[test]
    fn duplicate_blocks_are_seated_once() {
        let mut vault = EngramVault::new(4);
        let first = vault.append(&e("the same memory", 3));
        let again = vault.append(&e("the same memory", 3));
        assert_eq!(first, again, "de-dupe: the block already lives");
        assert_eq!(vault.stats().0, 1, "one unique block");
        assert_eq!(vault.stats().1, 1, "one write, not two");
    }

    #[test]
    fn the_virtual_address_space_spans_segments() {
        let mut vault = EngramVault::new(3); // tiny segments
        let mut addrs = Vec::new();
        for i in 0..8 {
            addrs.push(vault.append(&e(&format!("m{}", i), i as u64)));
        }
        // exactly one segment split happened (3 + 3 + 2)
        let segs: Vec<u32> = addrs.iter().map(|a| a.seg).collect();
        assert!(segs.contains(&0) && segs.contains(&1));
        assert_eq!(vault.read(addrs[7]).unwrap().text, "m7");
        assert_eq!(vault.read(addrs[0]).unwrap().text, "m0");
        // cross-segment reads stay stable
        let last = *addrs.last().unwrap();
        assert_eq!(vault.read(last).unwrap().tick, 7);
    }

    #[test]
    fn the_lazy_cache_decodes_once_and_flushes() {
        let mut vault = EngramVault::new(8);
        let addr = vault.append(&e("cached", 9));
        let mut cache = LazyCache::new_session();
        assert_eq!(cache.get(&vault, addr).unwrap().text, "cached");
        let (reads, hits) = cache.stats();
        assert_eq!(reads, 1);
        assert_eq!(cache.get(&vault, addr).unwrap().text, "cached");
        let (reads2, hits2) = cache.stats();
        assert_eq!(reads2, 1, "the second ask never touched the log");
        assert_eq!(hits2, hits + 1);
        cache.forget();
        assert_eq!(cache.memo_len(), 0, "the memo is empty after the session");
        let (reads3, _) = cache.stats();
        assert_eq!(reads3, 0, "the session counters reset with the session");
    }

    #[test]
    fn a_missing_address_reads_nothing() {
        let vault = EngramVault::new(4);
        assert!(vault
            .read(Addr {
                seg: 0,
                off: 99,
                len: 1
            })
            .is_none());
    }
}
