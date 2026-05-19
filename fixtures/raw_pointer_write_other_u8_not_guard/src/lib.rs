pub fn fill_words(ptr: *mut u16, other: *mut u8, len: usize, byte: u8) {
    let _ = other;

    // SAFETY: this fixture intentionally makes a different pointer `*mut u8`.
    unsafe { ptr.write_bytes(byte, len) }
}

#[cfg(test)]
mod tests {
    use super::fill_words;

    #[test]
    fn mentions_fill_words() {
        let _ = stringify!(fill_words);
    }
}

