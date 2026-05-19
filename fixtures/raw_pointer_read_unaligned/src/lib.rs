pub struct Header(u32);

pub fn read_unaligned_header(bytes: &[u8]) -> Header {
    assert!(bytes.len() >= core::mem::size_of::<Header>());
    let ptr = bytes.as_ptr().cast::<Header>();
    // SAFETY: length is checked above; read_unaligned intentionally does not
    // require alignment, so this card should not ask for an alignment guard.
    unsafe { core::ptr::read_unaligned(ptr) }
}

#[cfg(test)]
mod tests {
    use super::read_unaligned_header;

    #[test]
    fn reads_unaligned_header() {
        let bytes = [0_u8; 8];
        let _header = read_unaligned_header(&bytes[1..]);
    }
}
