pub struct Header(u32);

pub fn read_header(bytes: &[u8]) -> Header {
    assert!(bytes.len() >= core::mem::size_of::<Header>());
    let ptr = bytes.as_ptr();
    let aligned = (ptr as usize) % core::mem::align_of::<Header>() == 0;
    observe(aligned);
    // SAFETY: this fixture intentionally observes alignment without enforcing it.
    unsafe { ptr.cast::<Header>().read() }
}

fn observe(_aligned: bool) {}

#[cfg(test)]
mod tests {
    use super::read_header;

    #[test]
    fn mentions_read_header() {
        let _ = stringify!(read_header);
    }
}
