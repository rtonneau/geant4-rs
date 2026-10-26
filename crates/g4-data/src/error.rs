//! Typed errors of the data layer.

use g4_core::error::DnaError;
use thiserror::Error;

/// Errors raised while locating, reading or interpolating Geant4-DNA tables.
///
/// Energies outside a table are reported as [`DnaError::UnsupportedEnergy`] (wrapped in
/// [`DataError::Core`]): values are never clamped or extrapolated.
#[derive(Debug, Error, PartialEq)]
pub enum DataError {
    /// Energy-range, missing-file, checksum and I/O errors from `g4-core`.
    #[error(transparent)]
    Core(#[from] DnaError),
    /// A token or row of a data file cannot be parsed.
    #[error("{file}:{line}: {message}")]
    Parse {
        file: String,
        line: usize,
        message: String,
    },
    /// The file parses but does not have the structure its reader requires.
    #[error("{file}: invalid table: {message}")]
    InvalidTable { file: String, message: String },
    /// A component (excitation level / ionisation shell) index beyond the table.
    #[error("component index {index} out of range: table has {count} components")]
    ComponentOutOfRange { index: usize, count: usize },
    /// A cumulative-probability argument outside `[0, 1]` (or NaN).
    #[error("probability {0} outside [0, 1]")]
    InvalidProbability(f64),
    /// The Geant4 lookup is undefined for this argument (it reads past the end of a vector).
    #[error("lookup undefined: {0}")]
    UndefinedLookup(String),
    /// The data directory does not look like a G4EMLOW subset (no `dna/` folder).
    #[error("invalid data directory {path}: {reason}")]
    InvalidDataDir { path: String, reason: String },
}
