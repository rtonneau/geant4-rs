//! Data manifest: names, sizes and SHA-256 of the committed Geant4 data files.
//!
//! `data/MANIFEST.toml` is generated from the full G4EMLOW dataset by
//! `scripts/make_data_manifest.py`. [`Manifest::verify`] re-checks the files actually present,
//! so a changed byte, a line-ending conversion or a missing file is a typed error rather than
//! a silently different cross section (risk R2).

use crate::error::DnaError;
use crate::sha256::{Sha256, hex};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

/// One data file entry; `path` is relative to the data root (e.g. `dna/sigma_...dat`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ManifestFile {
    pub path: String,
    pub group: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Deserialize)]
struct Raw {
    dataset: String,
    version: String,
    #[serde(rename = "file")]
    files: Vec<ManifestFile>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    dataset: String,
    version: String,
    files: Vec<ManifestFile>,
    hash: String,
}

impl Manifest {
    pub fn from_toml_str(text: &str) -> Result<Self, DnaError> {
        let raw: Raw =
            toml::from_str(text).map_err(|e| DnaError::InvalidManifest(e.to_string()))?;
        let mut seen = HashSet::new();
        for f in &raw.files {
            // Use one portable spelling so identity never contains a machine root or aliases
            // another entry through traversal, Windows prefixes or separator differences.
            if f.path.contains(['\\', ':'])
                || f.path
                    .split('/')
                    .any(|part| matches!(part, "" | "." | ".."))
            {
                return Err(DnaError::InvalidManifest(format!(
                    "{}: path must be a relative forward-slash path without . or .. components",
                    f.path
                )));
            }
            let valid_hex = f.sha256.len() == 64
                && f.sha256
                    .bytes()
                    .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
            if !valid_hex {
                return Err(DnaError::InvalidManifest(format!(
                    "{}: sha256 must be 64 lowercase hex digits",
                    f.path
                )));
            }
            if !seen.insert(f.path.as_str()) {
                return Err(DnaError::InvalidManifest(format!(
                    "duplicate path {}",
                    f.path
                )));
            }
        }
        Ok(Self {
            dataset: raw.dataset,
            version: raw.version,
            files: raw.files,
            hash: hex(&Sha256::digest(text.as_bytes())),
        })
    }

    pub fn load(path: &Path) -> Result<Self, DnaError> {
        let text = std::fs::read_to_string(path).map_err(|e| io_error(path, &e))?;
        Self::from_toml_str(&text)
    }

    pub fn dataset(&self) -> &str {
        &self.dataset
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn files(&self) -> &[ManifestFile] {
        &self.files
    }
    /// SHA-256 of the manifest text; recorded in output metadata.
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Check every listed file under `data_root` against its recorded size and SHA-256.
    pub fn verify(&self, data_root: &Path) -> Result<(), DnaError> {
        self.files
            .iter()
            .try_for_each(|f| self.verify_path(data_root, &f.path))
    }

    /// Check one listed file (`rel_path` as written in the manifest, e.g. `dna/sigma_x.dat`).
    /// A path the manifest does not list is an error: an unlisted file cannot be vouched for.
    pub fn verify_path(&self, data_root: &Path, rel_path: &str) -> Result<(), DnaError> {
        self.read_verified(data_root, rel_path).map(|_| ())
    }

    /// Read a listed file once, returning precisely the bytes whose size and SHA-256 match.
    /// Callers that parse these bytes do not reopen the source after verification.
    pub fn read_verified(&self, data_root: &Path, rel_path: &str) -> Result<Vec<u8>, DnaError> {
        let f = self.entry(rel_path)?;
        let full = data_root.join(&f.path);
        let bytes = std::fs::read(&full).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => DnaError::MissingDataFile {
                path: f.path.clone(),
            },
            _ => io_error(&full, &e),
        })?;
        self.verify_bytes(rel_path, &bytes)?;
        Ok(bytes)
    }

    /// Check captured bytes against a listed file, without reading any filesystem path.
    pub fn verify_bytes(&self, rel_path: &str, bytes: &[u8]) -> Result<(), DnaError> {
        let f = self.entry(rel_path)?;
        let len = bytes.len() as u64;
        if len != f.bytes {
            return Err(DnaError::ChecksumMismatch {
                path: f.path.clone(),
                expected: format!("{} bytes", f.bytes),
                actual: format!("{len} bytes"),
            });
        }
        let actual = hex(&Sha256::digest(bytes));
        if actual != f.sha256 {
            return Err(DnaError::ChecksumMismatch {
                path: f.path.clone(),
                expected: f.sha256.clone(),
                actual,
            });
        }
        Ok(())
    }

    fn entry(&self, rel_path: &str) -> Result<&ManifestFile, DnaError> {
        self.files
            .iter()
            .find(|f| f.path == rel_path)
            .ok_or_else(|| DnaError::InvalidManifest(format!("{rel_path} is not listed")))
    }
}

