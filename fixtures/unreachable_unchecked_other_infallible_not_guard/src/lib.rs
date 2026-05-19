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
    let _other = allocate(Fallibility::Infallible);

    match allocate(Fallibility::Fallible) {
        Ok(value) => value,
        // SAFETY: this fixture intentionally makes a different call infallible.
        Err(_) => unsafe { core::hint::unreachable_unchecked() },
    }
}

#[cfg(test)]
mod tests {
    use super::with_capacity;

    #[test]
    fn mentions_with_capacity() {
        let _ = stringify!(with_capacity);
    }
}

