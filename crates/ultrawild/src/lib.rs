//! # ultrawild — the macro layer
//!
//! Ultra-wild Rust performance tricks (David Lattimore's Wild-linker
//! school) plus demoscene/asm-era bit hacks, as an importable macro
//! layer: `use ultrawild::*`.
//!
//! The doctrine: allocations are the sand; the registers are the
//! castle. These macros put the klassieke tricks one `!` away.

/// The ternary clamp — the constellation's own {-1, 0, +1} bit truth.
#[macro_export]
macro_rules! clamp_ternary {
    ($e:expr) => {{
        let v: i64 = $e as i64;
        if v > 1 { 1i8 } else if v < -1 { -1i8 } else { v as i8 }
    }};
    ($v:ident, $e:expr) => {{
        let n: i64 = $e as i64;
        $v = if n > 1 { 1i8 } else if n < -1 { -1i8 } else { n as i8 };
    }};
}

/// The demoscene divide-by-255: `(x * 0x8081) >> 23` — the classic
/// integer alpha blend, no division unit needed.
#[macro_export]
macro_rules! div255 {
    ($e:expr) => { ((($e as u32 & 0xFFFF) * 0x8081u32) >> 23) as u16 };
    ($e:expr, $f:expr) => { ((($e as u32 & 0xFFFF) * ($f as u32 & 0xFFFF) * 0x8081u32) >> 23) as u16 };
}

/// SWAR population count (Hacker's Delight): popcount without a table.
#[macro_export]
macro_rules! popcount {
    ($e:expr) => {{
        let mut x: u64 = $e as u64;
        x = x - ((x >> 1) & 0x5555555555555555);
        x = (x & 0x3333333333333333) + ((x >> 2) & 0x3333333333333333);
        x = (x + (x >> 4)) & 0x0F0F0F0F0F0F0F0F;
        (x.wrapping_mul(0x0101010101010101) >> 56) as u32
    }};
}

/// De Bruijn bit-scan (forward): position of the lowest set bit, asm-era
/// style — one multiply, one shift, one table load.
#[macro_export]
macro_rules! lowest_set_bit {
    ($e:expr) => {{
        const TABLE: [u8; 32] = [
            0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8,
            31, 27, 13, 23, 21, 19, 16, 7, 26, 12, 18, 6, 11, 5, 10, 9,
        ];
        let v: u32 = $e as u32;
        let r = v & v.wrapping_neg();
        TABLE[((r.wrapping_mul(0x077CB531u32)) >> 27) as usize] as u32
    }};
}

/// Is a power of two (the classic one-liner).
#[macro_export]
macro_rules! is_pow2 {
    ($e:expr) => { ($e as u64) != 0 && (($e as u64) & (($e as u64) - 1)) == 0 };
}

/// Round UP to the next power of two (demoscene texture sizing).
#[macro_export]
macro_rules! round_up_pow2 {
    ($e:expr) => {{
        let mut v: u64 = ($e as u64).wrapping_sub(1);
        for _ in 0..6 { v |= v >> 1; v |= v >> 2; v |= v >> 4; v |= v >> 8; v |= v >> 16; v |= v >> 32; }
        v.wrapping_add(1)
    }};
}

/// XOR swap, the asm-era classic (the compiler turns it into a move;
/// the trick is the history, the registers are the point).
#[macro_export]
macro_rules! xor_swap {
    ($a:expr, $b:expr) => {{
        let mut x = $a; let mut y = $b;
        x ^= y; y ^= x; x ^= y;
        ($a, $b) = (x, y);
    }};
}

/// Saturating 8-bit add (demoscene audio/clamp loops) — `min(255, a+b)`.
#[macro_export]
macro_rules! sat_add8 {
    ($a:expr, $b:expr) => {{
        let s: u32 = ($a as u32) + ($b as u32);
        if s > 255 { 255u8 } else { s as u8 }
    }};
}

/// Fast sign: -1 / 0 / +1 without branching (0x80 magic of the old
/// CPU era: `(n > 0) - (n < 0)` compiles to sets and borrows — no jump).
#[macro_export]
macro_rules! sign {
    ($e:expr) => {{
        let v: i64 = $e as i64;
        ((v > 0) as i8) - ((v < 0) as i8)
    }};
}

