//! One history: transport driven by its `(seed, id, Physics)` stream plus the per-history
//! observers (secondary spectra, radial deposit profile, solvation displacement, physchem events).

use crate::config::RunConfig;
use crate::error::SimError;
use dna_core::physchem::PhysChemSite;
use dna_core::rng::{RngStreams, Stage};
use dna_core::vec3::Vec3;
use dna_physics::models::ProcessKind;
use dna_physics::physchem::PhysChemEvent;
use dna_physics::table::ProcessTable;
use dna_physics::transport::{
    HistoryObserver, HistoryOutcome, NoObserver, SolvationEvent, StepEvent, transport_electron,
};

/// Lower edge of the secondary-energy histogram, eV.
pub const SPECTRUM_LOW_EV: f64 = 0.1;
/// Bins per decade of the secondary-energy histogram.
pub const SPECTRUM_BINS_PER_DECADE: usize = 10;
/// Number of decades covered (0.1 eV - 1 MeV).
pub const SPECTRUM_DECADES: usize = 7;
/// Number of histogram bins.
pub const SPECTRUM_BINS: usize = SPECTRUM_BINS_PER_DECADE * SPECTRUM_DECADES;
/// Secondary generations scored separately: 1 (ejected by the primary), 2, 3, and 4 or more.
pub const GENERATION_CLASSES: usize = 4;

/// Lower edge of the radial deposit profile, nm.
pub const RADIAL_LOW_NM: f64 = 0.1;
/// Bins per decade of the radial profile.
pub const RADIAL_BINS_PER_DECADE: usize = 5;
/// Decades covered by the radial profile (0.1 nm - 10 mm).
pub const RADIAL_DECADES: usize = 8;
/// Number of radial bins.
pub const RADIAL_BINS: usize = RADIAL_BINS_PER_DECADE * RADIAL_DECADES;

/// Bin edges in eV (`SPECTRUM_BINS + 1` values, log-spaced).
pub fn spectrum_edges_ev() -> Vec<f64> {
    (0..=SPECTRUM_BINS)
        .map(|i| SPECTRUM_LOW_EV * 10f64.powf(i as f64 / SPECTRUM_BINS_PER_DECADE as f64))
        .collect()
}

/// Radial bin edges in nm (`RADIAL_BINS + 1` values, log-spaced).
pub fn radial_edges_nm() -> Vec<f64> {
    (0..=RADIAL_BINS)
        .map(|i| RADIAL_LOW_NM * 10f64.powf(i as f64 / RADIAL_BINS_PER_DECADE as f64))
        .collect()
}

/// Secondary-electron energies of one generation class in one history: counts per log bin,
/// entries below the first edge and at or above the last one, and the exact energy sum.
#[derive(Debug, Clone, PartialEq)]
pub struct SecondarySpectrum {
    pub counts: Vec<u32>,
    pub underflow: u32,
    pub overflow: u32,
    pub energy_sum_ev: f64,
}

impl SecondarySpectrum {
    fn new() -> Self {
        Self {
            counts: vec![0; SPECTRUM_BINS],
            underflow: 0,
            overflow: 0,
            energy_sum_ev: 0.0,
        }
    }

    /// Number of secondaries recorded (bins + underflow + overflow).
    pub fn n(&self) -> u64 {
        self.counts.iter().map(|&c| u64::from(c)).sum::<u64>()
            + u64::from(self.underflow)
            + u64::from(self.overflow)
    }
}

/// Energy deposited per radial shell around the primary's start point, eV (with the energy
/// deposited closer than the first edge, at or beyond the last one, and the exact sum).
#[derive(Debug, Clone, PartialEq)]
pub struct RadialProfile {
    pub energy_ev: Vec<f64>,
    pub underflow_ev: f64,
    pub overflow_ev: f64,
}

/// Solvation displacement statistics of one history.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SolvationStats {
    pub n: u32,
    /// Sum of the displacement lengths, nm.
    pub sum_displacement_nm: f64,
    /// Sum of the thermalisation energies, eV.
    pub sum_energy_ev: f64,
}

/// Everything observed in one history besides the physics outcome.
pub struct HistoryObserverSet {
    spectrum_edges: Vec<f64>,
    radial_edges: Vec<f64>,
    spectra: Vec<SecondarySpectrum>,
    radial: RadialProfile,
    solvation: SolvationStats,
    physchem: Option<Vec<PhysChemEvent>>,
    origin: Vec3,
}

impl HistoryObserverSet {
    pub fn new(
        spectrum_edges: &[f64],
        radial_edges: &[f64],
        origin: Vec3,
        record_physchem: bool,
    ) -> Self {
        debug_assert_eq!(spectrum_edges.len(), SPECTRUM_BINS + 1);
        debug_assert_eq!(radial_edges.len(), RADIAL_BINS + 1);
        Self {
            spectrum_edges: spectrum_edges.to_vec(),
            radial_edges: radial_edges.to_vec(),
            spectra: (0..GENERATION_CLASSES)
                .map(|_| SecondarySpectrum::new())
                .collect(),
            radial: RadialProfile {
                energy_ev: vec![0.0; RADIAL_BINS],
                underflow_ev: 0.0,
                overflow_ev: 0.0,
            },
            solvation: SolvationStats::default(),
            physchem: record_physchem.then(Vec::new),
            origin,
        }
    }

