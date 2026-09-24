use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod commands;

/// git-notes CLI — decentralized code comments, inline reviews, and discussions
#[derive(Parser)]
#[command(name = "git-notes", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

    /// Generate shell completions
    Completions(commands::completions::CompletionsArgs),
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Add(args) => commands::add::run(args),
        Commands::Reply(args) => commands::reply::run(args),
        Commands::List(args) => commands::list::run(args),
        Commands::Show(args) => commands::show::run(args),
        Commands::Sync(args) => commands::sync::run(args),
        Commands::Resolve(args) => commands::resolve::run(args),
        Commands::Export(args) => commands::export::run(args),
        Commands::Doctor => commands::doctor::run(),
        Commands::Blame(args) => commands::blame::run(args),
        Commands::ImportPr(args) => commands::import_pr::run(args),
        Commands::Summarize(args) => commands::summarize::run(args),
        Commands::Completions(args) => commands::completions::run(args),
    }
}
