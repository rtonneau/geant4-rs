//! Run configuration.

use crate::error::SimError;
use dna_core::units::Energy;
use dna_core::vec3::Vec3;
use dna_physics::transport::{Primary, TransportConfig};
use serde::Serialize;

/// Parameters of a run of `n_histories` identical primaries.
///
/// Energies are eV, positions nm (field names carry the unit, docs/units.md). History `i`
/// (`first_history_id <= i < first_history_id + n_histories`) uses the stream
/// `(master_seed, i, Stage::Physics)`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunConfig {
    pub master_seed: u64,
    pub first_history_id: u64,
    pub n_histories: u64,
    pub primary_energy_ev: f64,
    /// Primary direction (normalised by the transport).
    pub direction: [f64; 3],
    /// Stop the primary below this energy (e.g. to compare with a Geant4 run with a tracking cut).
    pub tracking_cut_ev: Option<f64>,
    /// Stop each history after this many interactions (`Some(1)` reproduces the Geant4 `mfp`
    /// example, which kills the event after the first interaction).
    pub max_interactions: Option<u64>,
    /// Histories reduced together (memory bound; does not affect results, docs/05 E.3).
    pub batch_size: usize,
    /// Worker threads; `None` uses the global rayon pool. Does not affect results.
    pub threads: Option<usize>,
}

impl RunConfig {
    /// Defaults: seed 1, histories 0..n along +z, no tracking cut, batches of 1024.
    pub fn new(primary_energy_ev: f64, n_histories: u64) -> Self {
        Self {
            master_seed: 1,
            first_history_id: 0,
            n_histories,
            primary_energy_ev,
            direction: [0.0, 0.0, 1.0],
            tracking_cut_ev: None,
            max_interactions: None,
            batch_size: 1024,
            threads: None,
        }
    }

    pub fn validate(&self) -> Result<(), SimError> {
        let bad = |m: &str| Err(SimError::InvalidConfig(m.to_owned()));
        if !(self.primary_energy_ev.is_finite() && self.primary_energy_ev > 0.0) {
            return bad("primary_energy_ev must be a positive finite number");
        }
        if self.n_histories == 0 {
            return bad("n_histories must be at least 1");
        }
        if self.batch_size == 0 {
            return bad("batch_size must be at least 1");
        }
        if self.threads == Some(0) {
            return bad("threads must be at least 1");
        }
        if self.max_interactions == Some(0) {
            return bad("max_interactions must be at least 1 when set");
        }
        if let Some(cut) = self.tracking_cut_ev
            && !(cut.is_finite() && cut >= 0.0)
        {
            return bad("tracking_cut_ev must be finite and non-negative");
        }
        Ok(())
    }

    pub(crate) fn primary(&self) -> Primary {
        Primary {
            energy: Energy::from_ev(self.primary_energy_ev),
            position: Vec3::ZERO,
            direction: Vec3::new(self.direction[0], self.direction[1], self.direction[2]),
        }
    }

    pub(crate) fn transport(&self) -> TransportConfig {
        TransportConfig {
            tracking_cut: self.tracking_cut_ev.map(Energy::from_ev),
            max_interactions: self.max_interactions,
            ..TransportConfig::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_validate_and_bad_values_do_not() {
        assert!(RunConfig::new(1000.0, 10).validate().is_ok());
        assert!(RunConfig::new(0.0, 10).validate().is_err());
        assert!(RunConfig::new(f64::NAN, 10).validate().is_err());
        assert!(RunConfig::new(1000.0, 0).validate().is_err());
        let mut c = RunConfig::new(1000.0, 1);
        c.batch_size = 0;
        assert!(c.validate().is_err());
        c.batch_size = 1;
        c.threads = Some(0);
        assert!(c.validate().is_err());
        c.threads = None;
        c.tracking_cut_ev = Some(-1.0);
        assert!(c.validate().is_err());
    }
}
