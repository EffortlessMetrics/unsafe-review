use core::mem::MaybeUninit;

pub struct RawControl<Tag> {
    ctrl: *mut Tag,
    len: usize,
}

impl<Tag> RawControl<Tag> {
    pub fn ctrl_slice(&mut self) -> &mut [MaybeUninit<Tag>] {
        // SAFETY: this fixture returns a MaybeUninit slice but intentionally
        // omits pointer-live, alignment, and allocation proof.
        unsafe { core::slice::from_raw_parts_mut(self.ctrl.cast(), self.len) }
    }
}

#[cfg(test)]
mod tests {
    use super::RawControl;

    #[test]
    fn reaches_ctrl_slice() {
        let mut tags = [0_u8; 2];
        let mut raw = RawControl {
            ctrl: tags.as_mut_ptr(),
            len: tags.len(),
        };
        let _slice = raw.ctrl_slice();
    }
}
