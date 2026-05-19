pub fn pick_after_index_change(values: &mut [u8], mut index: usize) -> Option<&mut u8> {
    if index >= values.len() {
        return None;
    }
    index = values.len();
    // SAFETY: this fixture intentionally changes the checked index before use.
    Some(unsafe { values.get_unchecked_mut(index) })
}

#[cfg(test)]
mod tests {
    use super::pick_after_index_change;

    #[test]
    fn mentions_pick_after_index_change() {
        let _ = stringify!(pick_after_index_change);
    }
}
