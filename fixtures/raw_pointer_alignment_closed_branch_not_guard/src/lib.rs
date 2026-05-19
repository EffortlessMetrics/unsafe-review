pub struct Header(u32);

pub fn read_header(bytes: &[u8]) -> Header {
    assert!(bytes.len() >= core::mem::size_of::<Header>());
    let ptr = bytes.as_ptr();
    if ptr.cast::<Header>().is_aligned() {
        observe(ptr);
    }
    // SAFETY: this fixture intentionally closes the positive alignment branch
    // before the unsafe read.
    unsafe { ptr.cast::<Header>().read() }
}

fn observe<T>(_ptr: *const T) {}

#[cfg(test)]
mod tests {
    use super::read_header;

    #[test]
    fn mentions_read_header() {
        let _ = stringify!(read_header);
    }
}
