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

/// ternary_dream_c(prompt, alphabet, n, temperature) — dream the `n`-token
/// continuation for a prompt, with the engine's own seeded PRNG and the
/// crate's f32 sampler, INSIDE the wasm. `alphabet` is the manifest's
/// vocab string (the char↔index truth). The caller owns the prompt and
/// appends the returned continuation (freed with ternary_free). Because
/// this is the same function the native dream runs, the dreamed bytes
/// are identical on every surface — the dream is a three-surface
/// artifact, not just the golden.
#[no_mangle]
pub extern "C" fn ternary_dream_c(
    prompt_ptr: *const u8,
    prompt_len: usize,
    alphabet_ptr: *const u8,
    alphabet_len: usize,
    n: usize,
    temperature: f32,
) -> *mut u8 {
    if prompt_ptr.is_null() || alphabet_ptr.is_null() {
        return std::ptr::null_mut();
    }
    let Ok(prompt) =
        (unsafe { std::str::from_utf8(std::slice::from_raw_parts(prompt_ptr, prompt_len)) })
    else {
        return std::ptr::null_mut();
    };
    let Ok(alphabet) =
        (unsafe { std::str::from_utf8(std::slice::from_raw_parts(alphabet_ptr, alphabet_len)) })
    else {
        return std::ptr::null_mut();
    };
    match crate::model::dream_continuation(prompt, alphabet, n, temperature) {
        Some(s) => alloc_c_string(s),
        None => std::ptr::null_mut(),
    }
}

/// ternary_alloc(len) — a byte buffer on the wasm heap for INPUTS: the
/// caller writes data + a NUL terminator, and ternary_free releases it
/// (the same len+1 contract as alloc_c_string).
#[no_mangle]
pub extern "C" fn ternary_alloc(len: usize) -> *mut u8 {
    let layout = std::alloc::Layout::array::<u8>(len + 1).expect("layout");
    unsafe { std::alloc::alloc(layout) }
}

/// ternary_free — release a string or buffer returned by the ternary_* ABI.

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
