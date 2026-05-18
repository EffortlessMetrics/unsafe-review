use core::pin::Pin;

pub fn pin_mut_unchecked(value: &mut u8) -> Pin<&mut u8> {
    // SAFETY: fixture names the pinning contract but does not prove it locally.
    unsafe { Pin::new_unchecked(value) }
}

#[cfg(test)]
mod tests {
    use super::pin_mut_unchecked;

    #[test]
    fn pins_mut_reference() {
        let mut value = 1_u8;
        let _pinned = pin_mut_unchecked(&mut value);
    }
}
