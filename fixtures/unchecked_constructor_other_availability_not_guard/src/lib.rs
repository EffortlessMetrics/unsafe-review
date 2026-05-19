pub struct One {
    needle: u8,
}

pub struct Two;

impl Two {
    pub fn is_available() -> bool {
        true
    }
}

impl One {
    pub unsafe fn new_unchecked(needle: u8) -> One {
        One { needle }
    }

    pub fn new(needle: u8) -> Option<One> {
        if Two::is_available() {
            // SAFETY: this fixture intentionally checks a different constructor receiver.
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
    fn mentions_new() {
        let _ = stringify!(One::new);
    }
}

