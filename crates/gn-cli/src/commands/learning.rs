use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UserBehaviorProfile {
    /// Command usage frequencies: "a" -> 42, "l" -> 118, etc.
    pub command_counts: HashMap<String, u64>,
    /// Most frequent authors replied to
    pub favorite_authors: HashMap<String, u64>,
    /// Most frequently noted file paths
    pub file_frequencies: HashMap<String, u64>,
    /// Last active namespace (e.g., "comments", "review")
    pub preferred_namespace: Option<String>,
    /// Total commands run
    pub total_interactions: u64,
}

impl UserBehaviorProfile {
    fn profile_path() -> Option<PathBuf> {
        dirs_fallback().map(|p| p.join("profile.json"))
    }

    /// Load or initialize user behavior model
    pub fn load() -> Self {
        Self::profile_path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save state
    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::profile_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let data = serde_json::to_string_pretty(self)?;
            fs::write(path, data)?;
        }
        Ok(())
    }

    /// Learn from a command execution
    pub fn record_interaction(&mut self, cmd: &str, file: Option<&str>, ns: Option<&str>) {
        *self.command_counts.entry(cmd.to_string()).or_insert(0) += 1;
        self.total_interactions += 1;

        if let Some(f) = file {
            *self.file_frequencies.entry(f.to_string()).or_insert(0) += 1;
        }

        if let Some(n) = ns {
            self.preferred_namespace = Some(n.to_string());
        }

        let _ = self.save();
    }

    /// Smart contextual proactive suggestion based on learned history
    pub fn suggest_next_action(&self, current_file: Option<&str>) -> Option<String> {
        // If user frequently reviews or has files they frequently comment on
        if let Some(f) = current_file {
            if let Some(count) = self.file_frequencies.get(f) {
                if *count > 3 {
                    return Some(format!(
                        "\x1b[90m💡 Pro-tip: You frequently annotate '{}'. Run \x1b[36mgn d\x1b[90m to view inline diff notes.\x1b[0m",
                        f
                    ));
                }
            }
        }

        // Shortcut suggestion if user types long commands
        let list_count = self.command_counts.get("list").copied().unwrap_or(0);
        let l_count = self.command_counts.get("l").copied().unwrap_or(0);
        if list_count > 3 && l_count == 0 {
            return Some("\x1b[90m💡 Shortcut tip: Type \x1b[36mgn l\x1b[90m instead of 'git-notes list' to save keystrokes.\x1b[0m".to_string());
        }

        None
    }
}

fn dirs_fallback() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|h| PathBuf::from(h).join(".git-notes"))
}
