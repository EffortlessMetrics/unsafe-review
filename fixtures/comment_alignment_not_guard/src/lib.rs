pub struct Header(u32);

pub fn read_header_from_comment(bytes: &[u8]) -> Header {
    assert!(bytes.len() >= core::mem::size_of::<Header>());
    let ptr = bytes.as_ptr();
    // TODO: add an align_of / is_aligned guard before this unsafe read ships.
    // SAFETY: length is checked above; the alignment words here are prose only.
    unsafe { ptr.cast::<Header>().read() }
}

#[cfg(test)]
mod tests {
    use super::read_header_from_comment;

    #[test]
    fn reads_header_from_comment() {
        let bytes = [0_u8; 8];
        let _header = read_header_from_comment(&bytes);
    }
}
