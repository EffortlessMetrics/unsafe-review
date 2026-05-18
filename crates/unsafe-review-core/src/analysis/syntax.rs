use ra_ap_syntax::{AstNode, Edition, SourceFile};

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

pub(crate) fn parse_source(text: impl Into<String>) -> ParsedSource {
    let text = text.into();
    let parse = SourceFile::parse(&text, Edition::CURRENT);
    let parse_errors = parse
        .errors()
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();
    let tree = parse.tree();
    let nodes = tree
        .syntax()
        .descendants()
        .map(|node| {
            let range = node.text_range();
            let start = text_size_to_usize(range.start());
            let end = text_size_to_usize(range.end());
            let position = line_column(&text, start);
            SyntaxNodeFact {
                kind: format!("{:?}", node.kind()),
                start,
                end,
                line: position.line,
                column: position.column,
                snippet: snippet(&text, start, end),
            }
        })
        .collect();

    ParsedSource {
        text,
        parse_errors,
        nodes,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LineColumn {
    line: usize,
    column: usize,
}

fn line_column(text: &str, offset: usize) -> LineColumn {
    let offset = offset.min(text.len());
    let mut line = 1usize;
    let mut line_start = 0usize;

    for (idx, ch) in text.char_indices() {
        if idx >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            line_start = idx + ch.len_utf8();
        }
    }

    let column = text
        .char_indices()
        .skip_while(|(idx, _ch)| *idx < line_start)
        .take_while(|(idx, _ch)| *idx < offset)
        .count()
        + 1;

    LineColumn { line, column }
}

fn snippet(text: &str, start: usize, end: usize) -> String {
    text.get(start..end)
        .map_or_else(String::new, str::to_string)
}

fn text_size_to_usize(size: ra_ap_syntax::TextSize) -> usize {
    u32::from(size) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_complete_rust_source_without_errors() {
        let parsed = parse_source("pub fn answer() -> usize {\n    42\n}\n");

        assert!(parsed.parse_errors.is_empty());
        assert!(
            parsed
                .nodes
                .iter()
                .any(|node| node.kind == "FN" && node.snippet.contains("answer"))
        );
    }

    #[test]
    fn records_line_column_and_snippet_for_nested_nodes() -> Result<(), String> {
        let parsed =
            parse_source("fn outer() {\n    unsafe { core::ptr::read(0 as *const u8); }\n}\n");

        let Some(call) = parsed
            .nodes
            .iter()
            .find(|node| node.kind == "CALL_EXPR" && node.snippet.contains("core::ptr::read"))
        else {
            return Err("expected a nested call expression node".to_string());
        };

        assert_eq!(call.line, 2);
        assert_eq!(call.column, 14);
        assert_eq!(call.snippet, "core::ptr::read(0 as *const u8)");
        Ok(())
    }

    #[test]
    fn reports_parse_errors_without_discarding_tree() {
        let parsed = parse_source("pub fn broken( {\n");

        assert!(!parsed.parse_errors.is_empty());
        assert!(parsed.nodes.iter().any(|node| node.kind == "SOURCE_FILE"));
    }
}
