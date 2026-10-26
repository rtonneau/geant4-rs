//! Orchestration of geant4-rs physics runs (milestone M2): deterministic parallel histories,
//! scorers kept separate from the physics, and JSON output with provenance.
//!
//! * [`config`]: the run parameters, [`RunConfig`].
//! * [`history`]: one history on its own `(seed, id, Physics)` stream, plus the secondary spectrum.
//! * [`scoring`]: sequential, fixed-order reduction into `RunSummary`.
//! * [`run`]: rayon batches, thread-count independent, and the JSON `RunOutput`.
//!
//! Production output APIs accept `g4_physics::table::VerifiedProcessTable`, constructed from
//! a `g4_data::VerifiedDataSet` byte snapshot. They never accept a separate manifest. Raw
//! history/summary APIs remain available for model tests; the explicitly named
//! `run_unverified_for_testing` family emits an unverified marker and no dataset identity.
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
