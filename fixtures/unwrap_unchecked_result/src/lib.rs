pub fn extract(result: Result<u8, ()>) -> u8 {
    // SAFETY: fixture keeps the unchecked unwrap visible but omits concrete guards.
    unsafe { result.unwrap_unchecked() }
}

#[cfg(test)]
mod tests {
    use super::extract;

    #[test]
    fn extracts_ok_value() {
        assert_eq!(extract(Ok(7)), 7);
    }
}
