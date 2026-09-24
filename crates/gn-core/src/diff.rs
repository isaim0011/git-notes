use crate::note::Note;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HunkAnchor {
    pub file: String,
    pub line_start: u32,
    pub line_end: u32,
    pub commit: String,
}

impl HunkAnchor {
    pub fn matches_note(&self, note: &Note) -> bool {
        if note.commit != self.commit {
            return false;
        }
        if let Some(ref f) = note.file {
            if f != &self.file {
                return false;
            }
        }
        if let (Some(ns), Some(ne)) = (note.line_start, note.line_end) {
            // Checks if the hunk overlaps with the note
            if ne < self.line_start || ns > self.line_end {
                return false;
            }
        }
        true
    }
}

pub fn parse_diff_hunks(patch: &str) -> Vec<HunkAnchor> {
    let mut anchors = Vec::new();
    let mut current_file = String::new();
    let current_commit = "pending".to_string(); // In a real setup, this would be parsed or passed in

    for line in patch.lines() {
        if let Some(f) = line.strip_prefix("+++ b/") {
            current_file = f.to_string();
        } else if line.starts_with("@@ ") {
            // Parse @@ -a,b +c,d @@
            if let Some(end_idx) = line[3..].find(" @@") {
                let hunk_info = &line[3..3 + end_idx];
                let parts: Vec<&str> = hunk_info.split_whitespace().collect();
                if parts.len() >= 2 {
                    let added = parts[1]; // e.g. +c,d
                    let added_nums = &added[1..];
                    let line_parts: Vec<&str> = added_nums.split(',').collect();
                    if !line_parts.is_empty() {
                        if let Ok(start) = line_parts[0].parse::<u32>() {
                            let count = if line_parts.len() > 1 {
                                line_parts[1].parse::<u32>().unwrap_or(1)
                            } else {
                                1
                            };
                            anchors.push(HunkAnchor {
                                file: current_file.clone(),
                                line_start: start,
                                line_end: start + count.saturating_sub(1),
                                commit: current_commit.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    anchors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::namespace::Namespace;

    #[test]
    fn test_parse_diff_hunks_single_file_and_hunk() {
        let patch = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..89abcde 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,5 +15,20 @@ fn main() {
 println!("hello");
"#;
        let anchors = parse_diff_hunks(patch);
        assert_eq!(anchors.len(), 1);
        assert_eq!(
            anchors[0],
            HunkAnchor {
                file: "src/main.rs".to_string(),
                line_start: 15,
                line_end: 34,
                commit: "pending".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_diff_hunks_multiple_files_and_hunks() {
        let patch = r#"diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,5 @@
 foo line 1
@@ -10,2 +20,4 @@
 foo line 2
diff --git a/bar.rs b/bar.rs
--- a/bar.rs
+++ b/bar.rs
@@ -5,10 +100,2 @@
 bar line 1
"#;
        let anchors = parse_diff_hunks(patch);
        assert_eq!(anchors.len(), 3);
        assert_eq!(
            anchors[0],
            HunkAnchor {
                file: "foo.rs".to_string(),
                line_start: 1,
                line_end: 5,
                commit: "pending".to_string(),
            }
        );
        assert_eq!(
            anchors[1],
            HunkAnchor {
                file: "foo.rs".to_string(),
                line_start: 20,
                line_end: 23,
                commit: "pending".to_string(),
            }
        );
        assert_eq!(
            anchors[2],
            HunkAnchor {
                file: "bar.rs".to_string(),
                line_start: 100,
                line_end: 101,
                commit: "pending".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_diff_hunks_default_line_count() {
        let patch = r#"--- a/file.rs
+++ b/file.rs
@@ -1 +10 @@
 context line
"#;
        let anchors = parse_diff_hunks(patch);
        assert_eq!(anchors.len(), 1);
        assert_eq!(
            anchors[0],
            HunkAnchor {
                file: "file.rs".to_string(),
                line_start: 10,
                line_end: 10,
                commit: "pending".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_diff_hunks_empty_and_malformed() {
        assert!(parse_diff_hunks("").is_empty());
        assert!(parse_diff_hunks("just some random text\nno diff headers").is_empty());
        assert!(parse_diff_hunks("+++ b/file.rs\nno hunk headers").is_empty());

        let malformed = r#"+++ b/file.rs
@@ malformed hunk line @@
@@ - + @@
@@ -1,2 +invalid @@
"#;
        assert!(parse_diff_hunks(malformed).is_empty());
    }

    #[test]
    fn test_hunk_anchor_matches_note() {
        let anchor = HunkAnchor {
            file: "src/lib.rs".to_string(),
            line_start: 10,
            line_end: 20,
            commit: "pending".to_string(),
        };

        let ns = Namespace::Comments;

        // 1. Note overlapping with hunk range
        let note_overlap = Note::new(
            "pending".to_string(),
            Some("src/lib.rs".to_string()),
            Some(15),
            Some(18),
            "Overlap note".to_string(),
            "Author".to_string(),
            ns.clone(),
        );
        assert!(anchor.matches_note(&note_overlap));

        // 2. Note before hunk range (no overlap)
        let note_before = Note::new(
            "pending".to_string(),
            Some("src/lib.rs".to_string()),
            Some(1),
            Some(5),
            "Before note".to_string(),
            "Author".to_string(),
            ns.clone(),
        );
        assert!(!anchor.matches_note(&note_before));

        // 3. Note after hunk range (no overlap)
        let note_after = Note::new(
            "pending".to_string(),
            Some("src/lib.rs".to_string()),
            Some(25),
            Some(30),
            "After note".to_string(),
            "Author".to_string(),
            ns.clone(),
        );
        assert!(!anchor.matches_note(&note_after));

        // 4. Note with no line numbers (matches whole file)
        let note_file_level = Note::new(
            "pending".to_string(),
            Some("src/lib.rs".to_string()),
            None,
            None,
            "File note".to_string(),
            "Author".to_string(),
            ns.clone(),
        );
        assert!(anchor.matches_note(&note_file_level));

        // 5. Note with different file
        let note_other_file = Note::new(
            "pending".to_string(),
            Some("src/other.rs".to_string()),
            Some(15),
            Some(18),
            "Other file note".to_string(),
            "Author".to_string(),
            ns.clone(),
        );
        assert!(!anchor.matches_note(&note_other_file));

        // 6. Note with different commit
        let note_other_commit = Note::new(
            "other_commit".to_string(),
            Some("src/lib.rs".to_string()),
            Some(15),
            Some(18),
            "Other commit note".to_string(),
            "Author".to_string(),
            ns.clone(),
        );
        assert!(!anchor.matches_note(&note_other_commit));

        // 7. Note with None file (matches any file)
        let note_no_file = Note::new(
            "pending".to_string(),
            None,
            Some(15),
            Some(18),
            "No file note".to_string(),
            "Author".to_string(),
            ns,
        );
        assert!(anchor.matches_note(&note_no_file));
    }
}
