#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set("CompanyName", "Bimo");
    res.set(
        "FileDescription",
        "git-notes TUI — browse code hunks and comment threads",
    );
    res.set("LegalCopyright", "Copyright (C) 2026 Bimo");
    res.set("ProductName", "git-notes-tui");
    res.set("OriginalFilename", "git-notes-tui.exe");
    res.set("InternalName", "git-notes-tui");
    res.set("LegalTrademarks", "git-notes");
    res.set_language(0x0409);
    res.set_icon("../../assets/icon.ico");
    if let Err(e) = res.compile() {
        eprintln!("cargo:warning=Failed to compile windows resource: {}", e);
    }

    if std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "gnu" {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let obj_path = std::path::Path::new(&out_dir).join("resource.o");
        if obj_path.exists() {
            println!("cargo:rustc-link-arg={}", obj_path.display());
        }
    }
}

#[cfg(not(windows))]
fn main() {}
