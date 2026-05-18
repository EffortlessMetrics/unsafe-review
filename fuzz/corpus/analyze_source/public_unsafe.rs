/// # Safety
/// Caller must pass a valid pointer.
pub unsafe fn from_ptr(ptr: *const u8) -> u8 {
    unsafe { *ptr }
}
