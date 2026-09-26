use anyhow::{bail, Context, Result};
use clap::{Args, CommandFactory};
use clap_complete::{generate, Shell};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Args, Debug, Clone)]
pub struct CompletionsArgs {
    /// Shell to generate completions for (bash, zsh, fish, powershell, elvish)
    #[arg(value_enum)]
    pub shell: Option<Shell>,

    /// Automatically install completions into standard shell configuration directory
    #[arg(long)]
    pub install: bool,
}

pub fn run(args: &CompletionsArgs) -> Result<()> {
    if args.install {
        install_completions(args.shell)?;
    } else if let Some(shell) = args.shell {
        let mut cmd = crate::Cli::command();
        // Generate completions for git-notes and gn binary
        generate(shell, &mut cmd, "git-notes", &mut io::stdout());
        let mut cmd_gn = crate::Cli::command();
        generate(shell, &mut cmd_gn, "gn", &mut io::stdout());
    } else {
        bail!("Please specify a shell (bash, zsh, fish, powershell, elvish) or use --install");
    }
    Ok(())
}

/// Detect user's current shell from environment variables
pub fn detect_shell() -> Option<Shell> {
    if let Ok(shell_env) = std::env::var("SHELL") {
        let lower = shell_env.to_lowercase();
        if lower.contains("zsh") {
            return Some(Shell::Zsh);
        } else if lower.contains("fish") {
            return Some(Shell::Fish);
        } else if lower.contains("bash") {
            return Some(Shell::Bash);
        } else if lower.contains("elvish") {
            return Some(Shell::Elvish);
        }
    }

    #[cfg(windows)]
    {
        if std::env::var_os("PSModulePath").is_some() {
            return Some(Shell::PowerShell);
        }
    }

    None
}

/// Helper to get home directory across Linux, macOS, and Windows
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

/// Generate completion script bytes for both `git-notes` and `gn`
pub fn generate_completion_script(shell: Shell) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut cmd_git_notes = crate::Cli::command();
    generate(shell, &mut cmd_git_notes, "git-notes", &mut buf);
    buf.push(b'\n');
    let mut cmd_gn = crate::Cli::command();
    generate(shell, &mut cmd_gn, "gn", &mut buf);
    buf
}

