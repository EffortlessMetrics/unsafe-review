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
    let _observed = match allocate(Fallibility::Infallible) {
        Ok(value) => value,
        Err(_) => 0,
    };

    // SAFETY: fixture deliberately places this after the closed infallible match.
    unsafe { core::hint::unreachable_unchecked() }
}

#[cfg(test)]
mod tests {
    use super::with_capacity;

    #[test]
    fn mentions_with_capacity() {
        let _ = stringify!(with_capacity);
    }
}
