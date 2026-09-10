//! pack — the tri-state { -1, 0, +1 } storage lane.
//!
//! Four trits per byte, two bits each, little-endian field order: the
//! first trit occupies the low two bits. Encoding: `0b00 → 0`,
//! `0b01 → +1`, `0b10 → -1`, `0b11 → reserved` (never written by the
//! packer; `unpack_strict` refuses it, `unpack_lenient` maps it to 0).
//!
//! This is the engine's answer to "1.58 bits per weight" made file-real:
//! 2 bits per weight, 4 weights per byte, a 16-million-parameter model in
//! 4 MiB — a whole world in a pocket.

/// Tri-states per byte.
pub const PACK_PER_BYTE: usize = 4;

/// The number of bytes needed for `n` trits.
pub fn packed_len(n: usize) -> usize {
    n.div_ceil(PACK_PER_BYTE)
}

/// Pack tri-states into the 2-bit lane. Every value must be in
/// `{-1, 0, +1}` (the ternary wire's alphabet); anything else panics —
/// the lane never writes a silent `0b11`.
pub fn pack_trits(trits: &[i8]) -> Vec<u8> {
    let mut out = vec![0u8; packed_len(trits.len())];
    for (i, &t) in trits.iter().enumerate() {
        let field = match t {
            1 => 0b01u8,
            -1 => 0b10u8,
            0 => 0b00u8,
            other => panic!("pack: {other} is not a tri-state"),
        };
        out[i / PACK_PER_BYTE] |= field << (2 * (i % PACK_PER_BYTE));
    }
    out
}

fn field(byte: u8, i: usize) -> Option<i8> {
    match (byte >> (2 * i)) & 0b11 {
        0b00 => Some(0),
        0b01 => Some(1),
        0b10 => Some(-1),
        _ => None, // the reserved state: strict refuses, lenient zeroes
    }
}

/// Unpack, refusing the reserved `0b11` field: a corrupted lane is a
/// corrupted world, and corruption is reported, not absorbed.
pub fn unpack_strict(bytes: &[u8], n: usize) -> Result<Vec<i8>, String> {
    if packed_len(n) > bytes.len() {
        return Err(format!(
            "unpack: need {} bytes, have {}",
            packed_len(n),
            bytes.len()
        ));
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let byte = bytes[i / PACK_PER_BYTE];
        match field(byte, i % PACK_PER_BYTE) {
            Some(t) => out.push(t),
            None => return Err(format!("unpack: reserved 0b11 field at trit {i}")),
        }
    }
    Ok(out)
}

/// Unpack, leniently mapping the reserved `0b11` to 0 (safe-mode loaders,
/// forward-compat with encodings that fill the spare state).
pub fn unpack_lenient(bytes: &[u8], n: usize) -> Vec<i8> {
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let byte = bytes[i / PACK_PER_BYTE];
        out.push(field(byte, i % PACK_PER_BYTE).unwrap_or(0));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhaustive_round_trip_all_81_nibbles() {
        // Four tri-states per byte: 3^4 = 81 distinct combinations,
        // every one must survive the round trip byte-exactly.
        let states = [-1i8, 0, 1];
        let mut seen = std::collections::HashSet::new();
        for a in states {
            for b in states {
                for c in states {
                    for d in states {
                        let trits = [a, b, c, d];
                        let packed = pack_trits(&trits);
                        assert_eq!(packed.len(), 1);
                        let back = unpack_strict(&packed, 4).unwrap();
                        assert_eq!(back, trits);
                        seen.insert(packed[0]);
                    }
                }
            }
        }
        assert_eq!(seen.len(), 81, "all 81 nibbles must be distinct");
    }

    #[test]
    fn density_four_trits_per_byte() {
        assert_eq!(packed_len(0), 0);
        assert_eq!(packed_len(1), 1);
        assert_eq!(packed_len(4), 1);
        assert_eq!(packed_len(5), 2);
        assert_eq!(packed_len(1024), 256);
        // The lane's headline: a 16-million-parameter model in 4 MiB.
        assert_eq!(packed_len(16_000_000), 4_000_000);
    }

    #[test]
    fn reserved_field_is_refused_strict_and_lenient_zeroed() {
        let bad = [0b11111111u8];
        assert!(unpack_strict(&bad, 4).is_err());
        assert_eq!(unpack_lenient(&bad, 4), vec![0, 0, 0, 0]);
    }

    #[test]
    fn short_buffer_is_refused() {
        assert!(unpack_strict(&[0u8; 1], 5).is_err());
    }

    #[test]
    fn packing_is_deterministic() {
        let mut trits = Vec::new();
        for i in 0..1000 {
            trits.push(if i % 3 == 0 {
                1
            } else if i % 3 == 1 {
                -1
            } else {
                0
            });
        }
        assert_eq!(pack_trits(&trits), pack_trits(&trits));
    }
}
