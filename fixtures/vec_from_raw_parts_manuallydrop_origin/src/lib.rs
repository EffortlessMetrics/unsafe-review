pub fn rebuild_vec(input: Vec<u8>) -> Vec<u8> {
    let mut raw = core::mem::ManuallyDrop::new(input);
    let ptr = raw.as_mut_ptr();
    let len = raw.len();
    let cap = raw.capacity();
    // SAFETY: fixture shows same-pointer ownership transfer but no alignment or witness proof.
    unsafe { Vec::from_raw_parts(ptr, len, cap) }
}
