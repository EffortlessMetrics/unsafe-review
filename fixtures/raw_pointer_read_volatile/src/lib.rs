pub fn read_status(register: *const u32) -> u32 {
    // SAFETY: caller provides a valid MMIO status register pointer; this fixture
    // intentionally omits local alignment, lifetime, and witness evidence.
    unsafe { register.read_volatile() }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mentions_read_status() {
        let _ = stringify!(read_status);
    }
}
