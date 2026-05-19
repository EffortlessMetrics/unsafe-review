use core::ptr::NonNull;

pub fn expose_nonnull(ptr: *mut u8, other: *mut u8) -> Option<NonNull<u8>> {
    NonNull::new(other)?;
    // SAFETY: this fixture intentionally validates `other`, not `ptr`.
    Some(unsafe { NonNull::new_unchecked(ptr) })
}

#[cfg(test)]
mod tests {
    use super::expose_nonnull;

    #[test]
    fn mentions_expose_nonnull() {
        let _ = stringify!(expose_nonnull);
    }
}
