extern crate std;

use core::ptr::copy_nonoverlapping;
pub use core::mem::transmute as cast_value;

pub fn checked_len(bytes: &[u8]) -> usize {
    bytes.len()
}
