//! The Geant4 data subset committed under `data/` must match `data/MANIFEST.toml` byte for byte
//! (docs/10-data-requirements.md). Lets CI and cloud sessions run real-table tests.

use g4_core::manifest::Manifest;
use std::path::PathBuf;

fn data_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

#[test]
fn committed_manifest_describes_g4emlow_8_8() {
    let m = Manifest::load(&data_root().join("MANIFEST.toml")).unwrap();
    assert_eq!(m.dataset(), "G4EMLOW");
    assert_eq!(m.version(), "8.8");
    assert_eq!(m.files().len(), 20);
    assert!(m.files().iter().all(|f| f.path.starts_with("dna/")));
}

#[test]
fn committed_data_files_match_manifest_checksums() {
    let root = data_root();
    let m = Manifest::load(&root.join("MANIFEST.toml")).unwrap();
    m.verify(&root).unwrap();
}

#[test]
fn a_tampered_copy_is_detected() {
    let root = data_root();
    let m = Manifest::load(&root.join("MANIFEST.toml")).unwrap();
    let victim = &m.files()[0];
    let scratch = std::env::temp_dir().join(format!("g4rs-tamper-{}", std::process::id()));
    std::fs::create_dir_all(scratch.join("dna")).unwrap();
    let mut bytes = std::fs::read(root.join(&victim.path)).unwrap();
    bytes[0] ^= 0x01;
    std::fs::write(scratch.join(&victim.path), bytes).unwrap();
    let err = m.verify(&scratch).unwrap_err();
    std::fs::remove_dir_all(&scratch).unwrap();
    assert!(
        matches!(err, g4_core::error::DnaError::ChecksumMismatch { .. }),
        "{err:?}"
    );
}
