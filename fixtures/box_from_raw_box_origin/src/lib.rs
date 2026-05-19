pub fn round_trip_box(value: Box<u8>) -> Box<u8> {
    let ptr = Box::into_raw(value);
    // SAFETY: this fixture ties the raw pointer to Box::into_raw in the same scope.
    unsafe { Box::from_raw(ptr) }
}

#[cfg(test)]
mod tests {
    use super::round_trip_box;

    #[test]
    fn mentions_round_trip_box() {
        let value = Box::new(1);
        let _ = round_trip_box(value);
    }
}

