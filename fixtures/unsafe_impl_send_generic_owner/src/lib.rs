use core::cell::UnsafeCell;

pub struct Sender<T> {
    value: UnsafeCell<T>,
}

pub fn tick() {}

// SAFETY: this fixture states the generic sender thread-safety promise but provides no Loom witness.
unsafe impl<T: Send> Send for Sender<T> {}

#[cfg(test)]
mod tests {
    use super::{tick, Sender};
    use core::cell::UnsafeCell;

    #[test]
    fn constructs_sender() {
        tick();
        let _sender = Sender {
            value: UnsafeCell::new(1),
        };
    }
}

