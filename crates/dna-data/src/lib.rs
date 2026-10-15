//! Geant4-DNA data layer (milestone M1): readers and Geant4-exact interpolation for the
//! `G4EMLOW` `dna/` cross-section and differential tables.
//!
//! * [`parse`]: tolerant numeric table reader (LF/CRLF, spaces/tabs, `#` comments).
//! * [`xs`]: total/partial cross sections with log-log (`G4DNACrossSectionDataSet`) or lin-lin
//!   (Sanche) interpolation; energies outside the table are an error, never clamped.
//! * [`blocks`]: angular and ionisation cumulative tables and their deterministic lookups.
//! * [`interp`]: the 1-D and `QuadInterpolator` formulas.
//! * [`catalog`]: the 20 committed files with their transcribed units, columns and consumers.
//! * [`datadir`]: `G4RS_DATA_DIR` resolution, optional checksum verification, loading.
//!
//! Nothing here draws random numbers or transports particles (M2). Every table type is immutable
//! after construction, hence `Send + Sync`; share with `Arc`.

pub mod blocks;
pub mod catalog;
pub mod datadir;
pub mod error;
pub mod interp;
pub mod parse;
pub mod xs;

pub use error::DataError;

use dna_core::error::DnaError;
use std::path::Path;

/// Read a whole data file as text; a missing file is `MissingDataFile`.
pub(crate) fn read_text(path: &Path) -> Result<String, DataError> {
    std::fs::read_to_string(path).map_err(|e| {
        DataError::Core(match e.kind() {
            std::io::ErrorKind::NotFound => DnaError::MissingDataFile {
                path: path.display().to_string(),
            },
            _ => DnaError::Io {
                path: path.display().to_string(),
                message: e.to_string(),
            },
        })
    })
}
