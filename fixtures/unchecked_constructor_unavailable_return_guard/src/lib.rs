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

    pub fn new(needle: u8) -> Option<One> {
        if !One::is_available() {
            return None;
        }
        // SAFETY: the unavailable path returned before constructing this searcher.
        Some(unsafe { One::new_unchecked(needle) })
    }
}

#[cfg(test)]
mod tests {
    use super::One;

    #[test]
    fn constructs_when_available() {
        let _ = One::new(7);
    }
}
