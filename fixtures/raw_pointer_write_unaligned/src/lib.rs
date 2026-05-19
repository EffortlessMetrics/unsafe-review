pub struct Header(u32);

pub fn write_unaligned_header(bytes: &mut [u8], header: Header) {
    assert!(bytes.len() >= core::mem::size_of::<Header>());
    let ptr = bytes.as_mut_ptr().cast::<Header>();
    // SAFETY: length is checked above; write_unaligned intentionally does not
    // require alignment, so this card should not ask for an alignment guard.
    unsafe { core::ptr::write_unaligned(ptr, header) }
}

#[cfg(test)]
mod tests {
    use super::{write_unaligned_header, Header};

    #[test]
    fn writes_unaligned_header() {
        let mut bytes = [0_u8; 8];
        write_unaligned_header(&mut bytes[1..], Header(1));
    }
}