/// Automatically detect shell and install completion script
pub fn install_completions(explicit_shell: Option<Shell>) -> Result<()> {
    let shell = match explicit_shell.or_else(detect_shell) {
        Some(s) => s,
        None => {
            #[cfg(windows)]
            {
                Shell::PowerShell
            }
            #[cfg(not(windows))]
            {
                bail!("Could not automatically detect shell. Please specify one explicitly, e.g.: gn completions zsh --install");
            }
        }
    };

    let home = home_dir().context("Failed to determine user home directory")?;
    let script_bytes = generate_completion_script(shell);

    match shell {
        Shell::Zsh => {
            // ~/.zsh/completions/_git-notes and _gn
            let zsh_comp_dir = home.join(".zsh").join("completions");
            fs::create_dir_all(&zsh_comp_dir)
                .with_context(|| format!("Failed to create directory {}", zsh_comp_dir.display()))?;

            let mut gn_buf = Vec::new();
            let mut cmd_gn = crate::Cli::command();
            generate(Shell::Zsh, &mut cmd_gn, "gn", &mut gn_buf);
            let gn_path = zsh_comp_dir.join("_gn");
            fs::write(&gn_path, gn_buf)?;

            let mut git_notes_buf = Vec::new();
            let mut cmd_git_notes = crate::Cli::command();
            generate(Shell::Zsh, &mut cmd_git_notes, "git-notes", &mut git_notes_buf);
            let git_notes_path = zsh_comp_dir.join("_git-notes");
            fs::write(&git_notes_path, git_notes_buf)?;

            println!("\x1b[32m✓ Installed Zsh completions:\x1b[0m");
            println!("  • {}", gn_path.display());
            println!("  • {}", git_notes_path.display());
            println!("\n\x1b[90mEnsure your ~/.zshrc contains:\x1b[0m");
            println!("  fpath=(~/.zsh/completions $fpath)");
            println!("  autoload -Uz compinit && compinit");
        }
        Shell::Bash => {
            // ~/.bash_completion.d/gn or ~/.local/share/bash-completion/completions/gn
            let bash_comp_dir = home.join(".bash_completion.d");
            fs::create_dir_all(&bash_comp_dir)
                .with_context(|| format!("Failed to create directory {}", bash_comp_dir.display()))?;
            let target_path = bash_comp_dir.join("gn");
            fs::write(&target_path, script_bytes)?;

            println!("\x1b[32m✓ Installed Bash completions:\x1b[0m");
            println!("  • {}", target_path.display());
            println!("\n\x1b[90mEnsure ~/.bashrc sources ~/.bash_completion.d:\x1b[0m");
            println!("  for f in ~/.bash_completion.d/*; do [[ -f \"$f\" ]] && source \"$f\"; done");
        }
        Shell::Fish => {
            // ~/.config/fish/completions/gn.fish & git-notes.fish
            let fish_comp_dir = home.join(".config").join("fish").join("completions");
            fs::create_dir_all(&fish_comp_dir)
                .with_context(|| format!("Failed to create directory {}", fish_comp_dir.display()))?;

            let mut gn_buf = Vec::new();
            let mut cmd_gn = crate::Cli::command();
            generate(Shell::Fish, &mut cmd_gn, "gn", &mut gn_buf);
            let gn_path = fish_comp_dir.join("gn.fish");
            fs::write(&gn_path, gn_buf)?;

            let mut git_notes_buf = Vec::new();
            let mut cmd_git_notes = crate::Cli::command();
            generate(Shell::Fish, &mut cmd_git_notes, "git-notes", &mut git_notes_buf);
            let git_notes_path = fish_comp_dir.join("git-notes.fish");
            fs::write(&git_notes_path, git_notes_buf)?;

            println!("\x1b[32m✓ Installed Fish completions:\x1b[0m");
            println!("  • {}", gn_path.display());
            println!("  • {}", git_notes_path.display());
        }
        Shell::PowerShell => {
            // Append to PowerShell Profile or create ~/.git-notes/completions.ps1 and dot-source in profile
            let profile_path = get_powershell_profile_path(&home);
            if let Some(parent) = profile_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let comp_dir = home.join(".git-notes").join("completions");
            fs::create_dir_all(&comp_dir)?;
            let script_path = comp_dir.join("gn-completions.ps1");
            fs::write(&script_path, &script_bytes)?;

            let source_cmd = format!("\n. \"{}\"\n", script_path.display());
            let current_content = fs::read_to_string(&profile_path).unwrap_or_default();
            if !current_content.contains("gn-completions.ps1") {
                let mut new_content = current_content;
                new_content.push_str(&source_cmd);
                fs::write(&profile_path, new_content)?;
            }

            println!("\x1b[32m✓ Installed PowerShell completions:\x1b[0m");
            println!("  • Script: {}", script_path.display());
            println!("  • Profile: {}", profile_path.display());
        }
        Shell::Elvish => {
            // ~/.elvish/lib/gn.elv
            let elvish_comp_dir = home.join(".elvish").join("lib");
            fs::create_dir_all(&elvish_comp_dir)
                .with_context(|| format!("Failed to create directory {}", elvish_comp_dir.display()))?;
            let target_path = elvish_comp_dir.join("gn.elv");
            fs::write(&target_path, script_bytes)?;

            println!("\x1b[32m✓ Installed Elvish completions:\x1b[0m");
            println!("  • {}", target_path.display());
        }
        _ => {
            bail!("Automated installation not yet supported for {:?}. Please generate output directly with: gn completions {:?}", shell, shell);
        }
    }

    Ok(())
}

fn get_powershell_profile_path(home: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        // Check Documents\PowerShell\Microsoft.PowerShell_profile.ps1 or Documents\WindowsPowerShell\...
        let docs = home.join("Documents");
        let ps7 = docs.join("PowerShell").join("Microsoft.PowerShell_profile.ps1");
        if ps7.exists() || !docs.join("WindowsPowerShell").exists() {
            ps7
        } else {
            docs.join("WindowsPowerShell").join("Microsoft.PowerShell_profile.ps1")
        }
    }
    #[cfg(not(windows))]
    {
        home.join(".config").join("powershell").join("Microsoft.PowerShell_profile.ps1")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_completion_script() {
        let bash_script = generate_completion_script(Shell::Bash);
        assert!(!bash_script.is_empty());
        let bash_str = String::from_utf8_lossy(&bash_script);
        assert!(bash_str.contains("git-notes"));
        assert!(bash_str.contains("gn"));

        let zsh_script = generate_completion_script(Shell::Zsh);
        assert!(!zsh_script.is_empty());

        let fish_script = generate_completion_script(Shell::Fish);
        assert!(!fish_script.is_empty());

        let ps_script = generate_completion_script(Shell::PowerShell);
        assert!(!ps_script.is_empty());

        let elvish_script = generate_completion_script(Shell::Elvish);
        assert!(!elvish_script.is_empty());
    }
}
