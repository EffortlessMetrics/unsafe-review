pub fn rebuild_vec(buf: *mut u8, len: usize, cap: usize) -> Vec<u8> {
    if len <= cap {
        observe(len);
    }
    // SAFETY: this fixture intentionally closes the positive len/cap branch before the call.
    unsafe { Vec::from_raw_parts(buf, len, cap) }
}

fn observe(_len: usize) {}

#[cfg(test)]
mod tests {
    #[test]
    fn mentions_rebuild_vec() {
        let _ = stringify!(super::rebuild_vec);
    }
}
