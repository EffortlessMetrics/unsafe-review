use core::hint;

enum Fallibility {
    Infallible,
    Fallible,
}

fn allocate(fallibility: Fallibility) -> Result<u8, ()> {
    match fallibility {
        Fallibility::Infallible => Ok(7),
        Fallibility::Fallible => Err(()),
    }
}

pub fn with_capacity() -> u8 {
    match allocate(Fallibility::Infallible) {
        Ok(value) => value,
        // SAFETY: infallible mode handles allocation errors before this point.
        Err(_) => unsafe { hint::unreachable_unchecked() },
    }
}

#[cfg(test)]
mod tests {
    use super::with_capacity;

    #[test]
    fn allocates_infallibly() {
        assert_eq!(with_capacity(), 7);
    }
}
