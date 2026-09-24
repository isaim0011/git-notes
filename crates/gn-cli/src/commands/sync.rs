use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use gn_core::{LwwStrategy, Namespace};
use std::path::PathBuf;
use std::process::Command;

#[derive(Args, Debug, Clone)]
pub struct SyncArgs {
    #[command(subcommand)]
    pub action: Option<SyncAction>,

    /// Quick flag for P2P sync (e.g. `gn sync --p2p`)
    #[arg(long)]
    pub p2p: bool,

    /// Remote name for git push/pull (default: origin)
    #[arg(short, long, default_value = "origin")]
    pub remote: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SyncAction {
    /// Push notes to remote
    Push,
    /// Pull notes from remote
    Pull,
    /// Sync bidirectionally with remote
    Auto,
    /// Export or import notes via git bundle for offline / USB / AirDrop transfer
    Bundle(BundleArgs),
    /// Local LAN P2P git notes sync without internet
    P2p(P2pArgs),
}

#[derive(Args, Debug, Clone)]
pub struct BundleArgs {
    #[command(subcommand)]
    pub command: BundleCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum BundleCommand {
    /// Create a portable git bundle containing all refs/notes/* for transfer over USB or AirDrop
    Export {
        /// Destination path for the .bundle file
        output: PathBuf,
    },
    /// Fetch and merge notes from a portable .bundle file
    Import {
        /// Path to the .bundle file to import
        input: PathBuf,
    },
}

#[derive(Args, Debug, Clone)]
pub struct P2pArgs {
    #[command(subcommand)]
    pub command: Option<P2pCommand>,

