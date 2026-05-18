use core::mem::MaybeUninit;

pub fn fill_tag(slice: &mut [MaybeUninit<u8>], byte: u8) {
    // SAFETY: this fixture writes to MaybeUninit storage but omits pointer,
    // alignment, allocation, and witness evidence.
    unsafe { slice.as_mut_ptr().write_bytes(byte, slice.len()) }
}

#[cfg(test)]
mod tests {
    use super::fill_tag;
    use core::mem::MaybeUninit;

    #[test]
    fn fills_tag_storage() {
        let mut tags = [MaybeUninit::<u8>::uninit(); 4];
        fill_tag(&mut tags, 7);
    }
}