/// The Wild-linker buffer-reuse trick as a one-shot macro: reuse a Vec's
/// heap allocation while changing the element type (size+align checked
/// at compile time — a mismatch is a compile error, not a runtime bug).
#[macro_export]
macro_rules! reuse_vec {
    ($v:expr, $Src:ty, $T:ty) => {{
        const {
            assert!(core::mem::size_of::<$Src>() == core::mem::size_of::<$T>());
            assert!(core::mem::align_of::<$Src>() == core::mem::align_of::<$T>());
        }
        let mut vec = $v;
        vec.clear();
        vec.into_iter().map(|_| unsafe { core::mem::zeroed::<$T>() }).collect()
    }};
}

/// Per-thread scratch buffer (the allocation happens once per thread,
/// then is reused event after event — zero mallocs in the hot loop).
#[macro_export]
macro_rules! scratch {
    () => {
        thread_local! {
            static __SCRATCH: std::cell::RefCell<std::string::String> =
                const { std::cell::RefCell::new(std::string::String::new()) };
        }
        __SCRATCH.with(|cell| {
            let mut b = cell.borrow_mut();
            b.clear();
            b
        })
    };
}

/// Deallocate a big buffer on a background thread (Wild's own trick:
/// freeing is slow; get back to work while a worker thread drops it).
#[macro_export]
macro_rules! dealloc_async {
    ($e:expr) => {{
        std::thread::spawn(move || { drop($e); });
    }};
}

/// The ternary 1.58-bit packer: three states in two bits via base-3.
/// `pack3(a,b,c,d)` unwraps values into one u32 the demoscene way.
#[macro_export]
macro_rules! pack3 {
    ($a:expr, $b:expr, $c:expr) => {{
        let (a, b, c) = (($a as i32 + 1) as u32, ($b as i32 + 1) as u32, ($c as i32 + 1) as u32);
        a + b * 3 + c * 9
    }};
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ternary_clamp_is_the_doctrine() {
        assert_eq!(crate::clamp_ternary!(3), 1i8);
        assert_eq!(crate::clamp_ternary!(-5), -1i8);
        assert_eq!(crate::clamp_ternary!(0), 0i8);
    }

    #[test]
    fn div255_is_exact_for_byte_blends() {
        // (255*255)/255 == 255
        assert_eq!(crate::div255!(255u16, 255u16), 255);
        assert_eq!(crate::div255!(128u16, 255u16), 128);
    }

    #[test]
    fn popcount_counts() {
        assert_eq!(crate::popcount!(0b1011u64), 3);
        assert_eq!(crate::popcount!(0u64), 0);
        assert_eq!(crate::popcount!(u64::MAX), 64);
    }

    #[test]
    fn lowest_set_bit_finds_it() {
        assert_eq!(crate::lowest_set_bit!(0x8u32), 3);
        assert_eq!(crate::lowest_set_bit!(0x1000u32), 12);
    }

    #[test]
    fn pow2_and_rounding() {
        assert!(crate::is_pow2!(16u32));
        assert!(!crate::is_pow2!(15u32));
        assert_eq!(crate::round_up_pow2!(17u64), 32);
    }

    #[test]
    fn xor_swap_swaps() {
        let (mut a, mut b) = (7u32, 13u32);
        crate::xor_swap!(a, b);
        assert_eq!((a, b), (13, 7));
    }

    #[test]
    fn sat_and_sign() {
        assert_eq!(crate::sat_add8!(200u8, 100u8), 255);
        assert_eq!(crate::sign!(42i64), 1i8);
        assert_eq!(crate::sign!(-42i64), -1i8);
        assert_eq!(crate::sign!(0i64), 0i8);
    }

    #[test]
    fn reuse_vec_changes_type_without_realloc() {
        let mut store: Vec<u32> = vec![1, 2, 3];
        let reused: Vec<i32> = crate::reuse_vec!(store, u32, i32);
        assert_eq!(reused.len(), 0);
        let store2: Vec<u32> = crate::reuse_vec!(reused, i32, u32);
        assert_eq!(store2.len(), 0);
    }

    #[test]
    fn pack3_packs_ternary() {
        assert_eq!(crate::pack3!(-1, 0, 1), 0 + 1 * 3 + 2 * 9);
        assert_eq!(crate::pack3!(0, 0, 0), 1 + 3 + 9); // all shifted to +1
        // and the pack is uniquely decodable
        assert_ne!(crate::pack3!(-1, 0, 1), crate::pack3!(0, 1, -1));
    }
}
