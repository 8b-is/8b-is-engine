//! zigq — the Zig kernels, reached through the C ABI, wrapped typed.
//!
//! `extern "C"` is the DOOR; Zig is the ROOM. The kernels live in
//! `zig/kernels.zig` (compiled by `build.rs` when `zig` is on the
//! PATH), and this module declares their ABI as typed citizens: a
//! `ZigLane` that satisfies the crate's `Lane` contract (the same
//! i32-accumulation, proven bit-exact against the scalar authority in
//! tests), a safe `zig_pack` over the tri-state alphabet, and the
//! lane's version stamp.
//!
//! When `zig` is missing at build time, this module compiles to the
//! empty set: no ZigLane, no externs — the crate still builds, and the
//! scalar fallback carries the lane.

use crate::Lane;

/// The version stamp the Zig surface identifies itself with.
pub const ZIG_VERSION: u32 = 0x0001;

#[cfg(qdecorators_zig)]
extern "C" {
    fn zig_gemm(w: *const i16, a: *const i16, n_in: usize, n_out: usize, out: *mut i32);
    #[link_name = "zig_pack"]
    fn zig_pack_c(trits: *const i8, n: usize, out: *mut u8) -> usize;
    // the stamp is only referenced by the tests — the ABI keeps it warm
    #[allow(dead_code)]
    fn zig_version() -> u32;
}

/// ZigLane — the Zig kernels as a typed `Lane` (exists only when the
/// kernels were compiled into the build).
#[cfg(qdecorators_zig)]
pub struct ZigLane;

#[cfg(qdecorators_zig)]
impl crate::uqapi::sealed::Sealed for ZigLane {}

#[cfg(qdecorators_zig)]
impl Lane for ZigLane {
    const NAME: &'static str = "zig";
    fn gemm(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
        let mut out = vec![0i32; n_out];
        // SAFETY: the kernel is length-contracted by (n_in, n_out); the
        // buffers are exactly those sizes, and the kernel's inner loop
        // reads within them
        unsafe {
            zig_gemm(w.as_ptr(), a.as_ptr(), n_in, n_out, out.as_mut_ptr());
        }
        out
    }
}

/// zig_pack — the tri-state pack, in Zig's hands: four per byte,
/// little-endian fields, `0b11` reserved never written. Returns the
/// packed byte count.
#[cfg(qdecorators_zig)]
pub fn zig_pack(trits: &[i8]) -> Vec<u8> {
    let mut out = vec![0u8; trits.len().div_ceil(4)];
    unsafe {
        zig_pack_c(trits.as_ptr(), trits.len(), out.as_mut_ptr());
    }
    out
}

/// Whether the Zig kernels are in this build.
pub const ZIG_PRESENT: bool = cfg!(qdecorators_zig);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{scalar_authority_gemm, Uq};

    #[cfg(qdecorators_zig)]
    #[test]
    fn zig_is_bit_exact_with_the_scalar_authority() {
        let mut w = Vec::new();
        let mut a = Vec::new();
        for i in 0..4096i16 {
            w.push(if i % 3 == 0 {
                -1
            } else if i % 3 == 1 {
                1
            } else {
                0
            });
            a.push((((i as i32) * 37) % 200 - 100) as i16);
        }
        // wide + ragged shapes: the kernel loops must match on all
        for (n_in, n_out) in [(4096usize, 1usize), (64, 32), (4093, 3)] {
            let wn = n_in * n_out;
            let mut w2 = w[..wn.min(w.len())].to_vec();
            while w2.len() < wn {
                w2.push(0);
            }
            let a2 = &a[..n_in];
            let scalar = Uq::gemm::<crate::ScalarLane>(&w2, a2, n_in, n_out);
            let z = Uq::gemm::<ZigLane>(&w2, a2, n_in, n_out);
            assert_eq!(scalar, z, "the Zig kernel must not reorder the truth");
        }
    }

    #[cfg(qdecorators_zig)]
    #[test]
    fn zig_pack_matches_the_four_per_byte_alphabet() {
        let trits = [1i8, -1, 0, 1, -1, 0, 1, 1];
        let p = zig_pack(&trits);
        assert_eq!(p, vec![0b01_00_10_01, 0b01_01_00_10]);
        assert_eq!(p.len(), 2);
    }

    #[cfg(qdecorators_zig)]
    #[test]
    fn the_zig_version_stamp_identifies_the_lane() {
        unsafe {
            assert_eq!(zig_version(), ZIG_VERSION);
        }
    }

    #[test]
    fn the_presence_flag_is_honest() {
        assert_eq!(ZIG_PRESENT, cfg!(qdecorators_zig));
        // the scalar authority always exists, zig or not
        assert_eq!(scalar_authority_gemm(&[1, -1], &[3, 4], 2, 1), vec![3 - 4]);
    }
}
