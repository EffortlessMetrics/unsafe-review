pub fn decode(bytes: &[u8]) -> Result<&str, core::str::Utf8Error> {
    match core::str::from_utf8(bytes) {
        Ok(_) => {}
        Err(err) => return Err(err),
    }

    // SAFETY: the match above returns before this point when the same bytes are not UTF-8.
    Ok(unsafe { core::str::from_utf8_unchecked(bytes) })
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn decodes_ascii() {
        assert_eq!(decode(b"ok").unwrap(), "ok");
    }

    #[test]
    fn rejects_invalid() {
        assert!(decode(&[0xff]).is_err());
    }
}
