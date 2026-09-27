use anyhow::Result;
use clap::Args;
use gn_core::{DataValidator, Namespace};
use std::env;

#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Namespace to validate (comments, review, todos, or custom)
    #[arg(short, long)]
    pub namespace: Option<String>,

    /// Fail with exit code 1 if any orphan notes (anchoring to missing commits) are found
    #[arg(long, default_value_t = false)]
    pub strict: bool,

    /// Output results as structured JSON
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

pub fn run(args: &ValidateArgs) -> Result<()> {
    let repo_path = env::current_dir()?;
    let validator = DataValidator::new(&repo_path);

    let namespaces = if let Some(ns_str) = &args.namespace {
        vec![Namespace::from_str(ns_str)]
    } else {
        vec![
            Namespace::Comments,
            Namespace::Review,
            Namespace::Todos,
        ]
    };

    let report = validator.validate_all(&namespaces)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        if !report.healthy || (args.strict && report.total_orphans > 0) {
            std::process::exit(1);
        }
        return Ok(());
    }

    println!("\x1b[1;36m═══════════════════════════════════════════════════════════════════════\x1b[0m");
    println!("\x1b[1;36m           git-notes Data & Refspec Integrity Validator (fsck)          \x1b[0m");
    println!("\x1b[1;36m═══════════════════════════════════════════════════════════════════════\x1b[0m");
    println!("Repository: \x1b[1m{}\x1b[0m\n", report.repo_root);

    for ref_rep in &report.ref_reports {
        if !ref_rep.exists {
            println!("  \x1b[90m○ {:<24} (ref not yet created / 0 notes)\x1b[0m", ref_rep.ref_path);
            continue;
        }

        let status_badge = if ref_rep.corrupt_blobs == 0 && ref_rep.unparseable_notes == 0 {
            if ref_rep.orphan_notes == 0 {
                "\x1b[32m✔ OK\x1b[0m"
            } else {
                "\x1b[33m⚠ ORPHANS\x1b[0m"
            }
        } else {
            "\x1b[31m✗ CORRUPT\x1b[0m"
        };

        println!(
            "  [{}] \x1b[1m{:<24}\x1b[0m (commit: \x1b[90m{}\x1b[0m | {} note{})",
            status_badge,
            ref_rep.ref_path,
            ref_rep.commit_sha.as_deref().unwrap_or("none").chars().take(8).collect::<String>(),
            ref_rep.note_count,
            if ref_rep.note_count == 1 { "" } else { "s" }
        );

        for issue in &ref_rep.issues {
            if issue.contains("unreachable") || issue.contains("missing commit") {
                println!("      \x1b[33m⚠ {}\x1b[0m", issue);
            } else {
                println!("      \x1b[31m✗ {}\x1b[0m", issue);
            }
        }
    }

    println!("\n\x1b[1;36m───────────────────────────────────────────────────────────────────────\x1b[0m");
    println!(
        "Summary: {} note(s) scanned across {} ref(s). Corrupt: {}, Orphans: {}",
        report.total_notes,
        report.total_refs,
        report.total_corrupt,
        report.total_orphans
    );

    if !report.healthy {
        println!("\x1b[1;31m✗ Data corruption detected in git note references!\x1b[0m");
        anyhow::bail!("Data validation failed: {} corrupt blob(s) detected.", report.total_corrupt);
    } else if report.total_orphans > 0 {
        println!("\x1b[33m💡 Tip: Run 'gn heal' to automatically re-anchor orphan notes after rebases.\x1b[0m");
        if args.strict {
            anyhow::bail!("Strict mode: {} orphan note(s) found.", report.total_orphans);
        }
    } else {
        println!("\x1b[1;32m✔ All git-notes refs and JSON schemas are 100% valid and verified.\x1b[0m");
    }

    Ok(())
}
