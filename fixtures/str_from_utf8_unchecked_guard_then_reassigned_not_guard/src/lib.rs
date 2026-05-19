pub fn decode_reassigned(input: &[u8]) -> Result<&str, core::str::Utf8Error> {
    let mut bytes = input;
    core::str::from_utf8(bytes)?;
    bytes = b"\xff";
    // SAFETY: fixture deliberately invalidates the validated slice before unchecked decode.
    Ok(unsafe { core::str::from_utf8_unchecked(bytes) })
}

#[cfg(test)]
mod tests {
    use super::decode_reassigned;

    #[test]
    fn mentions_decode_reassigned() {
        let _ = stringify!(decode_reassigned);
    }
}
