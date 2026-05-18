pub fn only_ok(result: Result<u8, ()>) -> u8 {
    match result {
        Ok(value) => value,
        // SAFETY: fixture documents the intended unreachable path but omits proof.
        Err(_) => unsafe { core::hint::unreachable_unchecked() },
    }
}

#[cfg(test)]
mod tests {
    use super::only_ok;

    #[test]
    fn reads_ok_value() {
        assert_eq!(only_ok(Ok(7)), 7);
    }
}
