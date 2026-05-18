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

    fn constructor_boundary() {}

    pub fn new(needle: u8) -> Option<One> {
        Self::constructor_boundary();
        if One::is_available() {
            // SAFETY: availability is checked above before constructing this
            // target-specific searcher.
            unsafe { Some(One::new_unchecked(needle)) }
        } else {
            None
        }
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
        let one = One::new(7).expect("fixture availability is true");
        assert_eq!(one.needle(), 7);
    }
}
