pub fn shift_left(bytes: &mut [u8], count: usize) {
    let src = bytes.as_ptr().wrapping_add(1);
    let dst = bytes.as_mut_ptr();
    // SAFETY: ptr::copy permits overlap, but this fixture intentionally omits
    // range and initialization evidence for the source/destination ranges.
    unsafe { core::ptr::copy(src, dst, count) }
}

#[cfg(test)]
mod tests {
    use super::shift_left;

    #[test]
    fn shifts_left() {
        let mut bytes = [1_u8, 2, 3, 4];
        shift_left(&mut bytes, 3);
        assert_eq!(bytes[0], 2);
    }
}
