use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

pub mod commands;

/// git-notes CLI — decentralized code comments, inline reviews, and discussions
#[derive(Parser)]
#[command(name = "git-notes", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new note [alias: a]
    #[command(alias = "a")]
    Add(commands::add::AddArgs),

    /// Reply to an existing note thread [alias: r]
    #[command(alias = "r")]
    Reply(commands::reply::ReplyArgs),

    /// List notes [aliases: l, ls]
    #[command(alias = "l", alias = "ls")]
    List(commands::list::ListArgs),

    /// Show a specific note or thread [alias: s]
    #[command(alias = "s")]
    Show(commands::show::ShowArgs),

    /// Sync notes with remote [aliases: push, pull]
    #[command(alias = "push", alias = "pull")]
    Sync(commands::sync::SyncArgs),

    /// Resolve a note [aliases: ok, close]
    #[command(alias = "ok", alias = "close")]
    Resolve(commands::resolve::ResolveArgs),

    /// Export notes [alias: exp]
    #[command(alias = "exp")]
    Export(commands::export::ExportArgs),

    /// Check repository health and configuration [alias: doc]
    #[command(alias = "doc")]
    Doctor,

    /// View git blame with inline notes [alias: b]
    #[command(alias = "b")]
    Blame(commands::blame::BlameArgs),

    /// Import review comments from a GitHub Pull Request [alias: pr]
    #[command(alias = "pr")]
    ImportPr(commands::import_pr::ImportPrArgs),

    /// AI-powered summary of open discussion threads (Gemini) [alias: sum]
    #[command(alias = "sum")]
    Summarize(commands::summarize::SummarizeArgs),

    /// Setup git hooks and remote tracking refspec in 1 second [alias: i]
    #[command(alias = "i")]
    Init,

    /// Manage git auto-sync hooks
    Hook(commands::hook::HookArgs),

    /// View diff with inline notes attached to hunks [alias: d]
    #[command(alias = "d")]
    Diff(commands::diff::DiffArgs),

    /// View and customize quickies, CLI abbreviations, and keyboard shortcuts [aliases: shortcut, keys, quickies]
    #[command(alias = "shortcut", alias = "keys", alias = "quickies")]
    Shortcuts(commands::shortcuts::ShortcutsArgs),

    /// Generate shell completions
    Completions(commands::completions::CompletionsArgs),
}

pub fn run_cli() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let mut profile = commands::learning::UserBehaviorProfile::load();
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Add(args) => {
            profile.record_interaction("add", Some(&args.file), Some(&args.namespace));
            commands::add::run(args)
        }
        Commands::Reply(args) => {
            profile.record_interaction("reply", None, None);
            commands::reply::run(args)
        }
        Commands::List(args) => {
            profile.record_interaction("list", args.file.as_deref(), args.namespace.as_deref());
            commands::list::run(args)
        }
        Commands::Show(args) => {
            profile.record_interaction("show", None, None);
            commands::show::run(args)
        }
        Commands::Sync(args) => {
            profile.record_interaction("sync", None, None);
            commands::sync::run(args)
        }
        Commands::Resolve(args) => {
            profile.record_interaction("resolve", None, None);
            commands::resolve::run(args)
        }
        Commands::Export(args) => {
            profile.record_interaction("export", None, None);
            commands::export::run(args)
        }
        Commands::Doctor => {
            profile.record_interaction("doctor", None, None);
            commands::doctor::run()
        }
        Commands::Blame(args) => {
            profile.record_interaction("blame", Some(&args.file), Some(&args.namespace));
            commands::blame::run(args)
        }
        Commands::ImportPr(args) => {
            profile.record_interaction("import-pr", None, Some(&args.namespace));
            commands::import_pr::run(args)
        }
        Commands::Summarize(args) => {
            profile.record_interaction("summarize", None, Some(&args.namespace));
            commands::summarize::run(args)
        }
        Commands::Init => {
            profile.record_interaction("init", None, None);
            commands::init::run()
        }
        Commands::Hook(args) => {
            profile.record_interaction("hook", None, None);
            commands::hook::run(args)
        }
        Commands::Diff(args) => {
            profile.record_interaction("diff", None, Some(&args.namespace));
            commands::diff::run(args)
        }
        Commands::Shortcuts(args) => {
            profile.record_interaction("shortcuts", None, None);
            commands::shortcuts::run(args)
        }
        Commands::Completions(args) => commands::completions::run(args),
    };

    if let Some(tip) = profile.suggest_next_action(None) {
        println!("{}", tip);
    }

    result
}
