pub fn extract(result: Result<u8, ()>) -> u8 {
    // SAFETY: fixture deliberately observes Ok state without making it a guard.
    let _observed = result.is_ok();
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
