pub fn invalid_zeroed_nonnull() -> core::ptr::NonNull<u8> {
    // SAFETY: fixture keeps the zeroed value visible but omits valid-zero evidence.
    unsafe { core::mem::zeroed::<core::ptr::NonNull<u8>>() }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mentions_invalid_zeroed_nonnull() {
        // Mention the owner for static reach without executing the invalid zeroed value.
        let _ = stringify!(invalid_zeroed_nonnull);
    }
}
