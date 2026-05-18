use ra_ap_syntax::{AstNode, Edition, SourceFile};

use super::location::line_column;
use super::source_text::{snippet, text_size_to_usize};
use super::types::{ParsedSource, SyntaxNodeFact};

pub(crate) fn parse_source(text: impl Into<String>) -> ParsedSource {
    let text = text.into();
    let parse = SourceFile::parse(&text, Edition::CURRENT);
    let parse_errors = parse_errors(&parse);
    let nodes = parse_nodes(&text, &parse.tree());

    ParsedSource {
        text,
        parse_errors,
        nodes,
    }
}

fn parse_errors(parse: &ra_ap_syntax::Parse<SourceFile>) -> Vec<String> {
    parse
        .errors()
        .iter()
        .map(std::string::ToString::to_string)
        .collect()
}

fn parse_nodes(text: &str, tree: &SourceFile) -> Vec<SyntaxNodeFact> {
    tree.syntax()
        .descendants()
        .map(|node| node_fact(text, &node))
        .collect()
}

fn node_fact(text: &str, node: &ra_ap_syntax::SyntaxNode) -> SyntaxNodeFact {
    let range = node.text_range();
    let start = text_size_to_usize(range.start());
    let end = text_size_to_usize(range.end());
    let position = line_column(text, start);

    SyntaxNodeFact {
        kind: format!("{:?}", node.kind()),
        start,
        end,
        line: position.line,
        column: position.column,
        snippet: snippet(text, start, end),
    }
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
