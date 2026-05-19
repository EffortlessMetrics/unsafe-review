use core::mem::MaybeUninit;

pub fn fill_tag(ptr: *mut u16, len: usize, byte: u8) {
    let _scratch: MaybeUninit<u16> = MaybeUninit::uninit();

    // SAFETY: this fixture intentionally mentions unrelated MaybeUninit.
    unsafe { ptr.write_bytes(byte, len) }
}

#[cfg(test)]
mod tests {
    use super::fill_tag;

    #[test]
    fn mentions_fill_tag() {
        let _ = stringify!(fill_tag);
    }
}

