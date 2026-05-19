pub fn pick_checked_against_other<'a>(
    values: &'a mut [u8],
    other_values: &[u8],
    index: usize,
) -> Option<&'a mut u8> {
    if index < other_values.len() {
        // SAFETY: this fixture intentionally checks `other_values`, not `values`.
        Some(unsafe { values.get_unchecked_mut(index) })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::pick_checked_against_other;

    #[test]
    fn mentions_pick_checked_against_other() {
        let _ = stringify!(pick_checked_against_other);
    }
}

