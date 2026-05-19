pub fn byte_to_bool_closed_if_observed(value: u8) -> bool {
    assert_eq!(core::mem::size_of::<u8>(), core::mem::size_of::<bool>());
    if value <= 1 {
        let _observed_bool_byte = value;
    }
    // SAFETY: fixture deliberately observes the bool byte-domain branch without gating the call.
    unsafe { core::mem::transmute::<u8, bool>(value) }
}

#[cfg(test)]
mod tests {
    use super::byte_to_bool_closed_if_observed;

    #[test]
    fn mentions_byte_to_bool_closed_if_observed() {
        let _ = stringify!(byte_to_bool_closed_if_observed);
    }
}
