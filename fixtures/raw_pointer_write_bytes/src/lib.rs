pub fn fill_bytes(ptr: *mut u8, len: usize, byte: u8) {
    // SAFETY: fixture keeps the operation visible but omits concrete guards.
    unsafe { ptr.write_bytes(byte, len) }
}

#[cfg(test)]
mod tests {
    use super::fill_bytes;

    #[test]
    fn fills_bytes() {
        let mut bytes = [0_u8; 4];
        fill_bytes(bytes.as_mut_ptr(), bytes.len(), 7);
        assert_eq!(bytes, [7; 4]);
    }
}
