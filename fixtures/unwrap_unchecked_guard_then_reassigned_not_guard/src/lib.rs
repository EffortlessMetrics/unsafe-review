pub fn extract(mut option: Option<u8>) -> u8 {
    if option.is_none() {
        return 0;
    }

    // SAFETY: fixture deliberately invalidates the checked receiver before unwrap.
    option = None;
    unsafe { option.unwrap_unchecked() }
}

#[cfg(test)]
mod tests {
    use super::extract;

    #[test]
    fn extracts_some_value() {
        assert_eq!(extract(Some(7)), 7);
        assert_eq!(extract(None), 0);
    }
}
