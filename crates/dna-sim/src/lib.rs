//! Orchestration of geant4-rs physics runs (milestone M2): deterministic parallel histories,
//! scorers kept separate from the physics, and JSON output with provenance.
//!
//! * [`config`]: the run parameters, [`RunConfig`].
//! * [`history`]: one history on its own `(seed, id, Physics)` stream, plus the secondary spectrum.
//! * [`scoring`]: sequential, fixed-order reduction into `RunSummary`.
//! * [`run`]: rayon batches, thread-count independent, and the JSON `RunOutput`.
//!
//! Chemistry and the CLI arrive in M6-M8; this crate is where they will attach (docs/03).

pub mod config;
pub mod error;
pub mod history;
pub mod physchem_scoring;
pub mod run;
pub mod scoring;

pub use config::RunConfig;
pub use error::SimError;
