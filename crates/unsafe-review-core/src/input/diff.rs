use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub(crate) struct DiffIndex {
    pub(crate) changed_lines: BTreeMap<PathBuf, BTreeSet<usize>>,
}

impl DiffIndex {
    pub(crate) fn is_empty(&self) -> bool {
        self.changed_lines.is_empty()
    }

    pub(crate) fn contains_file(&self, path: &PathBuf) -> bool {
        self.changed_lines.contains_key(path)
    }

    pub(crate) fn contains_near(&self, path: &PathBuf, line: usize) -> bool {
        self.changed_lines
            .get(path)
            .is_some_and(|lines| lines.iter().any(|changed| changed.abs_diff(line) <= 6))
    }
}

pub(crate) fn parse_unified_diff(input: &str) -> DiffIndex {
    let mut index = DiffIndex::default();
    let mut current_path: Option<PathBuf> = None;
    let mut new_line = 0usize;

    for raw in input.lines() {
        if raw.starts_with("diff --git ") {
            current_path = None;
            continue;
        }

        if let Some(path) = raw.strip_prefix("+++ b/") {
            let path = PathBuf::from(path.trim());
            current_path = Some(path.clone());
            index.changed_lines.entry(path).or_default();
            continue;
        }

        if raw.starts_with("@@") {
            if let Some(start) = parse_new_start(raw) {
                new_line = start;
            }
            continue;
        }

        let Some(path) = current_path.as_ref() else {
            continue;
        };

        if raw.starts_with("+++") || raw.starts_with("---") {
            continue;
        }

        if raw.starts_with('+') {
            index
                .changed_lines
                .entry(path.clone())
                .or_default()
                .insert(new_line);
            new_line = new_line.saturating_add(1);
        } else if raw.starts_with('-') {
            // Removed lines do not advance the new-file coordinate.
        } else if raw.starts_with(' ') || raw.is_empty() {
            new_line = new_line.saturating_add(1);
        }
    }

    index
}

fn parse_new_start(header: &str) -> Option<usize> {
    // @@ -old,count +new,count @@
    let mut parts = header.split_whitespace();
    let _at = parts.next()?;
    let _old = parts.next()?;
    let new = parts.next()?;
    let new = new.trim_start_matches('+');
    let start = new.split(',').next()?;
    start.parse::<usize>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[derive(Clone, Debug)]
    enum DiffLine {
        Added,
        Removed,
        Context,
    }

    fn diff_line_strategy() -> impl Strategy<Value = DiffLine> {
        prop_oneof![
            Just(DiffLine::Added),
            Just(DiffLine::Removed),
            Just(DiffLine::Context),
        ]
    }

    fn render_diff(new_start: usize, lines: &[DiffLine]) -> (String, Vec<usize>) {
        let old_count = lines
            .iter()
            .filter(|line| !matches!(line, DiffLine::Added))
            .count();
        let new_count = lines
            .iter()
            .filter(|line| !matches!(line, DiffLine::Removed))
            .count();
        let mut next_new_line = new_start;
        let mut expected = Vec::new();
        let mut diff = format!(
            "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,{old_count} +{new_start},{new_count} @@\n"
        );

        for (idx, line) in lines.iter().enumerate() {
            match line {
                DiffLine::Added => {
                    expected.push(next_new_line);
                    next_new_line = next_new_line.saturating_add(1);
                    diff.push_str(&format!("+let added_{idx} = {idx};\n"));
                }
                DiffLine::Removed => {
                    diff.push_str(&format!("-let removed_{idx} = {idx};\n"));
                }
                DiffLine::Context => {
                    next_new_line = next_new_line.saturating_add(1);
                    diff.push_str(&format!(" let context_{idx} = {idx};\n"));
                }
            }
        }

        (diff, expected)
    }

    proptest! {
        #[test]
        fn parses_added_lines_in_new_file_coordinates(
            new_start in 1usize..500,
            lines in prop::collection::vec(diff_line_strategy(), 1..80),
        ) {
            let (diff, expected) = render_diff(new_start, &lines);
            let parsed = parse_unified_diff(&diff);
            let path = PathBuf::from("src/lib.rs");
            let actual = parsed
                .changed_lines
                .get(&path)
                .map(|lines| lines.iter().copied().collect::<Vec<_>>())
                .unwrap_or_default();

            prop_assert_eq!(actual, expected);
        }

        #[test]
        fn unchanged_and_removed_lines_never_count_as_changed(
            new_start in 1usize..500,
            lines in prop::collection::vec(prop_oneof![Just(DiffLine::Removed), Just(DiffLine::Context)], 1..80),
        ) {
            let (diff, _expected) = render_diff(new_start, &lines);
            let parsed = parse_unified_diff(&diff);
            let path = PathBuf::from("src/lib.rs");

            prop_assert!(parsed.contains_file(&path));
            prop_assert!(parsed.changed_lines.get(&path).is_some_and(BTreeSet::is_empty));
        }

        #[test]
        fn contains_near_matches_exact_six_line_window(
            changed_line in 1usize..1_000,
            query_line in 1usize..1_000,
        ) {
            let mut index = DiffIndex::default();
            let path = PathBuf::from("src/lib.rs");
            index
                .changed_lines
                .entry(path.clone())
                .or_default()
                .insert(changed_line);

            prop_assert_eq!(
                index.contains_near(&path, query_line),
                changed_line.abs_diff(query_line) <= 6
            );
        }
    }
}
