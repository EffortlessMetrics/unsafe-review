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
        if One::is_available() {
            observe_available();
        }
        // SAFETY: this fixture intentionally closes the availability branch before construction.
        unsafe { One::new_unchecked(needle) }
    }
}

fn observe_available() {}

#[cfg(test)]
mod tests {
    use super::One;

    #[test]
    fn mentions_new() {
        let _ = stringify!(One::new);
    }
}
