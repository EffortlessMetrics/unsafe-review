#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafetyObligation {
    pub key: String,
    pub description: String,
}

impl SafetyObligation {
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
        }
    }
}
