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
    use proptest::collection::vec;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    #[derive(Clone, Debug)]
    enum DiffLine {
        Added,
        Context,
        Removed,
    }

    fn diff_line_strategy() -> impl Strategy<Value = DiffLine> {
        prop_oneof![
            Just(DiffLine::Added),
            Just(DiffLine::Context),
            Just(DiffLine::Removed),
        ]
    }

    fn path_strategy() -> impl Strategy<Value = PathBuf> {
        ("[a-z][a-z0-9_]{0,12}", "[a-z][a-z0-9_]{0,12}")
            .prop_map(|(dir, file)| PathBuf::from(format!("{dir}/{file}.rs")))
    }

    proptest! {
        #[test]
        fn unified_diff_changed_lines_match_added_new_file_coordinates(
            path in path_strategy(),
            hunk_start in 1usize..500,
            lines in vec(diff_line_strategy(), 1..80),
        ) {
            let mut diff = format!(
                "diff --git a/{0} b/{0}\n--- a/{0}\n+++ b/{0}\n@@ -1,1 +{hunk_start},1 @@\n",
                path.display()
            );
            let mut expected = BTreeSet::new();
            let mut new_line = hunk_start;

            for line in lines {
                match line {
                    DiffLine::Added => {
                        diff.push_str("+added();\n");
                        expected.insert(new_line);
                        new_line = new_line.saturating_add(1);
                    }
                    DiffLine::Context => {
                        diff.push_str(" context();\n");
                        new_line = new_line.saturating_add(1);
                    }
                    DiffLine::Removed => {
                        diff.push_str("-removed();\n");
                    }
                }
            }

            let index = parse_unified_diff(&diff);
            let actual = index.changed_lines.get(&path).cloned().unwrap_or_default();
            prop_assert_eq!(actual, expected);
            prop_assert!(index.contains_file(&path));
        }

        #[test]
        fn parser_accepts_arbitrary_text_without_panicking(lines in vec(".{0,80}", 0..40)) {
            let input = lines.join("\n");
            let _index = parse_unified_diff(&input);
        }
    }
}
