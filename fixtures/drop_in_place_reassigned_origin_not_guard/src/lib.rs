pub fn drop_reassigned(value: Box<String>, foreign_ptr: *mut String) {
    let mut ptr = Box::into_raw(value);
    ptr = foreign_ptr;
    // SAFETY: this fixture intentionally reassigns ptr after Box::into_raw.
    unsafe {
        core::ptr::drop_in_place(ptr);
    }
}
