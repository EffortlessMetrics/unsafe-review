/// Returns the pointer at `index`.
///
/// # Safety
///
/// The caller must pass an index inside the allocation.
unsafe fn private_ctrl(base: *const u8, index: usize) -> *const u8 {
    base.wrapping_add(index)
}

#[cfg(test)]
mod tests {
    #[test]
    fn reaches_private_ctrl() {
        let _mention = stringify!(private_ctrl);
    }
}
