pub fn extend_len(values: &mut Vec<u8>, new_len: usize) {
    let capacity = values.capacity();
    record_capacity(capacity);
    // SAFETY: this fixture intentionally observes capacity without bounding new_len.
    unsafe { values.set_len(new_len) }
}

fn record_capacity(_capacity: usize) {}

#[cfg(test)]
mod tests {
    use super::extend_len;

    #[test]
    fn mentions_extend_len() {
        let mut values = Vec::with_capacity(4);
        let _ = stringify!(extend_len);
        let _ = &mut values;
    }
}

