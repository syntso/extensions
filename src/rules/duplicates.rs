use std::collections::HashSet;

use crate::registry::Entry;

pub fn duplicates(entries: &[Entry]) -> Vec<String> {
    let mut ids = HashSet::new();
    let mut paths = HashSet::new();
    let mut problems = Vec::new();

    for entry in entries {
        if !ids.insert(&entry.id) {
            problems.push(format!("{}: listed more than once", entry.id));
        }

        if !paths.insert(&entry.path) {
            problems.push(format!(
                "{}: shares the path {}",
                entry.id,
                entry.path.display()
            ));
        }
    }

    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, path: &str) -> Entry {
        Entry {
            id: id.to_owned(),
            path: path.into(),
            version: "0.1.0".to_owned(),
        }
    }

    #[test]
    fn accepts_distinct_entries() {
        let entries = [
            entry("uptime", "extensions/uptime"),
            entry("disk", "extensions/disk"),
        ];

        assert!(duplicates(&entries).is_empty());
    }

    #[test]
    fn finds_repeated_ids_and_paths() {
        let entries = [
            entry("uptime", "extensions/uptime"),
            entry("uptime", "extensions/uptime"),
        ];

        assert_eq!(duplicates(&entries).len(), 2);
    }
}
