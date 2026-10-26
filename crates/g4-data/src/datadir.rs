//! Locating and loading the Geant4 data files.
//!
//! Precedence (docs/10): explicit path (the TOML key `data_dir`) > environment variable
//! `G4RS_DATA_DIR` (a directory containing `dna/`, e.g. a full `G4EMLOW8.8`) > the repository's
//! committed `data/` (compile-time location, valid for development builds and CI only: an
//! installed binary must set the variable or the key). A path that is given but invalid is an
//! error; there is no silent fallback to the next candidate.

use crate::catalog::{DnaFile, Table};
use crate::error::DataError;
use g4_core::manifest::Manifest;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Name of the environment variable that selects the data root.
pub const ENV_VAR: &str = "G4RS_DATA_DIR";

/// Root of the repository's committed data subset (`<repo>/data`).
pub fn repo_data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

/// A data root known to contain a `dna/` directory.
#[derive(Debug, Clone, PartialEq)]
pub struct DataDir {
    root: PathBuf,
}

impl DataDir {
    /// Use `root` as data root; it must contain a `dna/` directory.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, DataError> {
        let root = root.into();
        if !root.join("dna").is_dir() {
            return Err(DataError::InvalidDataDir {
                path: root.display().to_string(),
                reason: "no dna/ subdirectory".to_string(),
            });
        }
        Ok(Self { root })
    }

    /// Resolve with the documented precedence, reading `G4RS_DATA_DIR` from the environment.
    pub fn resolve(explicit: Option<&Path>) -> Result<Self, DataError> {
        Self::resolve_with(explicit, std::env::var_os(ENV_VAR))
    }

    /// As [`resolve`](Self::resolve) with the environment value passed in (testable without
    /// mutating the process environment). An empty value counts as unset.
    pub fn resolve_with(explicit: Option<&Path>, env: Option<OsString>) -> Result<Self, DataError> {
        if let Some(p) = explicit {
            return Self::new(p);
        }
        match env {
            Some(v) if !v.is_empty() => Self::new(PathBuf::from(v)),
            _ => Self::new(repo_data_dir()),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Full path of a catalogued file.
    pub fn path(&self, file: DnaFile) -> PathBuf {
        self.root.join(file.relative_path())
    }

    /// Check size and SHA-256 of every file listed in `manifest` under this root.
    pub fn verify(&self, manifest: &Manifest) -> Result<(), DataError> {
        Ok(manifest.verify(&self.root)?)
    }

    /// Load one table without checksum verification.
    pub fn load(&self, file: DnaFile) -> Result<Table, DataError> {
        file.load_from(&self.path(file))
    }

    /// Verify the file against `manifest`, then load it.
    pub fn load_verified(&self, file: DnaFile, manifest: &Manifest) -> Result<Table, DataError> {
        manifest.verify_path(&self.root, file.relative_path())?;
        self.load(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use g4_core::error::DnaError;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("g4-data-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("dna")).unwrap();
        dir
    }

    #[test]
    fn precedence_explicit_then_env_then_repo() {
        let a = scratch("explicit");
        let b = scratch("env");
        let r = DataDir::resolve_with(Some(&a), Some(b.clone().into_os_string())).unwrap();
        assert_eq!(r.root(), a);
        let r = DataDir::resolve_with(None, Some(b.clone().into_os_string())).unwrap();
        assert_eq!(r.root(), b);
        let r = DataDir::resolve_with(None, None).unwrap();
        assert_eq!(r.root(), repo_data_dir());
        // empty variable = unset
        let r = DataDir::resolve_with(None, Some(OsString::new())).unwrap();
        assert_eq!(r.root(), repo_data_dir());
        std::fs::remove_dir_all(a).unwrap();
        std::fs::remove_dir_all(b).unwrap();
    }

    #[test]
    fn invalid_roots_are_errors_without_fallback() {
        let missing = Path::new("/nonexistent/g4rs-data");
        assert!(matches!(
            DataDir::resolve_with(Some(missing), None),
            Err(DataError::InvalidDataDir { .. })
        ));
        assert!(matches!(
            DataDir::resolve_with(None, Some(missing.as_os_str().to_owned())),
            Err(DataError::InvalidDataDir { .. })
        ));
        // a directory without dna/
        let d = std::env::temp_dir().join(format!("g4-data-nodna-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        assert!(matches!(
            DataDir::new(&d),
            Err(DataError::InvalidDataDir { .. })
        ));
        std::fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn missing_file_is_a_typed_error() {
        let d = scratch("missing");
        let dir = DataDir::new(&d).unwrap();
        let err = dir.load(DnaFile::MeltonAttachmentE).unwrap_err();
        assert!(
            matches!(err, DataError::Core(DnaError::MissingDataFile { .. })),
            "{err:?}"
        );
        std::fs::remove_dir_all(d).unwrap();
    }
}
