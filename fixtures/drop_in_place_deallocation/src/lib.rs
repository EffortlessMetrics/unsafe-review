pub fn drop_one(ptr: *mut String) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: caller transfers ownership of one initialized `String`.
    unsafe {
        core::ptr::drop_in_place(ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::drop_one;

    #[test]
    fn drop_one_reaches_operation() {
        let value = Box::new(String::from("owned"));
        drop_one(Box::into_raw(value));
    }
}

