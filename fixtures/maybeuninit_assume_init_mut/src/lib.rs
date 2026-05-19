use core::mem::MaybeUninit;

pub fn borrow_slot_mut(slot: &mut MaybeUninit<u32>) -> &mut u32 {
    // SAFETY: fixture documents the expected initialization contract but omits proof.
    unsafe { slot.assume_init_mut() }
}

#[cfg(test)]
mod tests {
    use super::borrow_slot_mut;

    #[test]
    fn mutably_borrows_slot() {
        let mut slot = MaybeUninit::new(7);
        let value = borrow_slot_mut(&mut slot);
        *value = 8;
    }
}

