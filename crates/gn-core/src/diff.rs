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
