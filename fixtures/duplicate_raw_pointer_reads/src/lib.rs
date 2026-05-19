pub struct Header(u32);

pub fn read_two_headers(bytes: &[u8]) {
    assert!(bytes.len() >= 2 * core::mem::size_of::<Header>());
    let ptr = bytes.as_ptr();
    // SAFETY: length is checked above, but alignment is intentionally omitted.
    unsafe { ptr.cast::<Header>().read() };
    // SAFETY: length is checked above, but alignment is intentionally omitted.
    unsafe { ptr.cast::<Header>().read() };
}
