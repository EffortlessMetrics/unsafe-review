use core::mem::MaybeUninit;

pub fn drop_slot(slot: &mut MaybeUninit<String>) {
    // SAFETY: fixture documents the expected initialization contract but omits proof.
    unsafe { slot.assume_init_drop() }
}

#[cfg(test)]
mod tests {
    use super::drop_slot;
    use core::mem::MaybeUninit;

    #[test]
    fn drops_slot() {
        let mut slot = MaybeUninit::new(String::from("drop me"));
        drop_slot(&mut slot);
    }
}

