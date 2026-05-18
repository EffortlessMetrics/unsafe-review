pub fn rebuild_vec(buf: *mut u8, len: usize, cap: usize) -> Vec<u8> {
    // SAFETY: fixture keeps the ownership operation visible but omits concrete guards.
    unsafe { Vec::from_raw_parts(buf, len, cap) }
}
