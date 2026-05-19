pub fn pause_once() {
    // SAFETY: fixture keeps inline assembly visible but omits register and memory invariants.
    unsafe { core::arch::asm!("nop") }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mentions_pause_once() {
        let _ = stringify!(pause_once);
    }
}
