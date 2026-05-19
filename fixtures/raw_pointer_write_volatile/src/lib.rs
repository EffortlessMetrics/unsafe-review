pub fn write_control(register: *mut u32, value: u32) {
    // SAFETY: caller provides a valid MMIO control register pointer; this
    // fixture intentionally omits local alignment and witness evidence.
    unsafe { register.write_volatile(value) }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mentions_write_control() {
        let _ = stringify!(write_control);
    }
}
