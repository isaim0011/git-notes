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
    /// Add a new note
    Add(commands::add::AddArgs),
    /// Reply to an existing note thread
    Reply(commands::reply::ReplyArgs),
    /// List notes
    List(commands::list::ListArgs),
    /// Show a specific note
    Show(commands::show::ShowArgs),
    /// Sync notes with remote
    Sync(commands::sync::SyncArgs),
    /// Resolve a note
    Resolve(commands::resolve::ResolveArgs),
    /// Export notes
    Export(commands::export::ExportArgs),
    /// Check repository health and configuration
    Doctor,
    /// View git blame with inline notes
    Blame(commands::blame::BlameArgs),
    /// Import review comments from a GitHub Pull Request
    ImportPr(commands::import_pr::ImportPrArgs),
    /// AI-powered summary of open discussion threads (Gemini)
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
