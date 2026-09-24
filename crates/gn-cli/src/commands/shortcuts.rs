use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ShortcutsArgs {
    /// Show all quickies, CLI abbreviations, and keyboard shortcuts
    #[arg(short, long)]
    pub list: bool,

    /// Bind or customize a shortcut (format: KEY=COMMAND, e.g. "c=add -m")
    #[arg(short, long)]
    pub set: Option<String>,

    /// Reset shortcuts and abbreviations to default configuration
    #[arg(long)]
    pub reset: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShortcutConfig {
    pub cli_aliases: BTreeMap<String, String>,
    pub tui_keybindings: BTreeMap<String, String>,
    pub custom_user_shortcuts: BTreeMap<String, String>,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        let mut cli_aliases = BTreeMap::new();
        cli_aliases.insert("gn a".into(), "Add a new note to file and line range [alias: add]".into());
        cli_aliases.insert("gn r".into(), "Reply to an existing note thread by ID or number [alias: reply]".into());
        cli_aliases.insert("gn l / gn ls".into(), "List notes with numbered quick-indexes [alias: list]".into());
        cli_aliases.insert("gn s".into(), "Interactive arrow-key note thread picker [alias: show]".into());
        cli_aliases.insert("gn ok / gn close".into(), "Resolve or approve a note by index # or ID [alias: resolve]".into());
        cli_aliases.insert("gn d".into(), "View git diff with inline notes attached to hunks [alias: diff]".into());
        cli_aliases.insert("gn b".into(), "View git blame with inline notes annotations [alias: blame]".into());
        cli_aliases.insert("gn push / gn pull".into(), "Sync notes bidirectionally with remote refs/notes/* [alias: sync]".into());
        cli_aliases.insert("gn sum".into(), "AI summary of open discussion threads via Gemini [alias: summarize]".into());
        cli_aliases.insert("gn doc".into(), "Check repo health, refspecs, and stale notes [alias: doctor]".into());
        cli_aliases.insert("gn i".into(), "1-second setup: configure fetch refspec & auto-sync hooks [alias: init]".into());

        let mut tui_keybindings = BTreeMap::new();
        tui_keybindings.insert("↑ / ↓ or k / j".into(), "Navigate files and notes list".into());
        tui_keybindings.insert("Enter / →".into(), "Expand file / focus diff & note thread panel".into());
        tui_keybindings.insert("Esc / ←".into(), "Navigate back to previous pane".into());
        tui_keybindings.insert("r".into(), "Open quick reply bar for highlighted thread".into());
        tui_keybindings.insert("a / ok".into(), "Mark highlighted note as Approved / Resolved".into());
        tui_keybindings.insert("q".into(), "Quit git-notes TUI".into());

        let custom_user_shortcuts = BTreeMap::new();

        Self {
            cli_aliases,
            tui_keybindings,
            custom_user_shortcuts,
        }
    }
}

impl ShortcutConfig {
    fn config_path() -> Option<PathBuf> {
        dirs_fallback().map(|p| p.join("shortcuts.json"))
    }

    pub fn load() -> Self {
        Self::config_path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let data = serde_json::to_string_pretty(self)?;
            fs::write(path, data)?;
        }
        Ok(())
    }
}

pub fn run(args: &ShortcutsArgs) -> Result<()> {
    let mut config = ShortcutConfig::load();

    if args.reset {
        config = ShortcutConfig::default();
        config.save()?;
        println!("\x1b[32m✓ Reset all shortcuts and quickies to default configuration.\x1b[0m");
        return Ok(());
    }

    if let Some(ref set_binding) = args.set {
        if let Some((key, val)) = set_binding.split_once('=') {
            config
                .custom_user_shortcuts
                .insert(key.trim().to_string(), val.trim().to_string());
            config.save()?;
            println!(
                "\x1b[32m✓ Saved custom shortcut:\x1b[0m \x1b[36m{}\x1b[0m ➜ \x1b[1m{}\x1b[0m",
                key.trim(),
                val.trim()
            );
            return Ok(());
        } else {
            eprintln!("\x1b[31m✗ Invalid format. Use --set KEY=COMMAND (e.g., gn shortcuts --set c=\"add -m\")\x1b[0m");
            return Ok(());
        }
    }

    println!("\n\x1b[1;36m⚡ git-notes Quickies & Keyboard Shortcuts Cheat Sheet\x1b[0m\n");

    println!("\x1b[1;33mCLI Quickies & Abbreviations (gn):\x1b[0m");
    println!("┌───────────────────────┬────────────────────────────────────────────────────────────────┐");
    println!("│ Quickie / Command     │ Action Description                                             │");
    println!("├───────────────────────┼────────────────────────────────────────────────────────────────┤");
    for (k, v) in &config.cli_aliases {
        println!("│ \x1b[36m{:<21}\x1b[0m │ {:<62} │", k, v);
    }
    println!("└───────────────────────┴────────────────────────────────────────────────────────────────┘");

    println!("\n\x1b[1;33mTUI & Terminal Keybindings:\x1b[0m");
    println!("┌───────────────────────┬────────────────────────────────────────────────────────────────┐");
    println!("│ Key / Keystroke       │ TUI Navigation & Review Action                                 │");
    println!("├───────────────────────┼────────────────────────────────────────────────────────────────┤");
    for (k, v) in &config.tui_keybindings {
        println!("│ \x1b[35m{:<21}\x1b[0m │ {:<62} │", k, v);
    }
    println!("└───────────────────────┴────────────────────────────────────────────────────────────────┘");

    if !config.custom_user_shortcuts.is_empty() {
        println!("\n\x1b[1;33mCustom User Shortcuts (~/.git-notes/shortcuts.json):\x1b[0m");
        for (k, v) in &config.custom_user_shortcuts {
            println!("  • \x1b[36m{}\x1b[0m ➜ {}", k, v);
        }
    } else {
        println!("\n\x1b[90m💡 Customize shortcuts anytime: \x1b[36mgn shortcuts --set <alias>=<command>\x1b[0m");
    }
    println!();

    Ok(())
}

fn dirs_fallback() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|h| PathBuf::from(h).join(".git-notes"))
}
