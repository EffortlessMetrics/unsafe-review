pub fn drop_owned_box(value: Box<String>) {
    let ptr = Box::into_raw(value);
    // SAFETY: this fixture ties the pointer to Box::into_raw and drops it once.
    unsafe {
        core::ptr::drop_in_place(ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::drop_owned_box;

    #[test]
    fn mentions_drop_owned_box() {
        let value = Box::new(String::from("owned"));
        drop_owned_box(value);
    }
}

