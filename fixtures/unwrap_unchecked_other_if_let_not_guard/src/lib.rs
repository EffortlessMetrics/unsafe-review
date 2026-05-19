pub fn extract(option: Option<u8>, other: Option<u8>) -> u8 {
    if let Some(_) = other.as_ref() {
        // SAFETY: fixture deliberately checks another option.
        unsafe { option.unwrap_unchecked() }
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::extract;

    #[test]
    fn extracts_when_other_is_some() {
        assert_eq!(extract(Some(7), Some(1)), 7);
    }
}
