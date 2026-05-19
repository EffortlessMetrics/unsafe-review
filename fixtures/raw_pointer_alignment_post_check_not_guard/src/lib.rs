pub struct Header(u32);

pub fn read_header(bytes: &[u8]) -> Option<Header> {
    assert!(bytes.len() >= core::mem::size_of::<Header>());
    let ptr = bytes.as_ptr();
    // SAFETY: fixture deliberately checks alignment only after the read.
    let header = unsafe { ptr.cast::<Header>().read() };
    if !ptr.cast::<Header>().is_aligned() {
        return None;
    }
    Some(header)
}

#[cfg(test)]
mod tests {
    use super::read_header;

    #[test]
    fn mentions_read_header() {
        let _ = stringify!(read_header);
    }
}
