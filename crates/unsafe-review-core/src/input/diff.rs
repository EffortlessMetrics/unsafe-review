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

    pub(crate) fn contains_in_range(&self, path: &PathBuf, start: usize, end: usize) -> bool {
        self.changed_lines.get(path).is_some_and(|lines| {
            lines
                .iter()
                .any(|changed| start <= *changed && *changed <= end)
        })
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

    #[derive(Clone, Debug)]
    enum DiffLine {
        Context(String),
        Added(String),
        Removed(String),
        EmptyContext,
    }

    #[test]
    fn parse_unified_diff_tracks_new_file_lines_and_skips_deletions() {
        let diff = r#"diff --git a/src/lib.rs b/src/lib.rs
index 1111111..2222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -8,7 +8,8 @@ fn demo() {
 context();
-old_call();
+new_call();
 unchanged();
+extra_call();
 tail();
"#;

        let index = parse_unified_diff(diff);
        let path = PathBuf::from("src/lib.rs");

        assert!(index.contains_file(&path));
        assert!(index.changed_lines[&path].contains(&9));
        assert!(index.changed_lines[&path].contains(&11));
        assert!(!index.changed_lines[&path].contains(&10));
        assert!(index.contains_near(&path, 15));
        assert!(!index.contains_near(&path, 18));
        assert!(index.contains_in_range(&path, 8, 12));
        assert!(!index.contains_in_range(&path, 12, 15));
    }

    #[test]
    fn parse_unified_diff_keeps_empty_entries_for_changed_files_without_additions() {
        let diff = r#"diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,2 +1,1 @@
 fn keep() {}
-fn removed() {}
"#;

        let index = parse_unified_diff(diff);
        let path = PathBuf::from("src/lib.rs");

        assert!(index.contains_file(&path));
        assert!(index.changed_lines[&path].is_empty());
        assert!(!index.contains_near(&path, 1));
    }

    #[test]
    fn parse_unified_diff_supports_single_line_hunk_headers() {
        let diff = r#"diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -42 +42 @@ fn main() {
+println!("new");
"#;

        let index = parse_unified_diff(diff);

        assert!(index.changed_lines[&PathBuf::from("src/main.rs")].contains(&42));
    }

    fn diff_line_strategy() -> impl Strategy<Value = DiffLine> {
        prop_oneof![
            any_line().prop_map(DiffLine::Context),
            any_line().prop_map(DiffLine::Added),
            any_line().prop_map(DiffLine::Removed),
            Just(DiffLine::EmptyContext),
        ]
    }

    fn any_line() -> impl Strategy<Value = String> {
        "[[:alnum:]_ (){};.,/*-]{0,40}".prop_map(|line: String| line.replace('\n', ""))
    }

    proptest! {
        #[test]
        fn added_lines_are_recorded_in_new_file_coordinates(
            start in 1usize..500,
            lines in prop::collection::vec(diff_line_strategy(), 0..80),
        ) {
            let path = PathBuf::from("src/lib.rs");
            let mut diff = format!(
                "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,1 +{start},1 @@\n"
            );
            let mut expected = BTreeSet::new();
            let mut new_line = start;

            for line in lines {
                match line {
                    DiffLine::Context(text) => {
                        diff.push(' ');
                        diff.push_str(&text);
                        diff.push('\n');
                        new_line = new_line.saturating_add(1);
                    }
                    DiffLine::Added(text) => {
                        diff.push('+');
                        diff.push_str(&text);
                        diff.push('\n');
                        expected.insert(new_line);
                        new_line = new_line.saturating_add(1);
                    }
                    DiffLine::Removed(text) => {
                        diff.push('-');
                        diff.push_str(&text);
                        diff.push('\n');
                    }
                    DiffLine::EmptyContext => {
                        diff.push('\n');
                        new_line = new_line.saturating_add(1);
                    }
                }
            }

            let parsed = parse_unified_diff(&diff);
            let actual = parsed.changed_lines.get(&path).cloned().unwrap_or_default();
            prop_assert_eq!(actual, expected);
        }

        #[test]
        fn removed_only_hunks_still_track_the_changed_file(
            start in 1usize..500,
            removed in prop::collection::vec(any_line(), 1..80),
        ) {
            let path = PathBuf::from("src/lib.rs");
            let mut diff = format!(
                "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,1 +{start},0 @@\n"
            );
            for line in removed {
                diff.push('-');
                diff.push_str(&line);
                diff.push('\n');
            }

            let parsed = parse_unified_diff(&diff);
            let actual = parsed.changed_lines.get(&path).cloned().unwrap_or_default();

            prop_assert!(parsed.contains_file(&path));
            prop_assert_eq!(actual, BTreeSet::new());
        }
    }
}
