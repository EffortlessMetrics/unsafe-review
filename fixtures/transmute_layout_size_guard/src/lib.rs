pub fn byte_to_bool_with_size_guard(value: u8) -> bool {
    assert_eq!(core::mem::size_of::<u8>(), core::mem::size_of::<bool>());
    // SAFETY: size equality is checked, but this fixture intentionally lacks a valid-value proof that value is 0 or 1.
    unsafe { core::mem::transmute::<u8, bool>(value) }
}

#[cfg(test)]
mod tests {
    use super::byte_to_bool_with_size_guard;

    #[test]
    fn converts_known_bool_byte() {
        let _value = byte_to_bool_with_size_guard(1);
    }
}

