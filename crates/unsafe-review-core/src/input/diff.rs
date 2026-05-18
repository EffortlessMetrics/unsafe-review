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
    fn indexes_added_lines_in_new_file_coordinates() {
        let diff = r"diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -10,4 +10,5 @@ fn demo() {
 context();
-removed();
+added_one();
 unchanged();
+added_two();
";

        let index = parse_unified_diff(diff);
        let path = PathBuf::from("src/lib.rs");

        assert!(!index.is_empty());
        assert!(index.contains_file(&path));
        assert_eq!(
            index.changed_lines.get(&path).cloned().unwrap_or_default(),
            BTreeSet::from([11, 13])
        );
    }

    #[test]
    fn supports_single_line_hunks_without_counts() {
        let diff = "diff --git a/src/main.rs b/src/main.rs\n\
--- a/src/main.rs\n\
+++ b/src/main.rs\n\
@@ -1 +7 @@\n\
-old();\n\
+new();\n";

        let index = parse_unified_diff(diff);
        let path = PathBuf::from("src/main.rs");

        assert_eq!(
            index.changed_lines.get(&path).cloned().unwrap_or_default(),
            BTreeSet::from([7])
        );
    }

    #[test]
    fn contains_near_uses_six_line_review_window() {
        let diff = "diff --git a/src/lib.rs b/src/lib.rs\n\
--- a/src/lib.rs\n\
+++ b/src/lib.rs\n\
@@ -20,0 +21,1 @@\n\
+unsafe { core::ptr::read(ptr) };\n";

        let index = parse_unified_diff(diff);
        let path = PathBuf::from("src/lib.rs");

        assert!(index.contains_near(&path, 15));
        assert!(index.contains_near(&path, 27));
        assert!(!index.contains_near(&path, 14));
        assert!(!index.contains_near(&path, 28));
        assert!(!index.contains_near(&PathBuf::from("src/other.rs"), 21));
    }
}
