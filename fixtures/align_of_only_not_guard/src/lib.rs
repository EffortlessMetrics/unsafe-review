pub struct Header(u32);

pub fn read_header(bytes: &[u8]) -> Header {
    assert!(bytes.len() >= core::mem::size_of::<Header>());
    let _required_alignment = core::mem::align_of::<Header>();
    let ptr = bytes.as_ptr();
    // SAFETY: length is checked above; merely computing align_of is not an
    // alignment guard for ptr.
    unsafe { ptr.cast::<Header>().read() }
}

#[cfg(test)]
mod tests {
    use super::read_header;

    #[test]
    fn mentions_read_header() {
        let bytes = [0_u8; 8];
        let _header = read_header(&bytes);
    }
}
