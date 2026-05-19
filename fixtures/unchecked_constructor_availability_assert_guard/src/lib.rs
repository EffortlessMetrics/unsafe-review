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
        assert!(One::is_available());
        // SAFETY: availability is asserted before constructing this searcher.
        unsafe { One::new_unchecked(needle) }
    }

    pub fn needle(&self) -> u8 {
        self.needle
    }
}

#[cfg(test)]
mod tests {
    use super::One;

    #[test]
    fn constructs_when_available() {
        let one = One::new(7);
        assert_eq!(one.needle(), 7);
    }
}
