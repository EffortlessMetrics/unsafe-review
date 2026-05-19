pub fn zero_u32() -> u32 {
    // SAFETY: zero is a valid bit pattern for u32.
    unsafe { core::mem::zeroed::<u32>() }
}

#[cfg(test)]
mod tests {
    use super::zero_u32;

    #[test]
    fn returns_zero() {
        assert_eq!(zero_u32(), 0);
    }
}
