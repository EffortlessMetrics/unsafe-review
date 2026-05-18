use core::cell::UnsafeCell;

pub struct Receiver<T> {
    value: UnsafeCell<T>,
}

pub fn fmt() {}

// SAFETY: this fixture states the generic receiver thread-safety promise but provides no Loom witness.
unsafe impl<T: Send> Sync for Receiver<T> {}

#[cfg(test)]
mod tests {
    use super::{fmt, Receiver};
    use core::cell::UnsafeCell;

    #[test]
    fn constructs_receiver() {
        fmt();
        let _receiver = Receiver {
            value: UnsafeCell::new(1),
        };
    }
}

