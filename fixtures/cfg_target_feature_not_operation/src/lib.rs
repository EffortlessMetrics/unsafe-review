#[cfg(target_feature = "neon")]
pub fn neon_available() -> bool {
    true
}

#[cfg(not(target_feature = "neon"))]
pub fn neon_available() -> bool {
    false
}
