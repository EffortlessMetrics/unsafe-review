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
    use std::collections::BTreeSet;

    #[derive(Clone, Copy, Debug)]
    enum DiffLine {
        Context,
        Addition,
        Removal,
    }

    prop_compose! {
        fn diff_lines()(lines in proptest::collection::vec(
            prop_oneof![
                Just(DiffLine::Context),
                Just(DiffLine::Addition),
                Just(DiffLine::Removal),
            ],
            0..64,
        )) -> Vec<DiffLine> {
            lines
        }
    }

    proptest! {
        #[test]
        fn prop_unified_diff_additions_map_to_new_file_line_numbers(
            new_start in 1usize..500,
            lines in diff_lines(),
        ) {
            let path = PathBuf::from("src/lib.rs");
            let mut diff = String::from("diff --git a/src/lib.rs b/src/lib.rs\n");
            diff.push_str("--- a/src/lib.rs\n");
            diff.push_str("+++ b/src/lib.rs\n");
            diff.push_str(&format!("@@ -1,1 +{new_start},1 @@\n"));

            let mut expected = BTreeSet::new();
            let mut new_line = new_start;
            for line in lines {
                match line {
                    DiffLine::Context => {
                        diff.push_str(" unchanged\n");
                        new_line = new_line.saturating_add(1);
                    }
                    DiffLine::Addition => {
                        diff.push_str("+added\n");
                        expected.insert(new_line);
                        new_line = new_line.saturating_add(1);
                    }
                    DiffLine::Removal => diff.push_str("-removed\n"),
                }
            }

            let index = parse_unified_diff(&diff);

            prop_assert_eq!(index.changed_lines.get(&path), Some(&expected));
        }
    }
}
