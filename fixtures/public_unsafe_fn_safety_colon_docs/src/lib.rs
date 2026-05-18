/// Registers a raw pointer contract.
///
/// Safety: caller must pass a valid pointer to one initialized byte.
pub unsafe fn caller_contract(ptr: *const u8) {
    let _ = ptr;
}

#[cfg(test)]
mod tests {
    use super::caller_contract;

    #[test]
    fn reaches_contract() {
        let byte = 7u8;
        unsafe { caller_contract(&byte) };
    }
}
