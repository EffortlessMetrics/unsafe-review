pub fn pick_checked(values: &mut [u8], index: usize) -> Option<&mut u8> {
    if index < values.len() {
        // SAFETY: `index` was checked against `values.len()`.
        Some(unsafe { values.get_unchecked_mut(index) })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::pick_checked;

    #[test]
    fn picks_checked_value() {
        let mut values = [1_u8, 2, 3];
        let picked = pick_checked(&mut values, 1).expect("in bounds");
        *picked = 9;
    }
}

