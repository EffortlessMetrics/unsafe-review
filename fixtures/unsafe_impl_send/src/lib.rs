use core::cell::UnsafeCell;

pub struct SharedCell {
    value: UnsafeCell<u32>,
}

// SAFETY: this fixture states the thread-safety promise but provides no Loom witness.
unsafe impl Send for SharedCell {}

#[cfg(test)]
mod tests {
    use super::SharedCell;
    use core::cell::UnsafeCell;

    #[test]
    fn constructs_shared_cell() {
        let _cell = SharedCell {
            value: UnsafeCell::new(1),
        };
    }
}
