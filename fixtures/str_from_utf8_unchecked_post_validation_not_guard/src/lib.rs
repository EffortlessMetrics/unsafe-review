pub fn decode(bytes: &[u8]) -> &str {
    // SAFETY: fixture deliberately validates only after the unchecked conversion.
    let decoded = unsafe { core::str::from_utf8_unchecked(bytes) };
    let _ = core::str::from_utf8(bytes).is_ok();
    decoded
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn decodes_ascii() {
        assert_eq!(decode(b"ok"), "ok");
    }
}
