//! Capture build evidence without substituting guessed revisions or checksums.

use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Command;

fn command_text(program: &str, args: &[&str], cwd: &Path) -> Option<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?.trim().to_owned();
    (!value.is_empty()).then_some(value)
}

fn emit(key: &str, value: &str) {
    println!("cargo:rustc-env=DNA_CORE_{key}={value}");
}

fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR");
    let crate_dir = Path::new(&crate_dir);
    let workspace = crate_dir.join("../..");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    emit(
        "RUSTC_VERSION",
        &command_text(&rustc, &["-V"], crate_dir).unwrap_or_default(),
    );

    let in_workspace_repo = command_text("git", &["rev-parse", "--show-toplevel"], &workspace)
        .and_then(|root| Path::new(&root).canonicalize().ok())
        == workspace.canonicalize().ok();
    let revision = in_workspace_repo
        .then(|| command_text("git", &["rev-parse", "--verify", "HEAD"], &workspace))
        .flatten();
    let dirty = Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=all"])
        .current_dir(&workspace)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| !output.stdout.is_empty())
        .filter(|_| in_workspace_repo);
    emit("GIT_REVISION", revision.as_deref().unwrap_or(""));
    emit(
        "GIT_DIRTY",
        match dirty {
            Some(true) => "true",
            Some(false) => "false",
            None => "unavailable",
        },
    );

    let lockfile = workspace.join("Cargo.lock");
    let lock_hash = std::fs::read(&lockfile)
        .ok()
        .map(|bytes| format!("{:x}", Sha256::digest(bytes)));
    emit("LOCKFILE_SHA256", lock_hash.as_deref().unwrap_or(""));
    emit("TARGET", &std::env::var("TARGET").unwrap_or_default());
    for (name, env_name) in [
        ("TARGET_ARCH", "CARGO_CFG_TARGET_ARCH"),
        ("TARGET_OS", "CARGO_CFG_TARGET_OS"),
        ("TARGET_ENV", "CARGO_CFG_TARGET_ENV"),
        ("TARGET_FAMILY", "CARGO_CFG_TARGET_FAMILY"),
    ] {
        emit(name, &std::env::var(env_name).unwrap_or_default());
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", lockfile.display());
    // Git metadata can change independently of source. Resolve paths through Git so linked
    // worktrees are handled without assuming `.git` is a directory.
    if in_workspace_repo {
        for item in ["HEAD", "index"] {
            if let Some(path) = command_text("git", &["rev-parse", "--git-path", item], &workspace)
            {
                let path = Path::new(&path);
                let path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    workspace.join(path)
                };
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    for path in ["crates", "data", "Cargo.toml", "rust-toolchain.toml"] {
        println!("cargo:rerun-if-changed={}", workspace.join(path).display());
    }
}
