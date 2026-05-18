pub fn write_header(ptr: *mut u32, value: u32) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: null is checked above, but alignment is not guarded here.
    unsafe {
        *ptr = value;
    }
}

#[cfg(test)]
mod tests {
    use super::write_header;

    #[test]
    fn writes_header() {
        let mut value = 0;
        write_header(&mut value, 7);
        assert_eq!(value, 7);
    }
}
