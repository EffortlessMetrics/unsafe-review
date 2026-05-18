pub fn copy_bytes(src: *const u8, dst: *mut u8, count: usize) {
    // SAFETY: fixture keeps the operation visible but omits range and overlap guards.
    unsafe { core::ptr::copy_nonoverlapping(src, dst, count) }
}

#[cfg(test)]
mod tests {
    use super::copy_bytes;

    #[test]
    fn copies_bytes() {
        let src = [1_u8, 2, 3, 4];
        let mut dst = [0_u8; 4];
        copy_bytes(src.as_ptr(), dst.as_mut_ptr(), src.len());
        assert_eq!(dst, src);
    }
}
