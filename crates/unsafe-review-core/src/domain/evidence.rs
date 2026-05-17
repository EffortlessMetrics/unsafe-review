#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractEvidence {
    pub present: bool,
    pub summary: String,
}

impl ContractEvidence {
    pub fn missing() -> Self {
        Self {
            present: false,
            summary: "No nearby `# Safety` docs or `SAFETY:` comment detected".to_string(),
        }
    }

    pub fn present(summary: impl Into<String>) -> Self {
        Self {
            present: true,
            summary: summary.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DischargeEvidence {
    pub present: bool,
    pub summary: String,
}

impl DischargeEvidence {
    pub fn missing() -> Self {
        Self {
            present: false,
            summary: "No visible local guard detected".to_string(),
        }
    }

    pub fn present(summary: impl Into<String>) -> Self {
        Self {
            present: true,
            summary: summary.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReachEvidence {
    pub state: String,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelatedTest {
    pub name: String,
    pub file: String,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingEvidence {
    pub kind: String,
    pub message: String,
}

impl MissingEvidence {
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
        }
    }
}
