pub unsafe fn read_first(ptr: *const u8, len: usize) -> u8 {
    if len > 0 {
        unsafe { ptr.read() }
    } else {
        0
    }
}
