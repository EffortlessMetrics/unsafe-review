#[inline]
unsafe fn write_one(ptr: *mut u8, byte: u8) {
    unsafe {
        core::ptr::write(ptr, byte);
    }
}
