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

    fn create_test_note(
        commit: &str,
        file: Option<&str>,
        line_start: Option<u32>,
        line_end: Option<u32>,
    ) -> Note {
        Note::new(
            commit.to_string(),
            file.map(|s| s.to_string()),
            line_start,
            line_end,
            "test body".to_string(),
            "Author <author@test.com>".to_string(),
            Namespace::Comments,
        )
    }

    #[test]
    fn test_matches_note_commit_mismatch() {
        let hunk = HunkAnchor {
            file: "src/main.rs".to_string(),
            line_start: 10,
            line_end: 20,
            commit: "commit_a".to_string(),
        };

        let note = create_test_note("commit_b", Some("src/main.rs"), Some(12), Some(15));
        assert!(!hunk.matches_note(&note));
    }

    #[test]
    fn test_matches_note_file_matching() {
        let hunk = HunkAnchor {
            file: "src/main.rs".to_string(),
            line_start: 10,
            line_end: 20,
            commit: "commit_a".to_string(),
        };

        // Different file -> no match
        let note_diff_file =
            create_test_note("commit_a", Some("src/lib.rs"), Some(12), Some(15));
        assert!(!hunk.matches_note(&note_diff_file));

        // Same file -> match
        let note_same_file =
            create_test_note("commit_a", Some("src/main.rs"), Some(12), Some(15));
        assert!(hunk.matches_note(&note_same_file));

        // Note file is None -> match (file-agnostic or repository level note)
        let note_no_file = create_test_note("commit_a", None, Some(12), Some(15));
        assert!(hunk.matches_note(&note_no_file));
    }

    #[test]
    fn test_matches_note_line_overlap_boundary_cases() {
        let hunk = HunkAnchor {
            file: "src/main.rs".to_string(),
            line_start: 10,
            line_end: 20,
            commit: "commit_a".to_string(),
        };

        // Case 1: Note entirely before hunk (5..9) -> false
        let note = create_test_note("commit_a", Some("src/main.rs"), Some(5), Some(9));
        assert!(!hunk.matches_note(&note));

        // Case 2: Note touches hunk start boundary (5..10) -> true
        let note = create_test_note("commit_a", Some("src/main.rs"), Some(5), Some(10));
        assert!(hunk.matches_note(&note));

        // Case 3: Note overlaps start boundary (5..15) -> true
        let note = create_test_note("commit_a", Some("src/main.rs"), Some(5), Some(15));
        assert!(hunk.matches_note(&note));

        // Case 4: Note strictly inside hunk (12..18) -> true
        let note = create_test_note("commit_a", Some("src/main.rs"), Some(12), Some(18));
        assert!(hunk.matches_note(&note));

        // Case 5: Note spans hunk entirely (5..25) -> true
        let note = create_test_note("commit_a", Some("src/main.rs"), Some(5), Some(25));
        assert!(hunk.matches_note(&note));

        // Case 6: Note overlaps end boundary (15..25) -> true
        let note = create_test_note("commit_a", Some("src/main.rs"), Some(15), Some(25));
        assert!(hunk.matches_note(&note));

        // Case 7: Note touches hunk end boundary (20..25) -> true
        let note = create_test_note("commit_a", Some("src/main.rs"), Some(20), Some(25));
        assert!(hunk.matches_note(&note));

        // Case 8: Note entirely after hunk (21..25) -> false
        let note = create_test_note("commit_a", Some("src/main.rs"), Some(21), Some(25));
        assert!(!hunk.matches_note(&note));
    }

    #[test]
    fn test_matches_note_partial_line_info() {
        let hunk = HunkAnchor {
            file: "src/main.rs".to_string(),
            line_start: 10,
            line_end: 20,
            commit: "commit_a".to_string(),
        };

        // Line start specified, line end missing -> match
        let note1 = create_test_note("commit_a", Some("src/main.rs"), Some(15), None);
        assert!(hunk.matches_note(&note1));

        // Line start missing, line end specified -> match
        let note2 = create_test_note("commit_a", Some("src/main.rs"), None, Some(15));
        assert!(hunk.matches_note(&note2));

        // Both missing -> match
        let note3 = create_test_note("commit_a", Some("src/main.rs"), None, None);
        assert!(hunk.matches_note(&note3));
    }

    #[test]
    fn test_parse_diff_hunks() {
        let diff = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..89abcdef 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,5 @@
 context
 context
+line1
+line2
 context
@@ -10 +20,3 @@
-old line
+new line1
+new line2
+new line3
"#;

        let anchors = parse_diff_hunks(diff);
        assert_eq!(anchors.len(), 2);
        assert_eq!(
            anchors[0],
            HunkAnchor {
                file: "src/main.rs".to_string(),
                line_start: 1,
                line_end: 5,
                commit: "pending".to_string(),
            }
        );
        assert_eq!(
            anchors[1],
            HunkAnchor {
                file: "src/main.rs".to_string(),
                line_start: 20,
                line_end: 22,
                commit: "pending".to_string(),
            }
        );
    }
}
