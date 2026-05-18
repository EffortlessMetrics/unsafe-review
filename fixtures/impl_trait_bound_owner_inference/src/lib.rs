pub fn try_reserve(
    ptr: *const u8,
    hasher: impl Fn(&u8) -> u64,
) -> u64 {
    // SAFETY: fixture keeps the unsafe wrapper visible but omits callee proof.
    unsafe { reserve_rehash(ptr, hasher) }
}

#[cfg(test)]
mod tests {
    use super::try_reserve;

    #[test]
    fn reaches_wrapper_name() {
        let _ = try_reserve(core::ptr::null(), |_| 7);
    }
}
