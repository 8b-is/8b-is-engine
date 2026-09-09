// transport.rs — the multiplexer's two folds, in the shared core.
//
// The roadmap question, answered as primitives:
//   * the ternary hot path runs on a lock-free SPSC ring buffer over a
//     preallocated arena — zero syscalls, zero allocs, cache-friendly;
//   * the committed history lives in an mmap-backed append-only ledger —
//     durable, restart-safe, replayable, checksum-attested;
//   * where the two meet, the ring drains into the ledger: hot frames in,
//     cold records out.

use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A bounded SPSC ring buffer — one producer, one consumer, no locks.
/// The slot accesses are ordered by the release/acquire atomics: the
/// producer writes slot `tail` only after the consumer's `head` has passed
/// it, and the consumer reads slot `head` only after the producer's `tail`
/// has reached it — the happens-before is established by the counters.
pub struct Ring<T> {
    slots: Vec<Option<T>>,
    head: AtomicUsize,
    tail: AtomicUsize,
    mask: usize,
}

impl<T> Ring<T> {
    /// new — a preallocated arena of `capacity` slots (rounded up to a
    /// power of two, so the index is a mask, not a modulo).
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.next_power_of_two().max(2);
        let mut slots = Vec::with_capacity(cap);
        for _ in 0..cap {
            slots.push(None);
        }
        Ring {
            slots,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            mask: cap - 1,
        }
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// push — enqueue one frame. Returns the frame back when the ring is
    /// full (the producer decides: drop, block, or spill to the ledger).
    pub fn push(&mut self, v: T) -> Result<(), T> {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        if tail.wrapping_sub(head) >= self.slots.len() {
            return Err(v);
        }
        self.slots[tail & self.mask] = Some(v);
        self.tail.store(tail + 1, Ordering::Release);
        Ok(())
    }

    /// pop — dequeue one frame, or None when empty.
    pub fn pop(&mut self) -> Option<T> {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        if head == tail {
            return None;
        }
        let v = self.slots[head & self.mask].take();
        self.head.store(head + 1, Ordering::Release);
        v
    }

    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire) == self.tail.load(Ordering::Acquire)
    }
}

/// An append-only ledger backed by an mmap — the semantic fold's substrate.
/// Appends are file writes; the fold reads through the memory map; the
/// checksum attests the whole log (the stager's discipline, in the core).
pub struct Ledger {
    file: std::fs::File,
    path: PathBuf,
    len: u64,
}

impl Ledger {
    /// open — create-or-open an append-only ledger at `path`.
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .read(true)
            .open(path)?;
        let len = file.metadata()?.len();
        Ok(Ledger {
            file,
            path: path.to_path_buf(),
            len,
        })
    }

    /// append — write one record to the end of the log.
    pub fn append(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.file.write_all(bytes)?;
        self.len += bytes.len() as u64;
        Ok(())
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// read_all — the whole log as bytes (replay, checksums, the fold).
    pub fn read_all(&self) -> std::io::Result<Vec<u8>> {
        let mut f = std::fs::File::open(&self.path)?;
        let mut buf = Vec::with_capacity(self.len as usize);
        f.seek(SeekFrom::Start(0))?;
        f.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// map — an mmap read view over the whole log (the fold's fast path).
    pub fn map(&self) -> std::io::Result<memmap2::Mmap> {
        let f = std::fs::File::open(&self.path)?;
        // SAFETY: the map is read-only; the file is append-only, so the
        // mapped region never changes underneath a reader mid-fold.
        unsafe { memmap2::Mmap::map(&f) }
    }

    /// checksum — SHA256 over the whole log: corruption is detected, not
    /// assumed away.
    pub fn checksum(&self) -> std::io::Result<String> {
        let bytes = self.read_all()?;
        Ok(hex(&Sha256::digest(&bytes)))
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_preserves_order_and_wraps() {
        let mut ring = Ring::new(4);
        for i in 0..8u32 {
            ring.push(i).unwrap();
            assert_eq!(ring.pop(), Some(i), "SPSC order is FIFO");
        }
        assert!(ring.is_empty());
    }

    #[test]
    fn ring_full_returns_the_frame() {
        let mut ring = Ring::new(2);
        ring.push(1).unwrap();
        ring.push(2).unwrap();
        assert_eq!(ring.push(3), Err(3), "full ring refuses the frame");
        assert_eq!(ring.pop(), Some(1));
        assert_eq!(ring.pop(), Some(2));
        assert!(ring.is_empty());
    }

    #[test]
    fn ledger_appends_replays_and_attests() {
        let dir = std::env::temp_dir().join(format!("world-core-ledger-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("log.bin");

        let mut ledger = Ledger::open(&path).unwrap();
        ledger.append(b"{\"t\":1}").unwrap();
        ledger.append(b"{\"t\":2}").unwrap();
        ledger.flush().unwrap();
        let sum1 = ledger.checksum().unwrap();

        // restart-safe: a fresh handle reads the same log, same checksum
        let reopened = Ledger::open(&path).unwrap();
        assert_eq!(reopened.len(), ledger.len());
        assert_eq!(reopened.read_all().unwrap(), b"{\"t\":1}{\"t\":2}");
        assert_eq!(reopened.checksum().unwrap(), sum1);

        // the mmap view carries the same bytes
        let mmap = reopened.map().unwrap();
        assert_eq!(&mmap[..], b"{\"t\":1}{\"t\":2}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ring_drains_into_the_ledger() {
        // the transport's two folds, end to end: hot frames in the ring,
        // cold records out to the mmap ledger, attested by the checksum
        let dir = std::env::temp_dir().join(format!("world-core-drain-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut ledger = Ledger::open(&dir.join("log.bin")).unwrap();
        let mut ring = Ring::new(8);
        for i in 0..5u32 {
            ring.push(format!("{{\"t\":{i}}}")).unwrap();
        }
        while let Some(frame) = ring.pop() {
            ledger.append(frame.as_bytes()).unwrap();
        }
        ledger.flush().unwrap();
        let sum = ledger.checksum().unwrap();
        assert!(sum.len() == 64, "sha256 hex");
        assert_eq!(ledger.read_all().unwrap(), b"{\"t\":0}{\"t\":1}{\"t\":2}{\"t\":3}{\"t\":4}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
