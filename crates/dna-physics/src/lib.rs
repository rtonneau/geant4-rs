//! Geant4-DNA electron physics in liquid water (milestone M2): cross-section models,
//! final-state sampling, the process table and the track-structure transport of one electron.
//!
//! Scope (decisions Q1/Q2, docs/08 M2): `G4EmDNAPhysics_option2` electrons, primary energy
//! 7.4 eV - 1 MeV, Champion elastic + Born excitation + Born1 ionisation, infinite medium,
//! secondaries recorded but not transported. Sanche, Melton and solvation are M3. No chemistry,
//! no scoring and no parallelism here: those live in `dna-sim` (docs/03).
//!
//! * [`models`]: the three models (`CrossSectionModel` trait + enum dispatch).
//! * [`table`]: [`ProcessTable`](table::ProcessTable) and model-window selection.
//! * [`transport`]: the single-electron loop, energy ledger and observer hook.
//! * [`water`]: constants with their Geant4 sources (number density, structure energies).

pub mod direction;
pub mod error;
pub mod models;
pub mod sampling;
pub mod table;
pub mod testing;
pub mod transport;
pub mod water;

pub use error::PhysicsError;
