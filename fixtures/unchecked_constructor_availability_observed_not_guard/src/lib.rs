pub struct One {
    needle: u8,
}

impl One {
    pub fn is_available() -> bool {
        true
    }

    pub unsafe fn new_unchecked(needle: u8) -> One {
        One { needle }
    }

    pub fn new(needle: u8) -> One {
        let available = One::is_available();
        observe(available);
        // SAFETY: this fixture intentionally observes availability without enforcing it.
        unsafe { One::new_unchecked(needle) }
    }
}

fn observe(_available: bool) {}

#[cfg(test)]
mod tests {
    use super::One;

    #[test]
    fn mentions_new() {
        let _ = stringify!(One::new);
    }
}
