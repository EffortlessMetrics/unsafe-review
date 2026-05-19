pub fn extract(option: Option<u8>) -> u8 {
    // SAFETY: fixture deliberately observes Some state without making it a guard.
    let _observed = option.is_some();
    unsafe { option.unwrap_unchecked() }
}

#[cfg(test)]
mod tests {
    use super::extract;

    #[test]
    fn extracts_some_value() {
        assert_eq!(extract(Some(7)), 7);
    }
}
