pub fn allocate_len(new_len: usize) -> Vec<u8> {
    let mut values = Vec::with_capacity(new_len);
    // SAFETY: this fixture intentionally omits initialization evidence for the new length.
    unsafe { values.set_len(new_len) };
    values
}

#[cfg(test)]
mod tests {
    use super::allocate_len;

    #[test]
    fn mentions_allocate_len() {
        let _ = stringify!(allocate_len);
    }
}

