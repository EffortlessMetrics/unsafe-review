pub struct Header(u32);

pub fn read_header(bytes: &[u8], other: *const u8) -> Option<Header> {
    assert!(bytes.len() >= core::mem::size_of::<Header>());
    let mut ptr = bytes.as_ptr();
    if !ptr.cast::<Header>().is_aligned() {
        return None;
    }
    ptr = other;
    // SAFETY: this fixture intentionally changes the checked pointer before
    // the unsafe read.
    Some(unsafe { ptr.cast::<Header>().read() })
}

#[cfg(test)]
mod tests {
    use super::read_header;

    #[test]
    fn mentions_read_header() {
        let bytes = [0_u8; 8];
        let _ = read_header(&bytes, bytes.as_ptr());
    }
}
