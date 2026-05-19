pub fn decode<'a>(bytes: &'a [u8], other: &[u8]) -> &'a str {
    if core::str::from_utf8(other).is_ok() {
        // SAFETY: fixture deliberately validates a different byte slice.
        unsafe { core::str::from_utf8_unchecked(bytes) }
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn decodes_ascii_when_other_is_valid() {
        assert_eq!(decode(b"ok", b"other"), "ok");
    }
}
