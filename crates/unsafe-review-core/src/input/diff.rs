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

    #[test]
    fn parses_added_lines_across_multiple_hunks() {
        let diff = parse_unified_diff(
            "diff --git a/src/lib.rs b/src/lib.rs\n\
--- a/src/lib.rs\n\
+++ b/src/lib.rs\n\
@@ -1,3 +1,4 @@\n\
 fn before() {}\n\
+fn added() {}\n\
 fn after() {}\n\
@@ -20,2 +21,3 @@\n\
 context();\n\
+changed();\n",
        );
        let path = PathBuf::from("src/lib.rs");

        assert!(diff.contains_file(&path));
        assert!(diff.contains_near(&path, 2));
        assert!(diff.contains_near(&path, 22));
        assert!(!diff.contains_near(&path, 40));
    }

    #[test]
    fn deleted_lines_do_not_advance_new_file_coordinate() {
        let diff = parse_unified_diff(concat!(
            "diff --git a/src/lib.rs b/src/lib.rs\n",
            "--- a/src/lib.rs\n",
            "+++ b/src/lib.rs\n",
            "@@ -10,3 +10,3 @@\n",
            " keep();\n",
            "-removed();\n",
            "+added();\n",
            " tail();\n",
        ));
        let path = PathBuf::from("src/lib.rs");
        let changed = diff.changed_lines.get(&path).cloned().unwrap_or_default();

        assert_eq!(changed, BTreeSet::from([11]));
    }

    #[test]
    fn parses_empty_added_file_index() {
        let diff = parse_unified_diff(
            "diff --git a/src/new.rs b/src/new.rs\n\
--- /dev/null\n\
+++ b/src/new.rs\n\
@@ -0,0 +1,2 @@\n\
+pub fn one() {}\n\
+pub fn two() {}\n",
        );
        let path = PathBuf::from("src/new.rs");
        let changed = diff.changed_lines.get(&path).cloned().unwrap_or_default();

        assert_eq!(changed, BTreeSet::from([1, 2]));
    }
}
