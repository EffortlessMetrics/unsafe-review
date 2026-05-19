pub fn pick_unchecked_mut(values: &mut [u8], index: usize) -> &mut u8 {
    // SAFETY: caller must ensure `index` is in bounds; this fixture has no guard.
    unsafe { values.get_unchecked_mut(index) }
}

#[cfg(test)]
mod tests {
    use super::pick_unchecked_mut;

    #[test]
    fn picks_unchecked_mut() {
        let mut values = [1_u8, 2, 3];
        let picked = pick_unchecked_mut(&mut values, 1);
        *picked = 9;
    }
}
