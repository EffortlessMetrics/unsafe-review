// SAFETY: callers inside this module uphold the helper contract.
unsafe fn private_helper() {}

#[cfg(test)]
mod tests {
    use super::private_helper;

    #[test]
    fn reaches_private_helper() {
        // SAFETY: fixture only proves static reach sees this private helper.
        unsafe { private_helper() };
    }
}
