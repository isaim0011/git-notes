use anyhow::{bail, Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use uuid::Uuid;

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(suffix: &str) -> Result<Self> {
        let file_name = format!("gn_sig_{}_{}{}", std::process::id(), Uuid::new_v4(), suffix);
        let path = std::env::temp_dir().join(file_name);
        Ok(Self { path })
    }

    fn write(&mut self, content: &[u8]) -> Result<()> {
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureStatus {
    Valid,
    Bad,
    Unsigned,
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub status: SignatureStatus,
    pub signer: Option<String>,
    pub details: Option<String>,
}

/// Check if cryptographic signing is requested either via CLI flag `--sign`
/// or git config `git-notes.sign` == true.
pub fn is_signing_requested(flag: bool) -> bool {
    if flag {
        return true;
    }
    match Command::new("git")
        .args(["config", "--bool", "git-notes.sign"])
        .output()
    {
        Ok(out) if out.status.success() => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
            val == "true" || val == "yes" || val == "1"
        }
        _ => false,
    }
}

/// Sign the payload string using git / gpg / ssh signing configured in git or default gpg.
pub fn sign_payload(payload: &str) -> Result<String> {
    let gpg_format = Command::new("git")
        .args(["config", "gpg.format"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !s.is_empty() {
                    Some(s)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or_else(|| "openpgp".to_string());

    let signing_key = Command::new("git")
        .args(["config", "user.signingkey"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !s.is_empty() {
                    Some(s)
                } else {
                    None
                }
            } else {
                None
            }
        });

    if gpg_format.eq_ignore_ascii_case("ssh") {
        if let Some(key) = &signing_key {
            if let Ok(sig) = sign_with_ssh(payload, key) {
                return Ok(sig);
            }
        }
    }

    // Try gpg with signing_key if specified
    if let Some(key) = &signing_key {
        if let Ok(sig) = sign_with_gpg(payload, Some(key)) {
            return Ok(sig);
        }
    }

    // Try default gpg
    if let Ok(sig) = sign_with_gpg(payload, None) {
        return Ok(sig);
    }

    // Fallback: git tag -s or git commit-tree or git var / mock fallback
    bail!(
        "Failed to sign note. Ensure gpg or ssh signing is configured (e.g. git config user.signingkey <key> or gpg is installed)."
    )
}

fn sign_with_gpg(payload: &str, key: Option<&str>) -> Result<String> {
    let mut cmd = Command::new("gpg");
    cmd.args(["--batch", "--armor", "--detach-sign", "--yes"]);
    if let Some(k) = key {
        cmd.args(["--default-key", k]);
    }
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().context("Failed to spawn gpg")?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(payload.as_bytes())?;
    }
    let output = child.wait_with_output().context("Failed to read gpg output")?;
    if output.status.success() {
        let sig = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !sig.is_empty() {
            return Ok(sig);
        }
    }
    bail!(
        "gpg signing failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    )
}

fn sign_with_ssh(payload: &str, key_path: &str) -> Result<String> {
    let mut temp_file = TempFile::new(".txt")?;
    temp_file.write(payload.as_bytes())?;
    let temp_path = temp_file.path().to_path_buf();

    let output = Command::new("ssh-keygen")
        .args([
            "-Y",
            "sign",
            "-n",
            "git-notes",
            "-f",
            key_path,
            temp_path.to_str().unwrap(),
        ])
        .output()
        .context("Failed to execute ssh-keygen -Y sign")?;

    if output.status.success() {
        let sig_path = temp_path.with_extension(format!(
            "{}.sig",
            temp_path.extension().and_then(|s| s.to_str()).unwrap_or("")
        ));
        if sig_path.exists() {
            let sig_content = std::fs::read_to_string(&sig_path)?;
            let _ = std::fs::remove_file(sig_path);
            return Ok(sig_content.trim().to_string());
        }
    }
    bail!(
        "ssh signing failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    )
}

/// Verify signature against payload
pub fn verify_signature(payload: &str, signature: Option<&str>, author: &str) -> VerificationResult {
    let sig = match signature {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => {
            return VerificationResult {
                status: SignatureStatus::Unsigned,
                signer: None,
                details: Some("No cryptographic signature present".to_string()),
            }
        }
    };

    // If signature is OpenPGP
    if sig.contains("-----BEGIN PGP SIGNATURE-----") {
        return verify_gpg_signature(payload, sig, author);
    }

    // If signature is SSH
    if sig.contains("-----BEGIN SSH SIGNATURE-----") {
        return verify_ssh_signature(payload, sig, author);
    }

    // Unknown signature format
    VerificationResult {
        status: SignatureStatus::Bad,
        signer: None,
        details: Some("Unrecognized signature format".to_string()),
    }
}

fn verify_gpg_signature(payload: &str, signature: &str, fallback_author: &str) -> VerificationResult {
    let mut sig_file = match TempFile::new(".sig") {
        Ok(f) => f,
        Err(e) => {
            return VerificationResult {
                status: SignatureStatus::Bad,
                signer: None,
                details: Some(format!("Failed to create temp signature file: {}", e)),
            }
        }
    };
    if let Err(e) = sig_file.write(signature.as_bytes()) {
        return VerificationResult {
            status: SignatureStatus::Bad,
            signer: None,
            details: Some(format!("Failed to write temp signature file: {}", e)),
        };
    }

    let mut data_file = match TempFile::new(".data") {
        Ok(f) => f,
        Err(e) => {
            return VerificationResult {
                status: SignatureStatus::Bad,
                signer: None,
                details: Some(format!("Failed to create temp data file: {}", e)),
            }
        }
    };
    if let Err(e) = data_file.write(payload.as_bytes()) {
        return VerificationResult {
            status: SignatureStatus::Bad,
            signer: None,
            details: Some(format!("Failed to write temp data file: {}", e)),
        };
    }

    let output = Command::new("gpg")
        .args([
            "--batch",
            "--verify",
            sig_file.path().to_str().unwrap(),
            data_file.path().to_str().unwrap(),
        ])
        .output();

    match output {
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            if out.status.success() || stderr.contains("Good signature from") {
                let signer = extract_gpg_signer(&stderr).unwrap_or_else(|| fallback_author.to_string());
                VerificationResult {
                    status: SignatureStatus::Valid,
                    signer: Some(signer),
                    details: Some("GPG cryptographic signature verified".to_string()),
                }
            } else {
                VerificationResult {
                    status: SignatureStatus::Bad,
                    signer: None,
                    details: Some(stderr.trim().to_string()),
                }
            }
        }
        Err(e) => VerificationResult {
            status: SignatureStatus::Bad,
            signer: None,
            details: Some(format!("Could not run gpg: {}", e)),
        },
    }
}

fn extract_gpg_signer(stderr: &str) -> Option<String> {
    for line in stderr.lines() {
        if let Some(idx) = line.find("Good signature from") {
            let rest = &line[idx + "Good signature from".len()..];
            return Some(rest.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn verify_ssh_signature(payload: &str, signature: &str, fallback_author: &str) -> VerificationResult {
    let mut sig_file = match TempFile::new(".sig") {
        Ok(f) => f,
        Err(e) => {
            return VerificationResult {
                status: SignatureStatus::Bad,
                signer: None,
                details: Some(format!("Failed to create temp signature file: {}", e)),
            }
        }
    };
    if let Err(e) = sig_file.write(signature.as_bytes()) {
        return VerificationResult {
            status: SignatureStatus::Bad,
            signer: None,
            details: Some(format!("Failed to write temp signature file: {}", e)),
        };
    }

    let output = Command::new("ssh-keygen")
        .args([
            "-Y",
            "check-novalidate",
            "-n",
            "git-notes",
            "-s",
            sig_file.path().to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match output {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(payload.as_bytes());
            }
            match child.wait_with_output() {
                Ok(out) => {
                    let combined = format!(
                        "{}\n{}",
                        String::from_utf8_lossy(&out.stdout),
                        String::from_utf8_lossy(&out.stderr)
                    );
                    if out.status.success() || combined.contains("Good") {
                        VerificationResult {
                            status: SignatureStatus::Valid,
                            signer: Some(fallback_author.to_string()),
                            details: Some("SSH cryptographic signature verified".to_string()),
                        }
                    } else {
                        VerificationResult {
                            status: SignatureStatus::Bad,
                            signer: None,
                            details: Some(combined.trim().to_string()),
                        }
                    }
                }
                Err(e) => VerificationResult {
                    status: SignatureStatus::Bad,
                    signer: None,
                    details: Some(format!("Failed to wait for ssh-keygen: {}", e)),
                },
            }
        }
        Err(e) => VerificationResult {
            status: SignatureStatus::Bad,
            signer: None,
            details: Some(format!("Could not run ssh-keygen: {}", e)),
        },
    }
}
