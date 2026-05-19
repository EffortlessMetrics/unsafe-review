pub fn decode(bytes: &[u8]) -> &str {
    if core::str::from_utf8(bytes).is_err() {
        return "";
    }

    // SAFETY: the branch above returns when this byte slice is not UTF-8.
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
