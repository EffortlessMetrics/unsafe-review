pub fn checked_len(bytes: &[u8]) -> usize {
    bytes.len()
}

pub fn double(value: usize) -> usize {
    value.saturating_mul(2)
}
