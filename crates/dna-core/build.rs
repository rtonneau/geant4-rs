//! Records the compiler version for output provenance.

use std::process::Command;

fn main() {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let version = Command::new(rustc)
        .arg("-V")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|| "rustc unknown".into());
    println!("cargo:rustc-env=DNA_CORE_RUSTC_VERSION={version}");
    println!("cargo:rerun-if-changed=build.rs");
}
