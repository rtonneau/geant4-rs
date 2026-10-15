//! Scorers: reduce per-history results into run statistics.
//!
//! Scoring consumes [`HistoryResult`]s only; it knows nothing about models (docs/03 rule 4).
//! Every accumulation is a plain sequential fold in history order, so the sums are identical for
//! any thread count (docs/05 E.3). Means use Welford's algorithm; standard errors are
//! `s/sqrt(n)` with the unbiased sample variance.
//!
//! # Definitions
//! * **First-step free path**: length of the first step of each history, all at the primary
//!   energy, hence an unbiased sample of `1/Sigma_tot(E0)` (exponential): its mean has standard
//!   error `mean/sqrt(n)`. This is what the Geant4 `mfp` example measures.
//! * **Mean path per interaction** of a process: total track length of all histories divided
//!   by the number of interactions of that process. Averaged over the slowing-down track, so it
//!   is *not* a mean free path at a single energy.
//! * **Track length**: total path of the primary; **range**: distance start-end;
//!   **projected range**: displacement along the initial direction.
//! * **Deposited energy** (docs/01 A.4): excitation energies + binding energies booked locally by
//!   the primary track; secondaries are recorded, not transported, so their energy is *not*
//!   deposited here; `remaining_untracked` is what is left at termination.

use crate::history::{HistoryResult, SPECTRUM_BINS, spectrum_edges_ev};
use dna_physics::models::ProcessKind;
use dna_physics::transport::{QuirkCounts, Termination};
use serde::Serialize;

/// Mean and standard error of a sample.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct MeanStat {
    pub n: u64,
    pub mean: f64,
    /// Standard error of the mean (0 for `n < 2`).
    pub std_err: f64,
}

/// Welford accumulator.
#[derive(Debug, Clone, Copy, Default)]
struct Welford {
    n: u64,
    mean: f64,
    m2: f64,
}

