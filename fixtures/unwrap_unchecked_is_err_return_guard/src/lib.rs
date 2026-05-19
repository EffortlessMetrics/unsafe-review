pub fn extract(result: Result<u8, ()>) -> u8 {
    if result.is_err() {
        return 0;
    }

    // SAFETY: the branch above returns when this result is Err.
    unsafe { result.unwrap_unchecked() }
}

#[cfg(test)]
mod tests {
    use super::extract;

    #[test]
    fn extracts_ok_value() {
        assert_eq!(extract(Ok(7)), 7);
        assert_eq!(extract(Err(())), 0);
    }
}
