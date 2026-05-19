pub fn push_encoded(c: char, ptr: *mut u8, len: usize, out: &mut Buffer) -> Result<(), ()> {
    let remaining_cap = out.capacity() - len;
    let n = encode_utf8(c, ptr, remaining_cap)?;

    // SAFETY: `encode_utf8` initialized the returned `n` bytes.
    unsafe {
        out.set_len(len + n);
    }

    Ok(())
}
