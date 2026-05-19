use core::mem::MaybeUninit;

pub fn expose_mut(ptr: *mut u8, len: usize) -> &'static mut [u8] {
    let _scratch: MaybeUninit<u8> = MaybeUninit::uninit();

    // SAFETY: this fixture intentionally mentions unrelated MaybeUninit.
    unsafe { core::slice::from_raw_parts_mut(ptr, len) }
}

#[cfg(test)]
mod tests {
    use super::expose_mut;

    #[test]
    fn mentions_expose_mut() {
        let _ = stringify!(expose_mut);
    }
}

