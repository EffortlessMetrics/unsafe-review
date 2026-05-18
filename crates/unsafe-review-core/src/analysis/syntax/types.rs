#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ParsedSource {
    pub(crate) text: String,
    pub(crate) parse_errors: Vec<String>,
    pub(crate) nodes: Vec<SyntaxNodeFact>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SyntaxNodeFact {
    pub(crate) kind: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) snippet: String,
}