    fn deposit(&mut self, position: Vec3, energy_ev: f64) {
        if energy_ev == 0.0 {
            return;
        }
        let r = (position - self.origin).norm();
        let pos = self.radial_edges.partition_point(|&e| e <= r);
        if pos == 0 {
            self.radial.underflow_ev += energy_ev;
        } else if pos > RADIAL_BINS {
            self.radial.overflow_ev += energy_ev;
        } else {
            self.radial.energy_ev[pos - 1] += energy_ev;
        }
    }
}

impl HistoryObserver for HistoryObserverSet {
    fn on_interaction(&mut self, event: &StepEvent<'_>) {
        self.deposit(event.position, event.final_state.local_deposit.ev());
        let Some(s) = event.final_state.secondary else {
            return;
        };
        let class = (event.generation as usize).min(GENERATION_CLASSES - 1);
        let sp = &mut self.spectra[class];
        let w = s.energy.ev();
        sp.energy_sum_ev += w;
        let pos = self.spectrum_edges.partition_point(|&e| e <= w);
        if pos == 0 {
            sp.underflow += 1;
        } else if pos > SPECTRUM_BINS {
            sp.overflow += 1;
        } else {
            sp.counts[pos - 1] += 1;
        }
    }

    fn on_solvation(&mut self, event: &SolvationEvent) {
        self.deposit(event.origin, event.energy.ev());
        self.solvation.n += 1;
        self.solvation.sum_displacement_nm += event.displacement_nm.norm();
        self.solvation.sum_energy_ev += event.energy.ev();
    }

    fn on_physchem(&mut self, event: &PhysChemEvent) {
        if let Some(v) = &mut self.physchem {
            v.push(*event);
        }
    }
}

/// Result of one history.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryResult {
    pub id: u64,
    pub outcome: HistoryOutcome,
    /// Secondary spectra by generation class (index 0: generation 1, ..., index 3: 4 or more).
    pub spectra: Vec<SecondarySpectrum>,
    pub radial: RadialProfile,
    pub solvation: SolvationStats,
    /// Physicochemical events in order of occurrence; empty unless
    /// [`RunConfig::record_physchem`].
    pub physchem: Vec<PhysChemEvent>,
}

impl HistoryResult {
    /// Interactions of one process in this history.
    pub fn count(&self, p: ProcessKind) -> u64 {
        self.outcome.counts.get(p)
    }
}

/// Run history `id` of `config` (stream `(master_seed, id, Physics)`).
pub fn run_history(
    table: &ProcessTable,
    config: &RunConfig,
    streams: &RngStreams,
    id: u64,
    spectrum_edges: &[f64],
    radial_edges: &[f64],
) -> Result<HistoryResult, SimError> {
    run_history_with_observer(
        table,
        config,
        streams,
        id,
        spectrum_edges,
        radial_edges,
        &mut NoObserver,
    )
}

/// Run the same scored history with an additional observational receiver.
/// Both receivers see the same transport calls; physics is executed exactly once.
pub fn run_history_with_observer(
    table: &ProcessTable,
    config: &RunConfig,
    streams: &RngStreams,
    id: u64,
    spectrum_edges: &[f64],
    radial_edges: &[f64],
    observer: &mut (impl HistoryObserver + ?Sized),
) -> Result<HistoryResult, SimError> {
    let mut rng = streams.stream(id, Stage::Physics)?;
    let primary = config.primary(id);
    let mut obs = HistoryObserverSet::new(
        spectrum_edges,
        radial_edges,
        primary.position,
        config.record_physchem,
    );
    struct Observers<'a, O: ?Sized>(&'a mut HistoryObserverSet, &'a mut O);
    impl<O: HistoryObserver + ?Sized> HistoryObserver for Observers<'_, O> {
        fn on_interaction(&mut self, event: &StepEvent<'_>) {
            self.0.on_interaction(event);
            self.1.on_interaction(event);
        }
        fn on_solvation(&mut self, event: &SolvationEvent) {
            self.0.on_solvation(event);
            self.1.on_solvation(event);
        }
        fn on_physchem(&mut self, event: &PhysChemEvent) {
            self.0.on_physchem(event);
            self.1.on_physchem(event);
        }
        fn on_physchem_site(&mut self, site: &PhysChemSite) {
            self.0.on_physchem_site(site);
            self.1.on_physchem_site(site);
        }
    }
    let outcome = transport_electron(
        table,
        &config.transport(),
        primary,
        &mut rng,
        &mut Observers(&mut obs, observer),
    )?;
    Ok(HistoryResult {
        id,
        outcome,
        spectra: obs.spectra,
        radial: obs.radial,
        solvation: obs.solvation,
        physchem: obs.physchem.unwrap_or_default(),
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

    #[test]
    fn radial_edges_span_0_1_nm_to_10_mm() {
        let e = radial_edges_nm();
        assert_eq!(e.len(), RADIAL_BINS + 1);
        assert!((e[0] - 0.1).abs() < 1e-15);
        assert!((e[RADIAL_BINS] / 1.0e7 - 1.0).abs() < 1e-12);
        assert!((e[RADIAL_BINS_PER_DECADE] - 1.0).abs() < 1e-12);
    }
}
