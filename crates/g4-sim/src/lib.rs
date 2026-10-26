//! Orchestration of geant4-rs physics runs (milestone M2): deterministic parallel histories,
//! scorers kept separate from the physics, and JSON output with provenance.
//!
//! * [`config`]: the run parameters, [`RunConfig`].
//! * [`history`]: one history on its own `(seed, id, Physics)` stream, plus the secondary spectrum.
//! * [`scoring`]: sequential, fixed-order reduction into `RunSummary`.
//! * [`run`]: rayon batches, thread-count independent, and the JSON `RunOutput`.
//!
//! The default `chemistry` feature adds coupled physicochemical runs, scoring and diagnostics.
//! Without it, physical transport and raw physicochemical handoff recording remain available.

pub mod config;
pub mod error;
pub mod history;
#[cfg(feature = "chemistry")]
pub mod physchem_scoring;
pub mod run;
pub mod scoring;
#[cfg(feature = "chemistry")]
pub mod spatial_diagnostic;

pub use config::RunConfig;
pub use error::SimError;
