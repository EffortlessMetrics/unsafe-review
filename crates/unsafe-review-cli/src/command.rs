use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    Human,
    Json,
    Markdown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CheckOptions {
    pub root: PathBuf,
    pub base: Option<String>,
    pub diff: Option<PathBuf>,
    pub format: Format,
    pub out: Option<PathBuf>,
    pub max_cards: Option<usize>,
}

impl Default for CheckOptions {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            base: None,
            diff: None,
            format: Format::Human,
            out: None,
            max_cards: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Command {
    Help,
    Version,
    Doctor {
        root: PathBuf,
    },
    Check(CheckOptions),
    Repo(CheckOptions),
    Pilot(CheckOptions),
    Badges {
        root: PathBuf,
        out: PathBuf,
    },
    Explain {
        root: PathBuf,
        id: String,
        format: Format,
    },
    Context {
        root: PathBuf,
        id: String,
    },
}
