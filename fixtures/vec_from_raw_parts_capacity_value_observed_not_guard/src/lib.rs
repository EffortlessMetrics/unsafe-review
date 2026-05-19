pub fn rebuild_vec(buf: *mut u8, len: usize, cap: usize) -> Vec<u8> {
    let enough_capacity = len <= cap;
    observe(enough_capacity);
    // SAFETY: this fixture intentionally observes the len/cap relation without enforcing it.
    unsafe { Vec::from_raw_parts(buf, len, cap) }
}

fn observe(_value: bool) {}

#[cfg(test)]
mod tests {
    #[test]
    fn mentions_rebuild_vec() {
        let _ = stringify!(super::rebuild_vec);
    }
}
