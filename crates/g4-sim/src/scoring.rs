//! Scorers: reduce per-history results into run statistics.
//!
//! Scoring consumes [`HistoryResult`]s only; it knows nothing about models (docs/03 rule 4).
//! Every accumulation is a plain sequential fold in history order, so the sums are identical for
//! any thread count (docs/05 E.3). Means use Welford's algorithm; standard errors are
//! `s/sqrt(n)` with the unbiased sample variance; ratios of totals use the delta method on
//! streaming (co)variances.
//!
//! # Definitions
//! * **First-step free path**: length of the first step of each history, all at the primary
//!   energy, hence an unbiased sample of `1/Sigma_tot(E0)` (exponential): its mean has standard
//!   error `mean/sqrt(n)`. This is what the Geant4 `mfp` example measures.
//! * **Mean path per interaction** of a process: total path of **all** followed tracks divided by
//!   the number of interactions of that process. Averaged over the whole cascade, so it is *not*
//!   a mean free path at a single energy.
//! * **Track length** (full slowing-down path, the CSDA-like quantity of the primary): total
//!   path of the primary track; **total track length**: sum over the primary and all transported
//!   secondaries; **range**: distance start to end of the primary; **projected range**:
//!   displacement of the primary along the initial direction.
//! * **Deposited energy** (docs/01 A.4): the sum of the local deposits of every track
//!   (excitation, vibrational, binding, attachment, solvation). In the infinite medium with all
//!   secondaries transported it equals `E_in` for every history, so its *distribution* is a
//!   delta function; the informative distributions are the split by category and the radial
//!   profile around the primary's start point.
//! * **Solvations per 100 eV**: `100 N_solv / E_dep`, reported as the ratio of totals (with
//!   delta-method error) and, as Geant4's `chem6` does for G (docs/01 A.4), as the mean of the
//!   per-history ratio.
//! * **W-like quantity**: `E_in / N_ionisation`, the mean energy spent per ionisation, as the
//!   ratio of totals and as the mean of per-history ratios. It includes the energy that goes
//!   into excitations, vibrations, sub-ionisation electrons and attachment, as the physical W
//!   does, but it is *not* the experimental W (which is defined for the gas phase and for
//!   ion pairs collected in an electric field).

use crate::history::{
    GENERATION_CLASSES, HistoryResult, RADIAL_BINS, SPECTRUM_BINS, SecondarySpectrum,
    radial_edges_nm, spectrum_edges_ev,
};
use g4_physics::models::{ProcessKind, SANCHE_LEVELS};
use g4_physics::transport::{QuirkCounts, Termination};
use serde::Serialize;

/// Mean and standard error of a sample.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct MeanStat {
    pub n: u64,
    pub mean: f64,
    /// Standard error of the mean (0 for `n < 2`).
    pub std_err: f64,
}

/// Ratio of two totals with its delta-method standard error.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RatioStat {
    pub n: u64,
    /// `sum(x) / sum(y)`.
    pub ratio: f64,
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

/// Streaming means and co-moments of paired samples (West/Welford), for ratio estimators.
#[derive(Debug, Clone, Copy, Default)]
struct PairedWelford {
    n: u64,
    mx: f64,
    my: f64,
    cxx: f64,
    cxy: f64,
    cyy: f64,
}

impl PairedWelford {
    fn add(&mut self, x: f64, y: f64) {
        self.n += 1;
        let n = self.n as f64;
        let dx = x - self.mx;
        let dy = y - self.my;
        self.mx += dx / n;
        self.my += dy / n;
        self.cxx += dx * (x - self.mx);
        self.cxy += dx * (y - self.my);
        self.cyy += dy * (y - self.my);
    }

