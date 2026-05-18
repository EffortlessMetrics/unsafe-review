// SAFETY: this fixture documents ABI intent but provides no sanitizer receipt.
unsafe extern "C" {
    fn strlen(ptr: *const u8) -> usize;
}
