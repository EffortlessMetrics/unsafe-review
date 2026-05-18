use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

pub struct Channel<T> {
    head: AtomicPtr<T>,
}

impl<T> Channel<T> {
    pub fn discard_head(&self) -> *mut T {
        self.head.swap(ptr::null_mut(), Ordering::AcqRel)
    }
}

#[cfg(test)]
mod tests {
    use super::Channel;
    use core::ptr;
    use core::sync::atomic::AtomicPtr;

    #[test]
    fn discard_head_returns_old_pointer() {
        let mut value = 7_u8;
        let channel = Channel {
            head: AtomicPtr::new(&mut value),
        };

        let old = channel.discard_head();

        assert_eq!(old, &mut value);
        assert_eq!(channel.discard_head(), ptr::null_mut());
    }
}
