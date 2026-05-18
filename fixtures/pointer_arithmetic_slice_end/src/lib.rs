pub fn count_bytes(haystack: &[u8]) -> usize {
    // SAFETY: `start` is derived from `haystack`, and adding `haystack.len()`
    // computes the one-past-the-end pointer for that same slice.
    unsafe {
        let start = haystack.as_ptr();
        let end = start.add(haystack.len());
        count_raw(start, end)
    }
}

pub unsafe fn count_raw(start: *const u8, end: *const u8) -> usize {
    end.addr().saturating_sub(start.addr())
}

#[cfg(test)]
mod tests {
    use super::count_bytes;

    #[test]
    fn counts_slice_distance() {
        assert_eq!(count_bytes(b"abcd"), 4);
    }
}
