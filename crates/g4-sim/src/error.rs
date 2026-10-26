//! Errors of the orchestration layer.

#[cfg(feature = "chemistry")]
use g4_chemistry::ChemistryError;
use g4_core::error::DnaError;
use g4_data::DataError;
use g4_physics::PhysicsError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SimError {
    #[cfg(feature = "chemistry")]
    #[error(transparent)]
    SpatialDiagnostic(#[from] crate::spatial_diagnostic::DiagnosticError),
    #[error(transparent)]
    Physics(#[from] PhysicsError),
    #[error(transparent)]
    Core(#[from] DnaError),
    #[error(transparent)]
    Data(#[from] DataError),
    #[cfg(feature = "chemistry")]
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    #[error("invalid run configuration: {0}")]
    InvalidConfig(String),
    #[error("cannot build the thread pool: {0}")]
    ThreadPool(String),
    #[error("JSON serialisation failed: {0}")]
    Json(String),
    #[error("cannot write {path}: {message}")]
    Io { path: String, message: String },
}