impl Welford {
    fn add(&mut self, x: f64) {
        self.n += 1;
        let d = x - self.mean;
        self.mean += d / self.n as f64;
        self.m2 += d * (x - self.mean);
    }
    fn stat(&self) -> MeanStat {
        let std_err = if self.n > 1 {
            (self.m2 / (self.n as f64 - 1.0) / self.n as f64).sqrt()
        } else {
            0.0
        };
        MeanStat {
            n: self.n,
            mean: self.mean,
            std_err,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ProcessSummary {
    pub process: ProcessKind,
    /// Interactions of this process summed over all histories.
    pub interactions: u64,
    /// Interactions per history (mean, standard error).
    pub per_history: MeanStat,
    /// Total track length / interactions, nm (see module docs).
    pub mean_path_per_interaction_nm: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FirstStepSummary {
    /// Mean of the first free path, nm: estimator of `1 / Sigma_tot(E0)`.
    pub mean_free_path_nm: MeanStat,
    /// Histories whose first interaction was of each process, in `ProcessKind::ALL` order.
    pub first_process_counts: [u64; 3],
    /// `first_process_counts / n` (binomial; standard error `sqrt(p(1-p)/n)`).
    pub first_process_fractions: [f64; 3],
    /// Histories with no step at all (immediate termination).
    pub histories_without_steps: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EnergySummary {
    pub e_in_ev: f64,
    pub deposited_excitation_ev: MeanStat,
    pub deposited_binding_ev: MeanStat,
    /// Excitation + binding.
    pub deposited_total_ev: MeanStat,
    pub secondaries_ev: MeanStat,
    pub remaining_untracked_ev: MeanStat,
    /// Largest `|E_in - sum(parts)| / E_in` over all histories (energy ledger, V7).
    pub max_abs_ledger_residual_relative: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TerminationCounts {
    pub interaction_limit: u64,
    pub tracking_cut: u64,
    pub below_lowest_window: u64,
    pub no_inelastic_channel: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpectrumSummary {
    /// Log-spaced bin edges, eV (`counts.len() + 1`).
    pub edges_ev: Vec<f64>,
    pub counts: Vec<u64>,
    pub underflow: u64,
    pub overflow: u64,
    /// Number of secondaries recorded.
    pub n_secondaries: u64,
    pub mean_energy_ev: Option<f64>,
}

/// Everything scored in one run.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunSummary {
    pub n_histories: u64,
    pub terminations: TerminationCounts,
    pub processes: Vec<ProcessSummary>,
    pub first_step: FirstStepSummary,
    pub track_length_nm: MeanStat,
    pub range_nm: MeanStat,
    pub projected_range_nm: MeanStat,
    pub energy: EnergySummary,
    pub excitation_levels: [u64; 5],
    pub ionisation_shells: [u64; 5],
    pub secondary_spectrum: SpectrumSummary,
    pub quirks: QuirkCounts,
}

/// Sequential reducer; feed it results in history order.
#[derive(Debug, Clone)]
pub struct Scorer {
    e_in: f64,
    n: u64,
    terminations: [u64; 4],
    interactions: [u64; 3],
    per_history: [Welford; 3],
    total_track_length_nm: f64,
    first_length: Welford,
    first_process: [u64; 3],
    no_steps: u64,
    track: Welford,
    range: Welford,
    projected: Welford,
    dep_exc: Welford,
    dep_bind: Welford,
    dep_total: Welford,
    secondaries: Welford,
    remaining: Welford,
    max_residual: f64,
    levels: [u64; 5],
    shells: [u64; 5],
    spectrum: Vec<u64>,
    underflow: u64,
    overflow: u64,
    n_secondaries: u64,
    secondary_energy_sum: f64,
    quirks: QuirkCounts,
}

impl Scorer {
    pub fn new(e_in_ev: f64) -> Self {
        Self {
            e_in: e_in_ev,
            n: 0,
            terminations: [0; 4],
            interactions: [0; 3],
            per_history: [Welford::default(); 3],
            total_track_length_nm: 0.0,
            first_length: Welford::default(),
            first_process: [0; 3],
            no_steps: 0,
            track: Welford::default(),
            range: Welford::default(),
            projected: Welford::default(),
            dep_exc: Welford::default(),
            dep_bind: Welford::default(),
            dep_total: Welford::default(),
            secondaries: Welford::default(),
            remaining: Welford::default(),
            max_residual: 0.0,
            levels: [0; 5],
            shells: [0; 5],
            spectrum: vec![0; SPECTRUM_BINS],
            underflow: 0,
            overflow: 0,
            n_secondaries: 0,
            secondary_energy_sum: 0.0,
            quirks: QuirkCounts::default(),
        }
    }

    pub fn add(&mut self, r: &HistoryResult) {
        let o = &r.outcome;
        self.n += 1;
        self.terminations[match o.termination {
            Termination::InteractionLimit => 0,
            Termination::TrackingCut => 1,
            Termination::BelowLowestWindow => 2,
            Termination::NoInelasticChannel => 3,
        }] += 1;
        for p in ProcessKind::ALL {
            let c = o.counts.get(p);
            self.interactions[p.index()] += c;
            self.per_history[p.index()].add(c as f64);
        }
        self.total_track_length_nm += o.track_length_nm;
        match o.first_step {
            Some(f) => {
                self.first_length.add(f.length_nm);
                self.first_process[f.process.index()] += 1;
            }
            None => self.no_steps += 1,
        }
        self.track.add(o.track_length_nm);
        self.range.add(o.range_nm());
        self.projected.add(o.projected_range_nm());
        let l = &o.ledger;
        self.dep_exc.add(l.deposited_excitation);
        self.dep_bind.add(l.deposited_binding);
        self.dep_total.add(l.deposited());
        self.secondaries.add(l.secondaries);
        self.remaining.add(l.remaining_untracked);
        self.max_residual = self.max_residual.max(l.residual_relative().abs());
        for i in 0..5 {
            self.levels[i] += o.excitation_levels[i];
            self.shells[i] += o.ionisation_shells[i];
        }
        for (acc, c) in self.spectrum.iter_mut().zip(&r.spectrum.counts) {
            *acc += u64::from(*c);
        }
        self.underflow += u64::from(r.spectrum.underflow);
        self.overflow += u64::from(r.spectrum.overflow);
        self.n_secondaries += o.n_secondaries;
        self.secondary_energy_sum += r.spectrum.energy_sum_ev;
        self.quirks.accumulate(&o.quirks);
    }

    pub fn finish(&self) -> RunSummary {
        let n = self.n as f64;
        let processes = ProcessKind::ALL
            .iter()
            .map(|&p| ProcessSummary {
                process: p,
                interactions: self.interactions[p.index()],
                per_history: self.per_history[p.index()].stat(),
                mean_path_per_interaction_nm: (self.interactions[p.index()] > 0)
                    .then(|| self.total_track_length_nm / self.interactions[p.index()] as f64),
            })
            .collect();
        RunSummary {
            n_histories: self.n,
            terminations: TerminationCounts {
                interaction_limit: self.terminations[0],
                tracking_cut: self.terminations[1],
                below_lowest_window: self.terminations[2],
                no_inelastic_channel: self.terminations[3],
            },
            processes,
            first_step: FirstStepSummary {
                mean_free_path_nm: self.first_length.stat(),
                first_process_counts: self.first_process,
                first_process_fractions: self.first_process.map(|c| c as f64 / n),
                histories_without_steps: self.no_steps,
            },
            track_length_nm: self.track.stat(),
            range_nm: self.range.stat(),
            projected_range_nm: self.projected.stat(),
            energy: EnergySummary {
                e_in_ev: self.e_in,
                deposited_excitation_ev: self.dep_exc.stat(),
                deposited_binding_ev: self.dep_bind.stat(),
                deposited_total_ev: self.dep_total.stat(),
                secondaries_ev: self.secondaries.stat(),
                remaining_untracked_ev: self.remaining.stat(),
                max_abs_ledger_residual_relative: self.max_residual,
            },
            excitation_levels: self.levels,
            ionisation_shells: self.shells,
            secondary_spectrum: SpectrumSummary {
                edges_ev: spectrum_edges_ev(),
                counts: self.spectrum.clone(),
                underflow: self.underflow,
                overflow: self.overflow,
                n_secondaries: self.n_secondaries,
                mean_energy_ev: (self.n_secondaries > 0)
                    .then(|| self.secondary_energy_sum / self.n_secondaries as f64),
            },
            quirks: self.quirks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn welford_matches_textbook_mean_and_standard_error() {
        let mut w = Welford::default();
        for x in [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0] {
            w.add(x);
        }
        let s = w.stat();
        assert_eq!(s.n, 8);
        assert_relative_eq!(s.mean, 5.0);
        // sample variance = 32/7, se = sqrt(32/7/8)
        assert_relative_eq!(s.std_err, (32.0f64 / 7.0 / 8.0).sqrt(), epsilon = 1e-14);
    }

    #[test]
    fn single_sample_has_zero_standard_error() {
        let mut w = Welford::default();
        w.add(3.0);
        assert_eq!(w.stat().std_err, 0.0);
    }
}
