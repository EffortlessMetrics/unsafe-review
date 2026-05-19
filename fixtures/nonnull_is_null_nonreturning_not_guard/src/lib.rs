use core::ptr::NonNull;

pub fn expose_nonnull(ptr: *mut u8) -> Option<NonNull<u8>> {
    if ptr.is_null() {
        record_null();
    }
    // SAFETY: this fixture intentionally observes null without exiting.
    Some(unsafe { NonNull::new_unchecked(ptr) })
}

fn record_null() {}

#[cfg(test)]
mod tests {
    use super::expose_nonnull;

    #[test]
    fn mentions_expose_nonnull() {
        let _ = stringify!(expose_nonnull);
    }
}

