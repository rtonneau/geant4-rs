//! Errors of the orchestration layer.

use dna_chemistry::ChemistryError;
use dna_core::error::DnaError;
use dna_data::DataError;
use dna_physics::PhysicsError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SimError {
    #[error(transparent)]
    Physics(#[from] PhysicsError),
    #[error(transparent)]
    Core(#[from] DnaError),
    #[error(transparent)]
    Data(#[from] DataError),
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
