use core::mem::MaybeUninit;

pub fn set_then_prepare(vec: &mut Vec<u8>, new_len: usize) {
    assert!(new_len <= vec.capacity());
    // SAFETY: fixture deliberately mentions initialization only after set_len.
    unsafe {
        vec.set_len(new_len);
    }
    let _late = MaybeUninit::new(0_u8);
}

#[cfg(test)]
mod tests {
    use super::set_then_prepare;

    #[test]
    fn mentions_set_then_prepare() {
        let _ = stringify!(set_then_prepare);
    }
}
