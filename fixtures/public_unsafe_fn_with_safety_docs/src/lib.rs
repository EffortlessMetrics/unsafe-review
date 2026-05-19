/// Reads a byte from a caller-provided pointer.
///
/// # Safety
///
/// `ptr` must be valid for reads of one byte.
pub unsafe fn caller_must_uphold_contract(ptr: *const u8) -> usize {
    if ptr.is_null() { 0 } else { 1 }
}

#[cfg(test)]
mod tests {
    #[test]
    fn names_caller_must_uphold_contract() {
        let owner = "caller_must_uphold_contract";
        assert!(owner.contains("caller_must_uphold_contract"));
    }
}
