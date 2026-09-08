// wasm.rs — the JS-facing surface of world-core.
//
// Minimal extern "C" exports (no wasm-bindgen dependency): the browser
// client calls these directly. Strings cross as NUL-terminated UTF-8;
// the caller frees with gaia_free.

use crate::gaia::{gaia_state, gaia_wire};

/// allocate a NUL-terminated UTF-8 string on the wasm heap
/// (exact layout, so gaia_free can dealloc it safely)
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

/// gaia_wire_c(brief, tick) — the compact mesh frame as a C string.
/// Returns null on invalid UTF-8 input.
#[no_mangle]
pub extern "C" fn gaia_wire_c(brief: *const u8, brief_len: usize, tick: u64) -> *mut u8 {
    if brief.is_null() {
        return std::ptr::null_mut();
    }
    let bytes = unsafe { std::slice::from_raw_parts(brief, brief_len) };
    let Ok(brief) = std::str::from_utf8(bytes) else {
        return std::ptr::null_mut();
    };
    alloc_c_string(gaia_wire(&gaia_state(brief, tick)))
}

/// gaia_free — release a string returned by gaia_wire_c.
#[no_mangle]
pub extern "C" fn gaia_free(ptr: *mut u8) {
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
