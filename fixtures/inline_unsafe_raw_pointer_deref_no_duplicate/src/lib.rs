pub fn with_byte(ptr: *mut u8, f: impl FnOnce(&mut u8)) {
    // SAFETY: caller provides a valid pointer for the duration of `f`.
    if !ptr.is_null() {
        f(unsafe { &mut *ptr });
    }
}

#[cfg(test)]
mod tests {
    use super::with_byte;

    #[test]
    fn with_byte_reaches_inline_unsafe() {
        let mut value = 1;
        with_byte(&mut value, |slot| *slot = 2);
        assert_eq!(value, 2);
    }
}

