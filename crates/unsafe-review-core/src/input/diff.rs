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
    use std::path::PathBuf;

    #[test]
    fn parses_added_line_coordinates_across_hunks_and_deletions() {
        let diff = parse_unified_diff(
            "diff --git a/src/lib.rs b/src/lib.rs\n\
             --- a/src/lib.rs\n\
             +++ b/src/lib.rs\n\
             @@ -1,4 +10,5 @@\n\
              context\n\
             -old line\n\
             +new line\n\
              after\n\
             @@ -20,2 +30,3 @@\n\
              more context\n\
             +second add\n",
        );

        let path = PathBuf::from("src/lib.rs");
        assert!(diff.contains_file(&path));
        assert!(diff.contains_near(&path, 11));
        assert!(diff.contains_near(&path, 31));
        assert!(!diff.contains_near(&path, 23));
    }

    #[test]
    fn ignores_removed_only_files_until_new_file_path_is_seen() {
        let diff = parse_unified_diff(
            "diff --git a/src/old.rs b/src/old.rs\n\
             --- a/src/old.rs\n\
             @@ -1,2 +0,0 @@\n\
             -gone\n",
        );

        assert!(diff.is_empty());
        assert!(!diff.contains_file(&PathBuf::from("src/old.rs")));
    }
}
