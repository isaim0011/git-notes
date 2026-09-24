use anyhow::{Context, Result};
use gn_core::{MergeStrategy, Namespace, NotesEngine};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs, UdpSocket};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pReport {
    pub peer: String,
    pub fetched: usize,
    pub merged: usize,
}

/// Helper structure managing a local Git P2P server process.
pub struct P2pServer {
    child: Option<Child>,
    port: u16,
    repo_path: PathBuf,
}

impl P2pServer {
    /// Start a lightweight git daemon serving `refs/notes/*` for the given repository.
    pub fn start(repo_path: &Path, port: u16) -> Result<Self> {
        let abs_repo = if repo_path.is_relative() {
            std::fs::canonicalize(repo_path)
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| repo_path.to_path_buf()))
        } else {
            repo_path.to_path_buf()
        };

        let child = Command::new("git")
            .args([
                "daemon",
                "--reuseaddr",
                "--export-all",
                "--base-path-relaxed",
                &format!("--base-path={}", abs_repo.display()),
                &format!("--port={}", port),
                "--enable=receive-pack",
                abs_repo.to_str().unwrap_or("."),
            ])
            .spawn()
            .with_context(|| format!("Failed to start git daemon on port {}", port))?;

        Ok(Self {
            child: Some(child),
            port,
            repo_path: abs_repo,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }

    /// Stop the server process.
    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }

    /// Wait for server process to exit.
    pub fn wait(&mut self) -> Result<std::process::ExitStatus> {
        if let Some(ref mut child) = self.child {
            let status = child.wait()?;
            Ok(status)
        } else {
            anyhow::bail!("Server is not running");
        }
    }
}

impl Drop for P2pServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Parse a peer address string into `(host, port)`.
/// Supports:
/// - `"192.168.1.50:9418"` -> `("192.168.1.50", 9418)`
/// - `"192.168.1.50"` -> `("192.168.1.50", default_port)`
/// - `"git://192.168.1.50:9418/"` -> `("192.168.1.50", 9418)`
pub fn parse_peer_address(peer: &str, default_port: u16) -> (String, u16) {
    let s = peer.trim();
    let s = s.strip_prefix("git://").unwrap_or(s);
    let s = s.trim_end_matches('/');

    if let Some((host, port_str)) = s.split_once(':') {
        if let Ok(p) = port_str.parse::<u16>() {
            return (host.to_string(), p);
        }
    }
    (s.to_string(), default_port)
}

/// Query local network IP addresses to assist peer pairing.
pub fn get_local_ips() -> Vec<IpAddr> {
    let mut ips = Vec::new();

    // 1. Probe local routing gateway addresses (non-blocking UDP socket check)
    for probe_target in &[
        "192.168.1.1:80",
        "192.168.0.1:80",
        "10.0.0.1:80",
        "172.16.0.1:80",
        "8.8.8.8:80",
    ] {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect(probe_target).is_ok() {
                if let Ok(local_addr) = socket.local_addr() {
                    let ip = local_addr.ip();
                    if !ip.is_loopback() && !ips.contains(&ip) {
                        ips.push(ip);
                    }
                }
            }
        }
    }

    // 2. Resolve local hostname
    if let Ok(hostname) = std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")) {
        if let Ok(addrs) = format!("{}:0", hostname).to_socket_addrs() {
            for addr in addrs {
                let ip = addr.ip();
                if ip.is_ipv4() && !ip.is_loopback() && !ips.contains(&ip) {
                    ips.push(ip);
                }
            }
        }
    }

    if ips.is_empty() {
        ips.push(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }

    ips
}

/// Start a lightweight local P2P git notes sync server on the local LAN.
pub fn serve_p2p(repo_path: &Path, port: u16) -> Result<()> {
    let mut server = P2pServer::start(repo_path, port)?;
    let local_ips = get_local_ips();

    println!("\n\x1b[1;36m📡 git-notes P2P Server listening on port {}\x1b[0m", port);
    println!("Repository: \x1b[33m{}\x1b[0m\n", server.repo_path().display());
    println!("Share your connection address with your team on the local LAN:");

    for ip in &local_ips {
        println!("  • \x1b[32mgn sync p2p connect {}:{}\x1b[0m", ip, port);
    }
    println!("  • (Loopback): \x1b[2mgn sync p2p connect 127.0.0.1:{}\x1b[0m\n", port);
    println!("\x1b[33m⚡ Ready! Press Ctrl+C at any time to stop serving.\x1b[0m\n");

    let _ = server.wait();
    println!("\x1b[32m✔ P2P server stopped.\x1b[0m");
    Ok(())
}

