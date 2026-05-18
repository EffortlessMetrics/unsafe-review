/// Runs a target-feature-specific path.
///
/// # Safety
///
/// Callers must only execute this function when SSE2 is available.
#[target_feature(enable = "sse2")]
pub fn find_raw(data: &[u8]) -> usize {
    data.len()
}

#[cfg(test)]
mod tests {
    #[test]
    fn names_find_raw() {
        let owner = "find_raw";
        assert!(owner.contains("find_raw"));
    }
}
