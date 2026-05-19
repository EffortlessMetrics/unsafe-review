enum Fallibility {
    Infallible,
    Fallible,
}

fn reserve_rehash(fallibility: Fallibility) -> Result<u8, ()> {
    match fallibility {
        Fallibility::Infallible => Ok(7),
        Fallibility::Fallible => Err(()),
    }
}

pub fn reserve() -> u8 {
    let other_result = reserve_rehash(Fallibility::Infallible);
    let result = reserve_rehash(Fallibility::Fallible);
    let _ = other_result;

    // SAFETY: this fixture intentionally makes a different result infallible.
    unsafe { result.unwrap_unchecked() }
}

#[cfg(test)]
mod tests {
    use super::reserve;

    #[test]
    fn mentions_reserve() {
        let _ = stringify!(reserve);
    }
}

