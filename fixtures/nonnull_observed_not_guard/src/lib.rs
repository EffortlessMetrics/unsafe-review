use core::ptr::NonNull;

pub fn expose_nonnull(ptr: *mut u8) -> Option<NonNull<u8>> {
    let _candidate = NonNull::new(ptr);
    // SAFETY: fixture deliberately observes NonNull::new without checking the result.
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
