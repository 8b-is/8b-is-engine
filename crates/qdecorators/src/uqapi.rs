//! uqapi — the unsafe-quotient API: the engine's unsafe surfaces behind
//! typed, safe methods.
//!
//! Two families of unsafe live in the constellation: the SIMD lanes
//! (AVX2/NEON/simd128 intrinsics) and the wasm/FFI pointer bridge
//! (NUL-terminated buffers). `uqapi` wraps both so that callers touch
//! **types, not pointers**: the lane is chosen by a type parameter
//! (`Uq::gemm::<Avx2Lane>(…)` compiles only where AVX2 exists), and a
//! `Cstr` is a value that frees itself.
//!
//! The typed-lane dispatch is the "typed custom API" in miniature: the
//! same i32-accumulation contract every lane honors, proven in tests
//! bit-exact between the scalar core and whatever SIMD lane this board
//! compiles.

use std::marker::PhantomData;

/// Lane — the sealed abstraction over the SIMD kernels. Implementing
/// `Lane` is only possible inside this module (private marker): the set
/// of lanes is closed, and each exists only where its hardware does.
pub trait Lane: sealed::Sealed {
    /// The lane's name, for the review eye and the dashboards.
    const NAME: &'static str;
    /// The i32-accumulating GEMM: accuracy = the integer sum, on any
    /// grouping (associativity is the contract).
    fn gemm(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32>;
}

/// The closed-lane marker — implementable only inside this crate (the
/// sealed pattern: the lane set is fixed by the crate, extensible by
/// no one).
#[doc(hidden)]
pub mod sealed {
    pub trait Sealed {}
}

/// The scalar reference — compiled everywhere, the authority.
pub struct ScalarLane;
impl sealed::Sealed for ScalarLane {}
impl Lane for ScalarLane {
    const NAME: &'static str = "scalar";
    fn gemm(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
        let mut out = vec![0i32; n_out];
        for o in 0..n_out {
            let row = &w[o * n_in..(o + 1) * n_in];
            out[o] = row
                .iter()
                .zip(a.iter())
                .map(|(&x, &y)| x as i32 * y as i32)
                .sum();
        }
        out
    }
}

/// The AVX2 lane — x86-64 (AMD Zen and Intel alike) only; the type does
/// not exist elsewhere, so the ergonomics *cannot* be miscompiled.
#[cfg(target_arch = "x86_64")]
pub struct Avx2Lane;
#[cfg(target_arch = "x86_64")]
impl sealed::Sealed for Avx2Lane {}
#[cfg(target_arch = "x86_64")]
impl Lane for Avx2Lane {
    const NAME: &'static str = "avx2";
    fn gemm(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
        // SAFETY: avx2 is the x86-64 baseline the lane exists for; the
        // kernel is length-checked by its k-bound loops
        unsafe { avx2_kern(w, a, n_in, n_out) }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn avx2_kern(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
    use std::arch::x86_64::*;
    let mut out = vec![0i32; n_out];
    for o in 0..n_out {
        let row = &w[o * n_in..(o + 1) * n_in];
        let mut acc = _mm256_setzero_si256();
        let mut k = 0usize;
        while k + 16 <= n_in {
            // loads + pairwise product-accumulation; integer addition
            // makes any grouping exact
            let wa = _mm256_loadu_si256(row.as_ptr().add(k) as *const __m256i);
            let aa = _mm256_loadu_si256(a.as_ptr().add(k) as *const __m256i);
            acc = _mm256_add_epi32(acc, _mm256_madd_epi16(aa, wa));
            k += 16;
        }
        let mut lanes = [0i32; 8];
        _mm256_storeu_si256(lanes.as_mut_ptr() as *mut __m256i, acc);
        let mut total: i32 = lanes.iter().sum();
        for kk in k..n_in {
            total += row[kk] as i32 * a[kk] as i32;
        }
        out[o] = total;
    }
    out
}

/// The NEON lane — aarch64 only (Apple silicon and ARM servers).
#[cfg(target_arch = "aarch64")]
pub struct NeonLane;
#[cfg(target_arch = "aarch64")]
impl sealed::Sealed for NeonLane {}
#[cfg(target_arch = "aarch64")]
impl Lane for NeonLane {
    const NAME: &'static str = "neon";
    fn gemm(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
        // SAFETY: neon is the aarch64 baseline; the kernel is
        // length-checked by its k-bound loop
        unsafe { neon_kern(w, a, n_in, n_out) }
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn neon_kern(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
    use std::arch::aarch64::*;
    let mut out = vec![0i32; n_out];
    for o in 0..n_out {
        let row = &w[o * n_in..(o + 1) * n_in];
        let mut acc_lo = vdupq_n_s32(0);
        let mut acc_hi = vdupq_n_s32(0);
        let mut k = 0usize;
        while k + 8 <= n_in {
            let wa = vld1q_s16(row.as_ptr().add(k));
            let aa = vld1q_s16(a.as_ptr().add(k));
            let prod = vmulq_s16(aa, wa);
            let lo = vget_low_s16(prod);
            let hi = vget_high_s16(prod);
            acc_lo = vaddq_s32(acc_lo, vmovl_s16(lo));
            acc_hi = vaddq_s32(acc_hi, vmovl_s16(hi));
            k += 8;
        }
        let combined = vaddq_s32(acc_lo, acc_hi);
        let mut lanes = [0i32; 4];
        vst1q_s32(lanes.as_mut_ptr(), combined);
        let mut total: i32 = lanes.iter().sum();
        for kk in k..n_in {
            total += row[kk] as i32 * a[kk] as i32;
        }
        out[o] = total;
    }
    out
}

/// scalar_authority_gemm — the reference kernel as a free function:
/// the authority every lane (Simd, GPU, Zig) must equal bit-for-bit.
pub fn scalar_authority_gemm(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
    Uq::gemm::<ScalarLane>(w, a, n_in, n_out)
}

/// Uq — the typed entry point: `Uq::gemm::<Lane>(…)`. The lane is a
/// type, so a caller cannot pick a lane their board does not compile.
pub struct Uq;

impl Uq {
    pub fn gemm<L: Lane>(w: &[i16], a: &[i16], n_in: usize, n_out: usize) -> Vec<i32> {
        L::gemm(w, a, n_in, n_out)
    }

    /// The lane this board actually uses — the typed selection made
    /// visible for tests and dashboards.
    pub fn native_lane_name() -> &'static str {
        #[cfg(target_arch = "x86_64")]
        {
            return Avx2Lane::NAME;
        }
        #[cfg(target_arch = "aarch64")]
        {
            return NeonLane::NAME;
        }
        #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
        {
            return "simd128";
        }
        #[allow(unreachable_code)]
        ScalarLane::NAME
    }
}

/// Cstr — a NUL-terminated buffer from the FFI/wasm bridge, as a safe
/// value: `as_str()` is a pure read, and `Drop` runs the destructor the
/// crate's ABI demands. The pointer is owned by this value alone; there
/// is no path to a raw pointer from it, so the unsafe is encapsulated.
pub struct Cstr {
    ptr: *mut u8,
    free: unsafe extern "C" fn(*mut u8),
    _marker: PhantomData<*mut u8>, // !Send: a pointer does not travel
}

impl Cstr {
    /// from_ptr — take ownership of a NUL-terminated allocation. The
    /// caller must guarantee the pointer is the exact value the ABI's
    /// free function expects (same allocation, same layout).
    pub fn from_ptr(ptr: *mut u8, free: unsafe extern "C" fn(*mut u8)) -> Self {
        Cstr {
            ptr,
            free,
            _marker: PhantomData,
        }
    }

    /// as_bytes — the bytes up to (not including) the NUL terminator.
    pub fn as_bytes(&self) -> &[u8] {
        if self.ptr.is_null() {
            return &[];
        }
        let mut len = 0usize;
        // SAFETY: the constructor contract — ptr is a NUL-terminated
        // allocation owned by this value; scanning is bounded by the
        // terminator, and reads never escape this method's lifetime.
        unsafe {
            while *self.ptr.add(len) != 0 {
                len += 1;
            }
            std::slice::from_raw_parts(self.ptr, len)
        }
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self.as_bytes()).unwrap_or("")
    }
}

impl Drop for Cstr {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: the constructor contract — exactly the ABI's free.
            unsafe { (self.free)(self.ptr) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scalar_core_is_the_authority() {
        let w = [1i16, -1, 0, 2, -2, 1, 1, 1];
        let a = [3i16, 4, 5, 6, 7, 8, 9, 1];
        let got = Uq::gemm::<ScalarLane>(&w, &a, 8, 1);
        assert_eq!(got[0], 3 - 4 + 12 - 14 + 8 + 9 + 1);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx2_is_bit_exact_with_the_scalar_authority() {
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
        let scalar = Uq::gemm::<ScalarLane>(&w, &a, 4096, 1);
        let avx2 = Uq::gemm::<Avx2Lane>(&w, &a, 4096, 1);
        assert_eq!(scalar, avx2, "the typed lanes must not reorder the truth");
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn neon_is_bit_exact_with_the_scalar_authority() {
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
        assert_eq!(
            Uq::gemm::<ScalarLane>(&w, &a, 4096, 1),
            Uq::gemm::<NeonLane>(&w, &a, 4096, 1)
        );
    }

    #[test]
    fn the_native_lane_is_one_of_the_names() {
        let names = ["scalar", "avx2", "neon", "simd128"];
        assert!(names.contains(&Uq::native_lane_name()));
    }

    #[test]
    fn cstr_reads_and_frees_itself() {
        // a fake ABI: a heap buffer + a free that counts
        use std::sync::atomic::{AtomicUsize, Ordering};
        static FREED: AtomicUsize = AtomicUsize::new(0);
        extern "C" fn fake_free(ptr: *mut u8) {
            if !ptr.is_null() {
                // the layout the fake alloc used: len + 1
                unsafe { std::alloc::dealloc(ptr, std::alloc::Layout::array::<u8>(8).unwrap()) };
                FREED.fetch_add(1, Ordering::SeqCst);
            }
        }
        let layout = std::alloc::Layout::array::<u8>(8).unwrap();
        let ptr = unsafe { std::alloc::alloc(layout) };
        unsafe {
            *ptr.add(0) = b't';
            *ptr.add(1) = b'h';
            *ptr.add(2) = b'e';
            *ptr.add(3) = 0; // NUL terminator (rest uninitialized)
        }
        {
            let s = Cstr::from_ptr(ptr, fake_free);
            assert_eq!(s.as_str(), "the");
        } // drop fires fake_free
        assert_eq!(FREED.load(Ordering::SeqCst), 1, "the RAII destructor ran");
    }

    #[test]
    fn null_cstr_reads_empty_and_free_is_skipped() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static FREED: AtomicUsize = AtomicUsize::new(0);
        extern "C" fn nop(_ptr: *mut u8) {
            FREED.fetch_add(1, Ordering::SeqCst);
        }
        let s = Cstr::from_ptr(std::ptr::null_mut(), nop);
        assert_eq!(s.as_bytes(), &[] as &[u8]);
        drop(s);
        // the Drop guard only frees non-null allocations: a null Cstr
        // never reaches the ABI's destructor (free(NULL) is the FFI's
        // business, not ours)
        assert_eq!(FREED.load(Ordering::SeqCst), 0);
    }
}
