use anyhow::{anyhow, Result};
use clap::Args;
use gn_core::NotesEngine;
use super::signing::{verify_signature, SignatureStatus};

#[derive(Args, Debug, Clone)]
pub struct VerifyArgs {
    /// Note ID or quick-index to verify
    pub note_id: Option<String>,

    /// Verify all notes in the repository
    #[arg(short, long)]
    pub all: bool,

    /// Filter by namespace (e.g., "comments", "review")
    #[arg(short, long)]
    pub namespace: Option<String>,
}

pub fn run(args: &VerifyArgs) -> Result<()> {
    let engine = NotesEngine::new(".");
    let namespaces = match &args.namespace {
        Some(ns) => vec![ns.clone()],
        None => vec![
            "comments".to_string(),
            "review".to_string(),
            "todos".to_string(),
        ],
    };

    let mut all_notes = Vec::new();
    for ns in &namespaces {
        let namespace_enum = gn_core::Namespace::Custom(ns.clone());
        if let Ok(notes) = engine.read_notes(&namespace_enum) {
            all_notes.extend(notes);
        }
    }

    if all_notes.is_empty() {
        println!("No notes found in repository to verify.");
        return Ok(());
    }

    if args.all || args.note_id.is_none() {
        println!("\n\x1b[1;36m🔐 Cryptographic Note Signature Verification\x1b[0m\n");
        println!(
            "{:<10} {:<24} {:<28} {}",
            "ID", "AUTHOR", "STATUS BADGE", "DETAILS"
        );
        println!("{}", "─".repeat(80));

        let mut signed_count = 0;
        let mut unsigned_count = 0;
        let mut bad_count = 0;

        for note in &all_notes {
            let payload = note.signing_payload();
            let result = verify_signature(&payload, note.signature.as_deref(), &note.author);
            let id_short = &note.id.to_string()[..8];
            let author_short = note.author.split('<').next().unwrap_or(&note.author).trim();

            let badge = match result.status {
                SignatureStatus::Valid => {
                    signed_count += 1;
                    format!(
                        "\x1b[32m[✔ Signed by {}]\x1b[0m",
                        result.signer.as_deref().unwrap_or(author_short)
                    )
                }
                SignatureStatus::Unsigned => {
                    unsigned_count += 1;
                    "\x1b[33m[⚠ Unsigned]\x1b[0m".to_string()
                }
                SignatureStatus::Bad => {
                    bad_count += 1;
                    "\x1b[31m[✗ Bad Signature]\x1b[0m".to_string()
                }
            };

            let details = result.details.unwrap_or_default();
            println!(
                "{:<10} {:<24} {:<38} {}",
                id_short, author_short, badge, details
            );
        }

        println!("\nSummary: \x1b[32m{} valid\x1b[0m, \x1b[33m{} unsigned\x1b[0m, \x1b[31m{} invalid\x1b[0m (Total: {})",
            signed_count, unsigned_count, bad_count, all_notes.len()
        );

        if bad_count > 0 {
            return Err(anyhow!("One or more notes failed cryptographic signature verification."));
        }

        return Ok(());
    }

    let raw_id = args.note_id.as_ref().unwrap();
    let target = raw_id.trim().trim_start_matches('#');
    let found_note = if target.eq_ignore_ascii_case("latest") || target == "^" {
        all_notes.last().cloned()
    } else if let Ok(idx) = target.parse::<usize>() {
        if idx >= 1 && idx <= all_notes.len() {
            Some(all_notes[idx - 1].clone())
        } else {
            None
        }
    } else {
        all_notes
            .iter()
            .find(|n| n.id.to_string().starts_with(target))
            .cloned()
    };

    let note = found_note.ok_or_else(|| anyhow!("Target note '{}' not found", raw_id))?;
    let payload = note.signing_payload();
    let result = verify_signature(&payload, note.signature.as_deref(), &note.author);

    let author_short = note.author.split('<').next().unwrap_or(&note.author).trim();
    let badge = match result.status {
        SignatureStatus::Valid => {
            format!(
                "\x1b[32m[✔ Signed by {}]\x1b[0m",
                result.signer.as_deref().unwrap_or(author_short)
            )
        }
        SignatureStatus::Unsigned => "\x1b[33m[⚠ Unsigned]\x1b[0m".to_string(),
        SignatureStatus::Bad => "\x1b[31m[✗ Bad Signature]\x1b[0m".to_string(),
    };

    println!("\nNote:      {}", note.id);
    println!("Author:    {}", note.author);
    println!("Status:    {}", badge);
    if let Some(details) = result.details {
        println!("Details:   {}", details);
    }
    if let Some(ref sig) = note.signature {
        println!("\nSignature:\n{}", sig.trim());
    }
    println!();

    if result.status == SignatureStatus::Bad {
        return Err(anyhow!("Note {} has a bad cryptographic signature.", note.id));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::signing::{verify_signature, SignatureStatus};
    use gn_core::note::Note;
    use gn_core::Namespace;

    #[test]
    fn test_verify_unsigned_note() {
        let note = Note::new(
            "dummy_sha".to_string(),
            Some("main.rs".to_string()),
            Some(10),
            Some(12),
            "Unsigned review comment".to_string(),
            "Alice <alice@example.com>".to_string(),
            Namespace::Comments,
        );

        let payload = note.signing_payload();
        let res = verify_signature(&payload, note.signature.as_deref(), &note.author);
        assert_eq!(res.status, SignatureStatus::Unsigned);
    }

    #[test]
    fn test_verify_bad_signature_format() {
        let mut note = Note::new(
            "dummy_sha".to_string(),
            None,
            None,
            None,
            "Tampered note".to_string(),
            "Bob <bob@example.com>".to_string(),
            Namespace::Comments,
        );
        note.signature = Some("not-a-valid-armor-signature".to_string());

        let payload = note.signing_payload();
        let res = verify_signature(&payload, note.signature.as_deref(), &note.author);
        assert_eq!(res.status, SignatureStatus::Bad);
    }

    #[test]
    fn test_verify_args_parsing() {
        let args = VerifyArgs {
            note_id: Some("1".to_string()),
            all: false,
            namespace: Some("comments".to_string()),
        };
        assert_eq!(args.note_id.as_deref(), Some("1"));
        assert!(!args.all);
    }
}
