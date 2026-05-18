use core::ptr::NonNull;

pub fn expose_bucket(bucket: Bucket) -> NonNull<u8> {
    // SAFETY: fixture models an occupied bucket pointer but omits concrete guards.
    unsafe { forward(NonNull::new_unchecked(bucket.as_ptr())) }
}
