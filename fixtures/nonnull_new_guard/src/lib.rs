use core::ptr::NonNull;

pub fn expose_nonnull(ptr: *mut u8) -> Option<NonNull<u8>> {
    NonNull::new(ptr)?;
    // SAFETY: NonNull::new returned Some above, so ptr is non-null.
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
