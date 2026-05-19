pub fn expose_mut(ptr: *mut u8, len: usize) -> &'static mut [u8] {
    // SAFETY: fixture keeps the operation visible but omits concrete guards.
    unsafe { core::slice::from_raw_parts_mut(ptr, len) }
}
