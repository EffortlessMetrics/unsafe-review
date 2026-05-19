pub fn extend_len(values: &mut Vec<u8>, new_len: usize, other: usize) {
    let capacity = values.capacity();
    if other < capacity {
        record_capacity(capacity);
    }
    // SAFETY: this fixture compares a different value with capacity but never bounds new_len.
    unsafe { values.set_len(new_len) }
}

fn record_capacity(_capacity: usize) {}

#[cfg(test)]
mod tests {
    use super::extend_len;

    #[test]
    fn mentions_extend_len() {
        let _ = stringify!(extend_len);
    }
}