fn io_error(path: &Path, e: &std::io::Error) -> DnaError {
    DnaError::Io {
        path: path.display().to_string(),
        message: e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DnaError;
    use std::path::PathBuf;

    const ABC_SHA: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    fn sample_toml() -> String {
        format!(
            r#"dataset = "G4EMLOW"
version = "8.8"

[[file]]
path = "dna/abc.dat"
group = "test"
bytes = 3
sha256 = "{ABC_SHA}"
"#
        )
    }

    /// Unique scratch directory under the cargo target dir; removed on drop.
    struct Scratch(PathBuf);
    impl Scratch {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("{tag}-{}", std::process::id()));
            std::fs::create_dir_all(dir.join("dna")).unwrap();
            Self(dir)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn parses_dataset_version_and_files() {
        let m = Manifest::from_toml_str(&sample_toml()).unwrap();
        assert_eq!(m.dataset(), "G4EMLOW");
        assert_eq!(m.version(), "8.8");
        assert_eq!(m.files().len(), 1);
        assert_eq!(m.files()[0].path, "dna/abc.dat");
        assert_eq!(m.files()[0].group, "test");
        assert_eq!(m.files()[0].bytes, 3);
    }

    #[test]
    fn manifest_hash_is_sha256_of_its_text() {
        let m = Manifest::from_toml_str(&sample_toml()).unwrap();
        assert_eq!(
            m.hash(),
            crate::sha256::hex(&crate::sha256::Sha256::digest(sample_toml().as_bytes()))
        );
    }

    #[test]
    fn rejects_malformed_checksum_and_duplicates() {
        let bad = sample_toml().replace(ABC_SHA, "XYZ");
        assert!(matches!(
            Manifest::from_toml_str(&bad),
            Err(DnaError::InvalidManifest(_))
        ));
        let dup = format!(
            "{}{}",
            sample_toml(),
            &sample_toml()[sample_toml().find("[[file]]").unwrap()..]
        );
        assert!(matches!(
            Manifest::from_toml_str(&dup),
            Err(DnaError::InvalidManifest(_))
        ));
        assert!(matches!(
            Manifest::from_toml_str("not toml ["),
            Err(DnaError::InvalidManifest(_))
        ));
    }

    #[test]
    fn verify_accepts_matching_file() {
        let s = Scratch::new("ok");
        std::fs::write(s.0.join("dna/abc.dat"), b"abc").unwrap();
        let m = Manifest::from_toml_str(&sample_toml()).unwrap();
        m.verify(&s.0).unwrap();
    }

    #[test]
    fn verify_reports_missing_file() {
        let s = Scratch::new("missing");
        let m = Manifest::from_toml_str(&sample_toml()).unwrap();
        assert_eq!(
            m.verify(&s.0),
            Err(DnaError::MissingDataFile {
                path: "dna/abc.dat".into()
            })
        );
    }

    #[test]
    fn verify_detects_changed_content_of_same_size() {
        let s = Scratch::new("content");
        std::fs::write(s.0.join("dna/abc.dat"), b"abd").unwrap();
        let m = Manifest::from_toml_str(&sample_toml()).unwrap();
        assert!(matches!(
            m.verify(&s.0),
            Err(DnaError::ChecksumMismatch { path, expected, .. }) if path == "dna/abc.dat" && expected == ABC_SHA
        ));
    }

    #[test]
    fn verify_detects_line_ending_conversion_by_size() {
        let s = Scratch::new("crlf");
        std::fs::write(s.0.join("dna/abc.dat"), b"abc\r\n").unwrap();
        let m = Manifest::from_toml_str(&sample_toml()).unwrap();
        assert!(matches!(
            m.verify(&s.0),
            Err(DnaError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn captured_bytes_are_checked_without_reopening_the_file() {
        let s = Scratch::new("captured");
        let path = s.0.join("dna/abc.dat");
        std::fs::write(&path, b"abc").unwrap();
        let m = Manifest::from_toml_str(&sample_toml()).unwrap();
        let captured = m.read_verified(&s.0, "dna/abc.dat").unwrap();
        std::fs::write(&path, b"abd").unwrap();
        assert_eq!(captured, b"abc");
        m.verify_bytes("dna/abc.dat", &captured).unwrap();
        assert!(matches!(
            m.read_verified(&s.0, "dna/abc.dat"),
            Err(DnaError::ChecksumMismatch { .. })
        ));
        assert!(matches!(
            m.verify_bytes("dna/unlisted.dat", b"abc"),
            Err(DnaError::InvalidManifest(_))
        ));
    }

    #[test]
    fn manifest_paths_are_portable_relative_paths() {
        for path in [
            "",
            "/dna/abc.dat",
            "../abc.dat",
            "dna/../abc.dat",
            "dna/./abc.dat",
            "dna//abc.dat",
            "C:/dna/abc.dat",
            "dna\\abc.dat",
        ] {
            // Escape backslashes for the TOML basic string.
            let text = sample_toml().replace("dna/abc.dat", &path.replace('\\', "\\\\"));
            assert!(
                matches!(
                    Manifest::from_toml_str(&text),
                    Err(DnaError::InvalidManifest(_))
                ),
                "accepted {path:?}"
            );
        }
    }
}