/// Pull and merge notes directly from a peer developer's IP on the local Wi-Fi without needing internet or GitHub!
pub fn connect_peer(
    repo_path: &Path,
    peer: &str,
    strategy: &dyn MergeStrategy,
) -> Result<P2pReport> {
    let (host, port) = parse_peer_address(peer, 9418);
    let remote_url = format!("git://{}:{}/", host, port);

    let temp_token = format!("p2p_{}", Uuid::new_v4().simple());
    let refspec = format!("refs/notes/*:refs/notes/{}/*", temp_token);

    println!(
        "\x1b[36m⏳ Connecting to peer at {} ({})...\x1b[0m",
        peer, remote_url
    );

    let fetch_out = Command::new("git")
        .current_dir(repo_path)
        .args(["fetch", &remote_url, &refspec])
        .output()
        .context("Failed to connect to peer git daemon")?;

    if !fetch_out.status.success() {
        let err = String::from_utf8_lossy(&fetch_out.stderr);
        anyhow::bail!(
            "Failed to pull notes from peer {}: {}",
            peer,
            err.trim()
        );
    }

    // Discover imported temporary refs
    let temp_ref_prefix = format!("refs/notes/{}/", temp_token);
    let list_out = Command::new("git")
        .current_dir(repo_path)
        .args(["for-each-ref", "--format=%(refname)", &temp_ref_prefix])
        .output()
        .context("Failed to inspect peer notes refs")?;

    let imported_refs: Vec<String> = String::from_utf8_lossy(&list_out.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| l.starts_with(&temp_ref_prefix))
        .collect();

    let engine = NotesEngine::new(repo_path);
    let mut total_fetched = 0;
    let mut total_merged = 0;

    for temp_ref in &imported_refs {
        let ns_suffix = temp_ref
            .strip_prefix(&temp_ref_prefix)
            .unwrap_or(temp_ref);

        let target_ns = Namespace::from_str(ns_suffix);
        let temp_ns = Namespace::Custom(format!("{}/{}", temp_token, ns_suffix));

        let local_notes = engine.read_notes(&target_ns).unwrap_or_default();
        let peer_notes = engine.read_notes(&temp_ns).unwrap_or_default();

        total_fetched += peer_notes.len();

        if !peer_notes.is_empty() {
            let merged = strategy.merge(&local_notes, &peer_notes);
            for note in &merged {
                let mut note_to_write = note.clone();
                note_to_write.namespace = target_ns.clone();
                engine.write_note(&note_to_write)?;
            }
            total_merged += merged.len();
        }

        // Clean up temporary ref
        let _ = Command::new("git")
            .current_dir(repo_path)
            .args(["update-ref", "-d", temp_ref])
            .status();
    }

    Ok(P2pReport {
        peer: format!("{}:{}", host, port),
        fetched: total_fetched,
        merged: total_merged,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gn_core::{LwwStrategy, Note};
    use std::fs;

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("gn_p2p_test_{}", Uuid::new_v4().simple()));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn run_git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .current_dir(dir)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success(), "git {:?} failed", args);
    }

    #[test]
    fn test_parse_peer_address() {
        assert_eq!(
            parse_peer_address("192.168.1.10:9418", 9418),
            ("192.168.1.10".to_string(), 9418)
        );
        assert_eq!(
            parse_peer_address("192.168.1.10", 9418),
            ("192.168.1.10".to_string(), 9418)
        );
        assert_eq!(
            parse_peer_address("git://10.0.0.5:8000/", 9418),
            ("10.0.0.5".to_string(), 8000)
        );
        assert_eq!(
            parse_peer_address("localhost:9999", 9418),
            ("localhost".to_string(), 9999)
        );
    }

    #[test]
    fn test_p2p_sync_end_to_end() {
        let port = 19428;
        let repo_peer = TestDir::new();
        let repo_local = TestDir::new();

        // Setup peer repo
        run_git(repo_peer.path(), &["init"]);
        run_git(repo_peer.path(), &["config", "user.name", "Peer Dev"]);
        run_git(repo_peer.path(), &["config", "user.email", "peer@lan.local"]);
        fs::write(repo_peer.path().join("main.rs"), "fn main() {}\n").unwrap();
        run_git(repo_peer.path(), &["add", "."]);
        run_git(repo_peer.path(), &["commit", "-m", "Init"]);

        let peer_engine = NotesEngine::new(repo_peer.path());
        let peer_note = Note::new(
            "HEAD".to_string(),
            Some("main.rs".to_string()),
            Some(1),
            Some(1),
            "LAN review comment from peer developer".to_string(),
            "Peer Dev <peer@lan.local>".to_string(),
            Namespace::Review,
        );
        peer_engine.write_note(&peer_note).unwrap();

        // Setup local repo
        run_git(repo_local.path(), &["init"]);
        run_git(repo_local.path(), &["config", "user.name", "Local Dev"]);
        run_git(repo_local.path(), &["config", "user.email", "local@lan.local"]);
        fs::write(repo_local.path().join("main.rs"), "fn main() {}\n").unwrap();
        run_git(repo_local.path(), &["add", "."]);
        run_git(repo_local.path(), &["commit", "-m", "Init"]);

        // Start P2P server on peer repo
        let mut server = P2pServer::start(repo_peer.path(), port).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(800));

        // Connect from local repo
        let strategy = LwwStrategy;
        let report = connect_peer(
            repo_local.path(),
            &format!("127.0.0.1:{}", port),
            &strategy,
        )
        .unwrap();

        assert_eq!(report.fetched, 1);
        assert_eq!(report.merged, 1);

        // Verify local repo received the note
        let local_engine = NotesEngine::new(repo_local.path());
        let notes = local_engine.read_notes(&Namespace::Review).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].body, "LAN review comment from peer developer");

        server.stop().unwrap();
    }
}
