pub fn decode(bytes: &[u8]) -> &str {
    if core::str::from_utf8(bytes).is_ok() {
        // SAFETY: the branch above validated this byte slice as UTF-8.
        unsafe { core::str::from_utf8_unchecked(bytes) }
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn decodes_ascii() {
        assert_eq!(decode(b"ok"), "ok");
    }
}
