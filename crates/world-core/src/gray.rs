//! gray — gray-code deltas for the hot path (the trick library, entry 1).
//!
//! The engine's wire moves the world's deltas every tick. Adjacent ticks
//! usually move by one step — a creature steps left, hunger rises by one
//! — and the binary-reflected Gray code is the encoding that honors that:
//! `gray_encode` maps a value so that **consecutive values differ in
//! exactly one bit**. A delta of ±1 becomes a single-bit toggle instead
//! of a multi-bit scramble, which is exactly what a compact ternary wire
//! wants: the rare bit, not the noisy byte.
//!
//! The lane is pure and deterministic: `gray_decode(gray_encode(v)) == v`
//! for every 32-bit value, and the adjacent-step property is tested
//! exhaustively over a window.

/// binary-reflected gray encode
pub fn gray_encode(v: u32) -> u32 {
    v ^ (v >> 1)
}

/// binary-reflected gray decode (the inverse of `gray_encode`)
pub fn gray_decode(g: u32) -> u32 {
    let mut v = g;
    v ^= v >> 16;
    v ^= v >> 8;
    v ^= v >> 4;
    v ^= v >> 2;
    v ^= v >> 1;
    v
}

/// Bits that changed between two gray codes — the hot path's cost meter:
/// `+1` steps toggle exactly 1 bit, so a well-behaved delta costs one
/// changed bit; a scramble shows its face as a wide spread.
pub fn changed_bits(a: u32, b: u32) -> u32 {
    (a ^ b).count_ones()
}

/// The delta wire for a sequence: consecutive gray codes of the values
/// (the receiver decodes with `gray_decode`). Transporting gray codes
/// costs one bit per ±1 step — the rare bit, not the noisy byte.
pub fn to_gray_delta(values: &[u32]) -> Vec<u32> {
    values.iter().map(|&v| gray_encode(v)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_is_identity_over_a_wide_window() {
        // a 24-bit sweep plus the u32 extremes: the decode must invert
        // the encode, exactly, everywhere
        for v in 0u32..(1 << 24) {
            assert_eq!(gray_decode(gray_encode(v)), v, "round trip at {v}");
        }
        for extreme in [u32::MAX, 1u32 << 31, 0xFFFF_FFFF - 1] {
            assert_eq!(gray_decode(gray_encode(extreme)), extreme);
        }
    }

    #[test]
    fn adjacent_steps_toggle_exactly_one_bit() {
        // the trick's promise: +1 → one changed bit, in both directions,
        // across the whole window (no wrap boundary exceptions in range)
        for v in 0u32..(1 << 24) {
            assert_eq!(
                changed_bits(gray_encode(v), gray_encode(v + 1)),
                1,
                "±1 step must toggle 1 bit at {v}"
            );
        }
    }

    #[test]
    fn wide_steps_show_their_face() {
        // the meter is honest: a K-step changes at most 2·⌈log2 K⌉ bits
        // (the value's top carry + its half's carry), so a 3-step costs
        // ≤ 4 bits and a 257-step costs ≤ 18 — a wide step cannot hide
        // in the one-bit lane
        for v in (0u32..(1 << 20)).step_by(17) {
            for k in [3u32, 17, 257] {
                let bits = changed_bits(gray_encode(v), gray_encode(v + k));
                let bound = 2 * (32 - (k - 1).leading_zeros());
                assert!(
                    (1..=bound).contains(&bits),
                    "step {k} at {v}: {bits} bits (bound {bound})"
                );
            }
        }
    }

    #[test]
    fn the_delta_wire_round_trips() {
        let values = [0u32, 3, 4, 9, 10, (1 << 24), u32::MAX];
        let wire = to_gray_delta(&values);
        let back: Vec<u32> = wire.iter().map(|&g| gray_decode(g)).collect();
        assert_eq!(back, values);
        // adjacent wire values differ in exactly one bit where the world
        // stepped by one
        assert_eq!(changed_bits(wire[0], wire[1]), 1);
        assert_eq!(changed_bits(wire[1], wire[2]), 1);
        assert_eq!(changed_bits(wire[3], wire[4]), 1);
    }
}
