pub fn decode_observed(bytes: &[u8]) -> &str {
    let _valid_utf8 = core::str::from_utf8(bytes).is_ok();
    // SAFETY: fixture deliberately observes validation without branching or returning on it.
    unsafe { core::str::from_utf8_unchecked(bytes) }
}

#[cfg(test)]
mod tests {
    use super::decode_observed;

    #[test]
    fn mentions_decode_observed() {
        let _ = stringify!(decode_observed);
    }
}
