pub fn rebuild_vec(buf: *mut u8, len: usize, cap: usize, other_cap: usize) -> Vec<u8> {
    let capacity = other_cap;
    assert!(len <= capacity);
    record_capacity(capacity);
    // SAFETY: fixture mentions capacity but does not bound the cap argument used by Vec::from_raw_parts.
    unsafe { Vec::from_raw_parts(buf, len, cap) }
}

fn record_capacity(_: usize) {}

