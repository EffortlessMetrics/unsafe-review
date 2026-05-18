pub fn copy_byte_to_bool(value: u8) -> bool {
    // SAFETY: this fixture intentionally lacks a valid-value proof that value is 0 or 1.
    unsafe {
        core::mem::transmute_copy::<
            u8,
            bool,
        >(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::copy_byte_to_bool;

    #[test]
    fn copies_known_bool_byte() {
        let _value = copy_byte_to_bool(1);
    }
}

