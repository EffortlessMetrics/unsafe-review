
pub struct Header(u32);

pub fn read_header(bytes: &[u8]) -> Header {
    assert!(bytes.len() >= core::mem::size_of::<Header>());
    let ptr = bytes.as_ptr();
    // SAFETY: length is checked above, but this fixture intentionally omits an
    // alignment guard so unsafe-review can emit a guard_missing card.
    unsafe { ptr.cast::<Header>().read() }
}

#[cfg(test)]
mod tests {
    use super::read_header;

    #[test]
    fn reads_header() {
        let bytes = [0_u8; 8];
        let _header = read_header(&bytes);
    }
}
