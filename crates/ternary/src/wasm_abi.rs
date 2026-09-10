// wasm_abi.rs — the JS-facing surface of the 1.58-bit lane (wasm32 only).
//
// The minimal extern "C" exports, in the engine's gaia_wire_c tradition:
// no wasm-bindgen, NUL-terminated UTF-8 across the boundary, the caller
// frees with ternary_free. Node drives the golden check in
// scripts/wasm-golden.mjs.

#[cfg(target_arch = "wasm32")]
fn alloc_c_string(s: String) -> *mut u8 {
    let bytes = s.as_bytes();
    let layout = std::alloc::Layout::array::<u8>(bytes.len() + 1).expect("layout");
    let ptr = unsafe { std::alloc::alloc(layout) };
    if ptr.is_null() {
        return ptr;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        *ptr.add(bytes.len()) = 0;
    }
    ptr
}

/// ternary_golden_hex() — the cross-surface determinism proof as a C
/// string: the committed base model's first 32 logits, hashed. The same
/// 64 hex chars must come out of the NEON lane (aarch64), the AVX2 lane
/// (x86-64, CI), and here, the simd128 lane.
#[no_mangle]
pub extern "C" fn ternary_golden_hex() -> *mut u8 {
    alloc_c_string(crate::golden_hash_hex())
}

/// ternary_free — release a string returned by ternary_golden_hex.
#[no_mangle]
pub extern "C" fn ternary_free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let mut len = 0usize;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let layout = std::alloc::Layout::array::<u8>(len + 1).expect("layout");
        std::alloc::dealloc(ptr, layout);
    }
}
