//! One history: transport driven by its `(seed, id, Physics)` stream plus the per-history
//! secondary-spectrum observer.

use crate::config::RunConfig;
use crate::error::SimError;
use dna_core::rng::{RngStreams, Stage};
use dna_physics::table::ProcessTable;
use dna_physics::transport::{HistoryObserver, HistoryOutcome, StepEvent, transport_electron};

/// Lower edge of the secondary-energy histogram, eV.
pub const SPECTRUM_LOW_EV: f64 = 0.1;
/// Bins per decade of the secondary-energy histogram.
pub const SPECTRUM_BINS_PER_DECADE: usize = 10;
/// Number of decades covered (0.1 eV - 1 MeV).
pub const SPECTRUM_DECADES: usize = 7;
/// Number of histogram bins.
pub const SPECTRUM_BINS: usize = SPECTRUM_BINS_PER_DECADE * SPECTRUM_DECADES;

/// Bin edges in eV (`SPECTRUM_BINS + 1` values, log-spaced).
pub fn spectrum_edges_ev() -> Vec<f64> {
    (0..=SPECTRUM_BINS)
        .map(|i| SPECTRUM_LOW_EV * 10f64.powf(i as f64 / SPECTRUM_BINS_PER_DECADE as f64))
        .collect()
}

/// Secondary-electron energies of one history: counts per log bin, entries below the first edge
/// and at or above the last one, and the exact energy sum.
#[derive(Debug, Clone, PartialEq)]
pub struct SecondarySpectrum {
    pub counts: Vec<u32>,
    pub underflow: u32,
    pub overflow: u32,
    pub energy_sum_ev: f64,
}

/// Observer filling a [`SecondarySpectrum`].
pub struct SpectrumObserver {
    edges: Vec<f64>,
    spectrum: SecondarySpectrum,
}

impl SpectrumObserver {
    pub fn new(edges: Vec<f64>) -> Self {
        debug_assert_eq!(edges.len(), SPECTRUM_BINS + 1);
        Self {
            edges,
            spectrum: SecondarySpectrum {
                counts: vec![0; SPECTRUM_BINS],
                underflow: 0,
                overflow: 0,
                energy_sum_ev: 0.0,
            },
        }
    }

    pub fn into_spectrum(self) -> SecondarySpectrum {
        self.spectrum
    }
}

impl HistoryObserver for SpectrumObserver {
    fn on_interaction(&mut self, event: &StepEvent<'_>) {
        let Some(s) = event.final_state.secondary else {
            return;
        };
        let w = s.energy.ev();
        self.spectrum.energy_sum_ev += w;
        let pos = self.edges.partition_point(|&e| e <= w);
        if pos == 0 {
            self.spectrum.underflow += 1;
        } else if pos > SPECTRUM_BINS {
            self.spectrum.overflow += 1;
        } else {
            self.spectrum.counts[pos - 1] += 1;
        }
    }
}

/// Result of one history.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryResult {
    pub id: u64,
    pub outcome: HistoryOutcome,
    pub spectrum: SecondarySpectrum,
}

/// Run history `id` of `config` (stream `(master_seed, id, Physics)`).
pub fn run_history(
    table: &ProcessTable,
    config: &RunConfig,
    streams: &RngStreams,
    id: u64,
    edges: &[f64],
) -> Result<HistoryResult, SimError> {
    let mut rng = streams.stream(id, Stage::Physics)?;
    let mut observer = SpectrumObserver::new(edges.to_vec());
    let outcome = transport_electron(
        table,
        &config.transport(),
        config.primary(),
        &mut rng,
        &mut observer,
    )?;
    Ok(HistoryResult {
        id,
        outcome,
        spectrum: observer.into_spectrum(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_are_log_spaced_from_0_1_ev_to_1_mev() {
        let e = spectrum_edges_ev();
        assert_eq!(e.len(), SPECTRUM_BINS + 1);
        assert!((e[0] - 0.1).abs() < 1e-15);
        assert!((e[SPECTRUM_BINS] - 1.0e6).abs() < 1e-6);
        assert!(e.windows(2).all(|w| w[1] > w[0]));
        assert!((e[10] - 1.0).abs() < 1e-12);
    }
}
