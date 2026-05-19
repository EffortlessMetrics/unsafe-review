pub fn set_named_cap(values: &mut Vec<u8>, requested: usize) {
    let cap = requested;
    // SAFETY: this fixture intentionally uses a local named cap without bounding capacity.
    unsafe { values.set_len(cap) }
}

