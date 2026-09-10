//! io — attested I/O abstractions: the engine's framing and checksum
//! discipline, reusable outside the engine.

use sha2::{Digest, Sha256};
use std::io::{Read, Write};

/// The wire's length prefix: 4 bytes, big-endian — the same framing the
/// mesh-node, the relay's doors, and the LSP-style Content-Length share.
/// One frame, one truth about its size, everywhere.
pub fn frame_len_prefix(len: usize) -> [u8; 4] {
    (len as u32).to_be_bytes()
}

/// read_len_frame — read one length-framed record from a reader: the
/// 4-byte BE prefix, then exactly that many bytes. The inverse of
/// `write_len_frame`; a truncated frame is an error, never a guess.
pub fn read_len_frame<R: Read>(r: &mut R) -> std::io::Result<Vec<u8>> {
    let mut prefix = [0u8; 4];
    r.read_exact(&mut prefix)?;
    let len = u32::from_be_bytes(prefix) as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

/// write_len_frame — one length-framed record into a writer.
pub fn write_len_frame<W: Write>(w: &mut W, data: &[u8]) -> std::io::Result<()> {
    w.write_all(&frame_len_prefix(data.len()))?;
    w.write_all(data)?;
    w.flush()
}

/// attest — the SHA-256 of a byte slice, hex-encoded: the record's
/// fingerprint, the same checksum the lane's stager pins.
pub fn attest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// The attestation of a framed stream: hash the (length-prefixed) frames
/// as they fall — the ledger's repetitive wire, fingerprinted as it is
/// written, not after the fact.
pub fn attest_frames<I: IntoIterator<Item = Vec<u8>>>(frames: I) -> String {
    let mut hasher = Sha256::new();
    for f in frames {
        hasher.update(frame_len_prefix(f.len()));
        hasher.update(&f);
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framing_round_trips_and_truncation_is_refused() {
        let mut buf = Vec::new();
        let payload = b"{\"t\":1,\"s\":7,\"n\":{\"h\":0.5}}".to_vec();
        write_len_frame(&mut buf, &payload).unwrap();
        write_len_frame(&mut buf, b"second frame").unwrap();
        let mut cur = &buf[..];
        assert_eq!(read_len_frame(&mut cur).unwrap(), payload);
        assert_eq!(read_len_frame(&mut cur).unwrap(), b"second frame".to_vec());
        // a sad truncated frame must error, not half-return: the first
        // frame is intact, the second is cut mid-payload
        let mut short = &buf[..buf.len() - 3];
        assert_eq!(read_len_frame(&mut short).unwrap(), payload);
        assert!(read_len_frame(&mut short).is_err());
    }

    #[test]
    fn the_prefix_is_little_network_endian_and_four_bytes() {
        assert_eq!(frame_len_prefix(1), [0, 0, 0, 1]);
        assert_eq!(frame_len_prefix(0x01020304), [1, 2, 3, 4]);
    }

    #[test]
    fn attestation_is_stable_and_sensitive() {
        let a = attest(b"the world runs without you");
        let b = attest(b"the world runs without you.");
        assert_eq!(a.len(), 64);
        assert_ne!(a, b, "a trailing dot must be heard");
        assert_eq!(a, attest(b"the world runs without you"));
    }

    #[test]
    fn framed_attestation_depends_on_order() {
        let a = attest_frames([vec![1, 2], vec![3]]);
        let b = attest_frames([vec![1], vec![2, 3]]);
        assert_ne!(a, b);
    }
}
