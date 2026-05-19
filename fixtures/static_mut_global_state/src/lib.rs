// SAFETY: fixture documents a process-global mutable state contract but omits synchronization proof.
pub static mut GLOBAL_COUNT: usize = 0;

#[cfg(test)]
mod tests {
    #[test]
    fn mentions_global_count() {}
}
