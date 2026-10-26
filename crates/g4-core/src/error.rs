//! Typed errors shared across the workspace.

use thiserror::Error;

/// Errors raised by `g4-core`. Physics crates add their own and wrap this one.
#[derive(Debug, Error, PartialEq)]
pub enum DnaError {
    /// A model was asked for a value outside its validated energy range (never extrapolated).
    #[error("energy {energy_ev} eV outside supported range [{min_ev}, {max_ev}] eV")]
    UnsupportedEnergy {
        energy_ev: f64,
        min_ev: f64,
        max_ev: f64,
    },
    /// History ids must fit in 62 bits so that `(id << 2) | stage` is a unique ChaCha stream.
    #[error("history id {history_id} exceeds the maximum {max}")]
    HistoryIdOutOfRange { history_id: u64, max: u64 },
    /// A data file listed in the manifest (or requested by a model) does not exist.
    #[error("missing data file {path}")]
    MissingDataFile { path: String },
    /// A data file exists but differs from the manifest (size or SHA-256).
    #[error("data file {path} does not match the manifest: expected {expected}, found {actual}")]
    ChecksumMismatch {
        path: String,
        expected: String,
        actual: String,
    },
    /// The manifest itself cannot be parsed or is inconsistent.
    #[error("invalid data manifest: {0}")]
    InvalidManifest(String),
    /// A file could not be read.
    #[error("cannot read {path}: {message}")]
    Io { path: String, message: String },
}