    /// `None` if no pair or the denominator mean is zero.
    fn ratio(&self) -> Option<RatioStat> {
        if self.n == 0 || self.my == 0.0 {
            return None;
        }
        let r = self.mx / self.my;
        let std_err = if self.n > 1 {
            let n = self.n as f64;
            let var = (self.cxx - 2.0 * r * self.cxy + r * r * self.cyy) / (n - 1.0);
            (var.max(0.0) / n).sqrt() / self.my.abs()
        } else {
            0.0
        };
        Some(RatioStat {
            n: self.n,
            ratio: r,
            std_err,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ProcessSummary {
    pub process: ProcessKind,
    /// Interactions of this process summed over all histories.
    pub interactions: u64,
    /// Interactions per history (mean, standard error).
    pub per_history: MeanStat,
    /// Total path of all followed tracks / interactions, nm (see module docs).
    pub mean_path_per_interaction_nm: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SolvationSummary {
    /// Solvation events summed over all histories.
    pub events: u64,
    pub per_history: MeanStat,
    /// Mean length of the displacement of the solvated electron, nm.
    pub mean_displacement_nm: Option<f64>,
    /// Mean thermalisation energy, eV.
    pub mean_energy_ev: Option<f64>,
    /// Solvation events per 100 eV deposited, ratio of totals.
    pub per_100_ev: Option<RatioStat>,
    /// Same, as the mean of the per-history ratio (Geant4 `chem6` convention, docs/01 A.4).
    pub per_100_ev_mean_of_ratios: MeanStat,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FirstStepSummary {
    /// Mean of the first free path, nm: estimator of `1 / Sigma_tot(E0)`.
    pub mean_free_path_nm: MeanStat,
    /// Histories whose first interaction was of each process, in `ProcessKind::ALL` order.
    pub first_process_counts: [u64; ProcessKind::COUNT],
    /// `first_process_counts / n` (binomial; standard error `sqrt(p(1-p)/n)`).
    pub first_process_fractions: [f64; ProcessKind::COUNT],
    /// Histories with no cross-section step at all (the primary was solvated at once).
    pub histories_without_steps: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TrackSummary {
    /// Full slowing-down path of the primary, nm.
    pub primary_track_length_nm: MeanStat,
    /// Sum of the paths of all followed tracks, nm.
    pub total_track_length_nm: MeanStat,
    pub range_nm: MeanStat,
    pub projected_range_nm: MeanStat,
    pub tracks_per_history: MeanStat,
    pub secondaries_per_history: MeanStat,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EnergySummary {
    pub e_in_ev: f64,
    pub deposited_excitation_ev: MeanStat,
    pub deposited_vibrational_ev: MeanStat,
    pub deposited_binding_ev: MeanStat,
    pub deposited_attachment_ev: MeanStat,
    pub deposited_solvation_ev: MeanStat,
    /// Sum of the five categories.
    pub deposited_total_ev: MeanStat,
    pub untracked_secondaries_ev: MeanStat,
    pub remaining_untracked_ev: MeanStat,
    /// Largest `|E_in - sum(parts)| / E_in` over all histories (energy ledger, V7).
    pub max_abs_ledger_residual_relative: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WSummary {
    pub ionisations_per_history: MeanStat,
    /// `E_in / N_ion`, ratio of totals, eV.
    pub w_ev_ratio_of_totals: Option<RatioStat>,
    /// Mean of the per-history `E_in / N_ion` over histories with `N_ion > 0`, eV.
    pub w_ev_mean_of_ratios: MeanStat,
    pub histories_without_ionisation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TerminationCounts {
    pub completed: u64,
    pub interaction_limit: u64,
    pub tracking_cut: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpectrumSummary {
    pub counts: Vec<u64>,
    pub underflow: u64,
    pub overflow: u64,
    /// Number of secondaries recorded.
    pub n_secondaries: u64,
    pub mean_energy_ev: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SecondarySpectraSummary {
    /// Log-spaced bin edges, eV (`counts.len() + 1`).
    pub edges_ev: Vec<f64>,
    /// Generation 1 (ejected by the primary), 2, 3, and 4 or more.
    pub by_generation: Vec<SpectrumSummary>,
    /// All generations together.
    pub all: SpectrumSummary,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RadialSummary {
    /// Log-spaced radial edges, nm (`mean_energy_ev.len() + 1`).
    pub edges_nm: Vec<f64>,
    /// Energy deposited per shell per history, eV.
    pub mean_energy_ev: Vec<f64>,
    pub mean_underflow_ev: f64,
    pub mean_overflow_ev: f64,
}

/// Everything scored in one run.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunSummary {
    pub n_histories: u64,
    pub terminations: TerminationCounts,
    pub processes: Vec<ProcessSummary>,
    pub solvation: SolvationSummary,
    pub first_step: FirstStepSummary,
    pub tracks: TrackSummary,
    pub energy: EnergySummary,
    pub w_value: WSummary,
    pub excitation_levels: [u64; 5],
    pub ionisation_shells: [u64; 5],
    pub vibrational_levels: [u64; SANCHE_LEVELS],
    pub secondary_spectra: SecondarySpectraSummary,
    pub radial_deposit: RadialSummary,
    pub quirks: QuirkCounts,
}

#[derive(Debug, Clone)]
struct SpectrumAcc {
    counts: Vec<u64>,
    underflow: u64,
    overflow: u64,
    n: u64,
    energy_sum: f64,
}

impl SpectrumAcc {
    fn new() -> Self {
        Self {
            counts: vec![0; SPECTRUM_BINS],
            underflow: 0,
            overflow: 0,
            n: 0,
            energy_sum: 0.0,
        }
    }
    fn add(&mut self, s: &SecondarySpectrum) {
        for (a, c) in self.counts.iter_mut().zip(&s.counts) {
            *a += u64::from(*c);
        }
        self.underflow += u64::from(s.underflow);
        self.overflow += u64::from(s.overflow);
        self.n += s.n();
        self.energy_sum += s.energy_sum_ev;
    }
    fn summary(&self) -> SpectrumSummary {
        SpectrumSummary {
            counts: self.counts.clone(),
            underflow: self.underflow,
            overflow: self.overflow,
            n_secondaries: self.n,
            mean_energy_ev: (self.n > 0).then(|| self.energy_sum / self.n as f64),
        }
    }
}

/// Sequential reducer; feed it results in history order.
#[derive(Debug, Clone)]
pub struct Scorer {
    e_in: f64,
    n: u64,
    terminations: [u64; 3],
    interactions: [u64; ProcessKind::COUNT],
    per_history: [Welford; ProcessKind::COUNT],
    total_track_length_sum_nm: f64,
    first_length: Welford,
    first_process: [u64; ProcessKind::COUNT],
    no_steps: u64,
    track: Welford,
    total_track: Welford,
    range: Welford,
    projected: Welford,
    tracks: Welford,
    secondaries_n: Welford,
    dep_exc: Welford,
    dep_vib: Welford,
    dep_bind: Welford,
    dep_att: Welford,
    dep_solv: Welford,
    dep_total: Welford,
    untracked_sec: Welford,
    remaining: Welford,
    max_residual: f64,
    levels: [u64; 5],
    shells: [u64; 5],
    vib_levels: [u64; SANCHE_LEVELS],
    solvation_events: u64,
    solvation_per_history: Welford,
    solvation_displacement_sum: f64,
    solvation_energy_sum: f64,
    solvation_per_100ev: PairedWelford,
    solvation_per_100ev_each: Welford,
    ionisations: Welford,
    w_totals: PairedWelford,
    w_each: Welford,
    no_ionisation: u64,
    spectra: Vec<SpectrumAcc>,
    radial: Vec<f64>,
    radial_under: f64,
    radial_over: f64,
    quirks: QuirkCounts,
}

impl Scorer {
    pub fn new(e_in_ev: f64) -> Self {
        Self {
            e_in: e_in_ev,
            n: 0,
            terminations: [0; 3],
            interactions: [0; ProcessKind::COUNT],
            per_history: [Welford::default(); ProcessKind::COUNT],
            total_track_length_sum_nm: 0.0,
            first_length: Welford::default(),
            first_process: [0; ProcessKind::COUNT],
            no_steps: 0,
            track: Welford::default(),
            total_track: Welford::default(),
            range: Welford::default(),
            projected: Welford::default(),
            tracks: Welford::default(),
            secondaries_n: Welford::default(),
            dep_exc: Welford::default(),
            dep_vib: Welford::default(),
            dep_bind: Welford::default(),
            dep_att: Welford::default(),
            dep_solv: Welford::default(),
            dep_total: Welford::default(),
            untracked_sec: Welford::default(),
            remaining: Welford::default(),
            max_residual: 0.0,
            levels: [0; 5],
            shells: [0; 5],
            vib_levels: [0; SANCHE_LEVELS],
            solvation_events: 0,
            solvation_per_history: Welford::default(),
            solvation_displacement_sum: 0.0,
            solvation_energy_sum: 0.0,
            solvation_per_100ev: PairedWelford::default(),
            solvation_per_100ev_each: Welford::default(),
            ionisations: Welford::default(),
            w_totals: PairedWelford::default(),
            w_each: Welford::default(),
            no_ionisation: 0,
            spectra: (0..=GENERATION_CLASSES)
                .map(|_| SpectrumAcc::new())
                .collect(),
            radial: vec![0.0; RADIAL_BINS],
            radial_under: 0.0,
            radial_over: 0.0,
            quirks: QuirkCounts::default(),
        }
    }

    pub fn add(&mut self, r: &HistoryResult) {
        let o = &r.outcome;
        self.n += 1;
        self.terminations[match o.termination {
            Termination::Completed => 0,
            Termination::InteractionLimit => 1,
            Termination::TrackingCut => 2,
        }] += 1;
        for p in ProcessKind::ALL {
            let c = o.counts.get(p);
            self.interactions[p.index()] += c;
            self.per_history[p.index()].add(c as f64);
        }
        self.total_track_length_sum_nm += o.total_track_length_nm;
        match o.first_step {
            Some(f) => {
                self.first_length.add(f.length_nm);
                self.first_process[f.process.index()] += 1;
            }
            None => self.no_steps += 1,
        }
        self.track.add(o.track_length_nm);
        self.total_track.add(o.total_track_length_nm);
        self.range.add(o.range_nm());
        self.projected.add(o.projected_range_nm());
        self.tracks.add(o.n_tracks as f64);
        self.secondaries_n.add(o.n_secondaries as f64);
        let l = &o.ledger;
        self.dep_exc.add(l.deposited_excitation);
        self.dep_vib.add(l.deposited_vibrational);
        self.dep_bind.add(l.deposited_binding);
        self.dep_att.add(l.deposited_attachment);
        self.dep_solv.add(l.deposited_solvation);
        self.dep_total.add(l.deposited());
        self.untracked_sec.add(l.untracked_secondaries);
        self.remaining.add(l.remaining_untracked);
        self.max_residual = self.max_residual.max(l.residual_relative().abs());
        for i in 0..5 {
            self.levels[i] += o.excitation_levels[i];
            self.shells[i] += o.ionisation_shells[i];
        }
        for (a, b) in self.vib_levels.iter_mut().zip(&o.vibrational_levels) {
            *a += b;
        }
        self.solvation_events += o.solvations;
        self.solvation_per_history.add(o.solvations as f64);
        self.solvation_displacement_sum += r.solvation.sum_displacement_nm;
        self.solvation_energy_sum += r.solvation.sum_energy_ev;
        let e_dep = l.deposited();
        self.solvation_per_100ev
            .add(100.0 * o.solvations as f64, e_dep);
        if e_dep > 0.0 {
            self.solvation_per_100ev_each
                .add(100.0 * o.solvations as f64 / e_dep);
        }
        let n_ion = o.counts.get(ProcessKind::Ionisation);
        self.ionisations.add(n_ion as f64);
        self.w_totals.add(l.e_in, n_ion as f64);
        if n_ion > 0 {
            self.w_each.add(l.e_in / n_ion as f64);
        } else {
            self.no_ionisation += 1;
        }
        for (class, sp) in r.spectra.iter().enumerate() {
            self.spectra[class].add(sp);
            self.spectra[GENERATION_CLASSES].add(sp);
        }
        for (a, b) in self.radial.iter_mut().zip(&r.radial.energy_ev) {
            *a += b;
        }
        self.radial_under += r.radial.underflow_ev;
        self.radial_over += r.radial.overflow_ev;
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
                    .then(|| self.total_track_length_sum_nm / self.interactions[p.index()] as f64),
            })
            .collect();
        let per_hist = |v: f64| v / n;
        RunSummary {
            n_histories: self.n,
            terminations: TerminationCounts {
                completed: self.terminations[0],
                interaction_limit: self.terminations[1],
                tracking_cut: self.terminations[2],
            },
            processes,
            solvation: SolvationSummary {
                events: self.solvation_events,
                per_history: self.solvation_per_history.stat(),
                mean_displacement_nm: (self.solvation_events > 0)
                    .then(|| self.solvation_displacement_sum / self.solvation_events as f64),
                mean_energy_ev: (self.solvation_events > 0)
                    .then(|| self.solvation_energy_sum / self.solvation_events as f64),
                per_100_ev: self.solvation_per_100ev.ratio(),
                per_100_ev_mean_of_ratios: self.solvation_per_100ev_each.stat(),
            },
            first_step: FirstStepSummary {
                mean_free_path_nm: self.first_length.stat(),
                first_process_counts: self.first_process,
                first_process_fractions: self.first_process.map(|c| c as f64 / n),
                histories_without_steps: self.no_steps,
            },
            tracks: TrackSummary {
                primary_track_length_nm: self.track.stat(),
                total_track_length_nm: self.total_track.stat(),
                range_nm: self.range.stat(),
                projected_range_nm: self.projected.stat(),
                tracks_per_history: self.tracks.stat(),
                secondaries_per_history: self.secondaries_n.stat(),
            },
            energy: EnergySummary {
                e_in_ev: self.e_in,
                deposited_excitation_ev: self.dep_exc.stat(),
                deposited_vibrational_ev: self.dep_vib.stat(),
                deposited_binding_ev: self.dep_bind.stat(),
                deposited_attachment_ev: self.dep_att.stat(),
                deposited_solvation_ev: self.dep_solv.stat(),
                deposited_total_ev: self.dep_total.stat(),
                untracked_secondaries_ev: self.untracked_sec.stat(),
                remaining_untracked_ev: self.remaining.stat(),
                max_abs_ledger_residual_relative: self.max_residual,
            },
            w_value: WSummary {
                ionisations_per_history: self.ionisations.stat(),
                w_ev_ratio_of_totals: self.w_totals.ratio(),
                w_ev_mean_of_ratios: self.w_each.stat(),
                histories_without_ionisation: self.no_ionisation,
            },
            excitation_levels: self.levels,
            ionisation_shells: self.shells,
            vibrational_levels: self.vib_levels,
            secondary_spectra: SecondarySpectraSummary {
                edges_ev: spectrum_edges_ev(),
                by_generation: self.spectra[..GENERATION_CLASSES]
                    .iter()
                    .map(SpectrumAcc::summary)
                    .collect(),
                all: self.spectra[GENERATION_CLASSES].summary(),
            },
            radial_deposit: RadialSummary {
                edges_nm: radial_edges_nm(),
                mean_energy_ev: self.radial.iter().map(|&v| per_hist(v)).collect(),
                mean_underflow_ev: per_hist(self.radial_under),
                mean_overflow_ev: per_hist(self.radial_over),
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

    #[test]
    fn ratio_of_totals_matches_hand_computation_and_delta_method() {
        // x = (2, 4, 6), y = (1, 2, 2): R = 12/5 = 2.4.
        // residuals x - R y = (-0.4, -0.8, 1.2), sample variance = (0.16+0.64+1.44)/2 = 1.12,
        // se = sqrt(1.12 / 3) / mean(y) = 0.61101 / (5/3).
        let mut p = PairedWelford::default();
        for (x, y) in [(2.0, 1.0), (4.0, 2.0), (6.0, 2.0)] {
            p.add(x, y);
        }
        let r = p.ratio().unwrap();
        assert_relative_eq!(r.ratio, 2.4, epsilon = 1e-14);
        assert_relative_eq!(
            r.std_err,
            (1.12f64 / 3.0).sqrt() / (5.0 / 3.0),
            epsilon = 1e-13
        );
    }

    #[test]
    fn ratio_with_zero_denominator_is_none() {
        let mut p = PairedWelford::default();
        p.add(1.0, 0.0);
        assert!(p.ratio().is_none());
    }
}
