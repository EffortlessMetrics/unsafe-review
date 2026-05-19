pub struct Header(u32);

pub fn read_header(bytes: &[u8], other_ptr: *const Header) -> Header {
    assert!(bytes.len() >= core::mem::size_of::<Header>());
    if (other_ptr as usize) % core::mem::align_of::<Header>() != 0 {
        return Header(0);
    }
    let ptr = bytes.as_ptr();
    // SAFETY: this fixture intentionally checks `other_ptr`, not the pointer being read.
    unsafe { ptr.cast::<Header>().read() }
}

#[cfg(test)]
mod tests {
    use super::read_header;

    #[test]
    fn mentions_read_header() {
        let _ = stringify!(read_header);
    }
}
