pub struct ArrayString<const CAP: usize> {
    xs: [u8; CAP],
    len: usize,
}

impl<const CAP: usize> ArrayString<CAP> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        CAP
    }

    pub fn try_push(&mut self, c: char) -> Result<(), ()> {
        let len = self.len();
        let ptr = self.xs[len..].as_mut_ptr();
        let remaining_cap = self.capacity() - len;
        // SAFETY: `ptr` points to `remaining_cap` bytes.
        match unsafe { encode_utf8(c, ptr, remaining_cap) } {
            Ok(n) => {
                self.len = len + n;
                Ok(())
            }
            Err(_) => Err(()),
        }
    }
}

unsafe fn encode_utf8(_c: char, _ptr: *mut u8, _len: usize) -> Result<usize, ()> {
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::ArrayString;

    #[test]
    fn reaches_try_push() {
        let mut value = ArrayString {
            xs: [0; 4],
            len: 0,
        };
        let _result = value.try_push('a');
    }
}