    /// Port to listen on (default: 9418)
    #[arg(short, long, default_value_t = 9418)]
    pub port: u16,
}

#[derive(Subcommand, Debug, Clone)]
pub enum P2pCommand {
    /// Start a lightweight local P2P git notes sync server on the local LAN
    Serve {
        /// Port to listen on (default: 9418)
        #[arg(short, long, default_value_t = 9418)]
        port: u16,
    },
    /// Pull and merge notes directly from a peer developer's IP on the local Wi-Fi
    Connect {
        /// Peer address in format <ip:port> or <ip> (default port: 9418)
        peer: String,
    },
}

fn get_repo_root() -> Result<PathBuf> {
    let root_out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to check git repository")?;

    if !root_out.status.success() {
        anyhow::bail!("Not inside a Git repository. Run 'git init' first.");
    }

    let repo_root_str = String::from_utf8_lossy(&root_out.stdout).trim().to_string();
    Ok(PathBuf::from(repo_root_str))
}

pub fn run(args: &SyncArgs) -> Result<()> {
    let repo_root = get_repo_root()?;

    // Check --p2p flag first
    if args.p2p {
        return gn_sync::serve_p2p(&repo_root, 9418);
    }

    let default_action = SyncAction::Auto;
    let action = args.action.as_ref().unwrap_or(&default_action);

    match action {
        SyncAction::Bundle(bundle_args) => match &bundle_args.command {
            BundleCommand::Export { output } => {
                let report = gn_sync::export_bundle(&repo_root, output)?;
                println!(
                    "\n\x1b[32m✔ Exported {} notes refs to git bundle:\x1b[0m {}",
                    report.refs_count,
                    report.path.display()
                );
                println!("  Transfer this bundle file to teammates via USB, AirDrop, or shared drive.");
                println!("  Teammates can import it with: \x1b[36mgn sync bundle import <path>\x1b[0m\n");
                Ok(())
            }
            BundleCommand::Import { input } => {
                let strategy = LwwStrategy;
                let report = gn_sync::import_bundle(&repo_root, input, &strategy)?;
                println!(
                    "\n\x1b[32m✔ Successfully imported git bundle:\x1b[0m {}",
                    report.path.display()
                );
                println!("  Fetched: {} notes across {} refs", report.fetched, report.refs_count);
                println!("  Merged:  {} notes (conflict-free LWW)\n", report.merged);
                Ok(())
            }
        },
        SyncAction::P2p(p2p_args) => {
            let default_cmd = P2pCommand::Serve {
                port: p2p_args.port,
            };
            let cmd = p2p_args.command.as_ref().unwrap_or(&default_cmd);
            match cmd {
                P2pCommand::Serve { port } => gn_sync::serve_p2p(&repo_root, *port),
                P2pCommand::Connect { peer } => {
                    let strategy = LwwStrategy;
                    let report = gn_sync::connect_peer(&repo_root, peer, &strategy)?;
                    println!(
                        "\n\x1b[32m✔ Successfully synced with peer {}\x1b[0m",
                        report.peer
                    );
                    println!("  Fetched: {} notes", report.fetched);
                    println!("  Merged:  {} notes\n", report.merged);
                    Ok(())
                }
            }
        }
        SyncAction::Push => {
            let namespaces = vec![
                Namespace::Comments,
                Namespace::Review,
                Namespace::Todos,
            ];
            gn_sync::push::push_notes(&repo_root, &args.remote, &namespaces)?;
            println!("\x1b[32m✔ Pushed notes to {}\x1b[0m", args.remote);
            Ok(())
        }
        SyncAction::Pull => {
            let namespaces = vec![
                Namespace::Comments,
                Namespace::Review,
                Namespace::Todos,
            ];
            let strategy = LwwStrategy;
            let report = gn_sync::fetch::fetch_notes(&repo_root, &args.remote, &namespaces, &strategy)?;
            println!(
                "\x1b[32m✔ Pulled notes from {}: fetched={} merged={} conflicts={}\x1b[0m",
                args.remote, report.fetched, report.merged, report.conflicts
            );
            Ok(())
        }
        SyncAction::Auto => {
            let namespaces = vec![
                Namespace::Comments,
                Namespace::Review,
                Namespace::Todos,
            ];
            let strategy = LwwStrategy;
            let report = gn_sync::fetch::fetch_notes(&repo_root, &args.remote, &namespaces, &strategy)?;
            let _ = gn_sync::push::push_notes(&repo_root, &args.remote, &namespaces);
            println!(
                "✓ Synced: fetched={} merged={} conflicts={}",
                report.fetched, report.merged, report.conflicts
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        cmd: TestSub,
    }

    #[derive(Subcommand)]
    enum TestSub {
        Sync(SyncArgs),
    }

    #[test]
    fn test_sync_cli_bundle_export_parsing() {
        let cli = TestCli::try_parse_from(["app", "sync", "bundle", "export", "my_notes.bundle"]).unwrap();
        match cli.cmd {
            TestSub::Sync(args) => match args.action.unwrap() {
                SyncAction::Bundle(b) => match b.command {
                    BundleCommand::Export { output } => {
                        assert_eq!(output, PathBuf::from("my_notes.bundle"));
                    }
                    _ => panic!("Expected Export"),
                },
                _ => panic!("Expected Bundle"),
            },
        }
    }

    #[test]
    fn test_sync_cli_bundle_import_parsing() {
        let cli = TestCli::try_parse_from(["app", "sync", "bundle", "import", "my_notes.bundle"]).unwrap();
        match cli.cmd {
            TestSub::Sync(args) => match args.action.unwrap() {
                SyncAction::Bundle(b) => match b.command {
                    BundleCommand::Import { input } => {
                        assert_eq!(input, PathBuf::from("my_notes.bundle"));
                    }
                    _ => panic!("Expected Import"),
                },
                _ => panic!("Expected Bundle"),
            },
        }
    }

    #[test]
    fn test_sync_cli_p2p_serve_parsing() {
        let cli = TestCli::try_parse_from(["app", "sync", "p2p", "serve", "--port", "9999"]).unwrap();
        match cli.cmd {
            TestSub::Sync(args) => match args.action.unwrap() {
                SyncAction::P2p(p) => match p.command.unwrap() {
                    P2pCommand::Serve { port } => {
                        assert_eq!(port, 9999);
                    }
                    _ => panic!("Expected Serve"),
                },
                _ => panic!("Expected P2p"),
            },
        }
    }

    #[test]
    fn test_sync_cli_p2p_connect_parsing() {
        let cli = TestCli::try_parse_from(["app", "sync", "p2p", "connect", "192.168.1.100:9418"]).unwrap();
        match cli.cmd {
            TestSub::Sync(args) => match args.action.unwrap() {
                SyncAction::P2p(p) => match p.command.unwrap() {
                    P2pCommand::Connect { peer } => {
                        assert_eq!(peer, "192.168.1.100:9418");
                    }
                    _ => panic!("Expected Connect"),
                },
                _ => panic!("Expected P2p"),
            },
        }
    }
}
