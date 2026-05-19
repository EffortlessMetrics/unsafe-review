pub fn decode(bytes: &[u8]) -> &str {
    // SAFETY: fixture keeps the unchecked UTF-8 conversion visible but omits validation.
    unsafe { core::str::from_utf8_unchecked(bytes) }
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn decodes_ascii() {
        assert_eq!(decode(b"ok"), "ok");
    }
}
