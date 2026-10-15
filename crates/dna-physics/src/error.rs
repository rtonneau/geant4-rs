//! Errors of the physics layer.

use dna_core::error::DnaError;
use dna_data::DataError;
use thiserror::Error;

/// Errors raised by models, the process table and the transport loop.
#[derive(Debug, Error, PartialEq)]
pub enum PhysicsError {
    /// Energy outside a validated window (never clamped or extrapolated), missing data, ...
    #[error(transparent)]
    Core(#[from] DnaError),
    /// A table lookup failed (including `UndefinedLookup` where Geant4 reads past a vector).
    #[error(transparent)]
    Data(#[from] DataError),
    /// A history violated the energy ledger by more than the hard limit.
    #[error(
        "energy ledger violated: residual {residual_ev:e} eV on E_in = {e_in_ev} eV (limit {limit_relative:e} relative)"
    )]
    LedgerViolation {
        residual_ev: f64,
        e_in_ev: f64,
        limit_relative: f64,
    },
    /// The step limit guarding against runaway histories was reached.
    #[error("history exceeded the step limit of {max_steps}")]
    StepLimitExceeded { max_steps: u64 },
    /// The primary direction is zero or not finite.
    #[error("primary direction must be a non-zero finite vector")]
    InvalidDirection,
    /// The table was built without a model for a process the caller asked for.
    #[error("no {0} model registered")]
    MissingModel(&'static str),
    /// A model chain is not ordered by ascending energy window.
    #[error("invalid model chain: {0}")]
    InvalidModelChain(String),
    /// A data table does not cover the model window it is meant to serve.
    #[error(
        "{table} covers [{table_min_ev}, {table_max_ev}] eV but the {model} window is [{min_ev}, {max_ev}] eV"
    )]
    TableDoesNotCoverWindow {
        table: &'static str,
        model: &'static str,
        table_min_ev: f64,
        table_max_ev: f64,
        min_ev: f64,
        max_ev: f64,
    },
    /// A table is shorter than its model window and its last node is not zero, so Geant4's
    /// constant clamp beyond the table would be a non-zero extrapolation.
    #[error(
        "{table}: table ends at {last_energy_ev} eV with value {last_value_nm2} nm^2 (not zero) \
         but the model window extends further"
    )]
    TableTailNotZero {
        table: &'static str,
        last_energy_ev: f64,
        last_value_nm2: f64,
    },
    /// The cross sections of all processes vanish at this energy and no solvation applies: the
    /// track would never interact. Reported instead of ending the history silently.
    #[error("no process can act on an electron of {energy_ev} eV")]
    NoProcessAvailable { energy_ev: f64 },
    /// A sampled final state is not physical (non-finite direction/energy).
    #[error("{model}: non-physical final state ({what})")]
    NonPhysicalFinalState {
        model: &'static str,
        what: &'static str,
    },
}
