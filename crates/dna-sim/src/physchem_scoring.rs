//! Initial-species scoring at the 1 ps boundary between the physicochemical and chemical stages.
//!
//! `alive` counts are the species handed to M7 after recombination. `produced_by_event` attributes
//! products to excitation, ionisation, attachment and solvation before a later recombination can
//! consume an electron. Displacements are from the parent physical event. Pair distances are for
//! products of the same decay, avoiding an ill-defined and quadratic all-pairs observable.
//! Radius instead uses the surviving species positions relative to the primary origin, which
//! `RunConfig` fixes at (0, 0, 0). This reducer consumes the completed 1 ps stage; it does not
//! advance species in time or select a snapshot from a time series.

use dna_chemistry::HistoryChemistry;
use dna_core::species::Species;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default)]
struct Moments {
    n: u64,
    mean: f64,
    m2: f64,
}

impl Moments {
    fn add(&mut self, value: f64) {
        self.n += 1;
        let delta = value - self.mean;
        self.mean += delta / self.n as f64;
        self.m2 += delta * (value - self.mean);
    }

    fn summary(self) -> Estimate {
        Estimate {
            n: self.n,
            value: self.mean,
            std_err: if self.n > 1 {
                (self.m2 / (self.n - 1) as f64 / self.n as f64).sqrt()
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct RatioMoments {
    n: u64,
    mx: f64,
    my: f64,
    cxx: f64,
    cxy: f64,
    cyy: f64,
}

impl RatioMoments {
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

    fn summary(self) -> Option<Estimate> {
        if self.n == 0 || self.my == 0.0 {
            return None;
        }
        let ratio = self.mx / self.my;
        let std_err = if self.n > 1 {
            let variance = (self.cxx - 2.0 * ratio * self.cxy + ratio * ratio * self.cyy)
                / (self.n - 1) as f64;
            (variance.max(0.0) / self.n as f64).sqrt() / self.my.abs()
        } else {
            0.0
        };
        Some(Estimate {
            n: self.n,
            value: ratio,
            std_err,
        })
    }
}

/// An estimate and its standard error.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Estimate {
    pub n: u64,
    pub value: f64,
    pub std_err: f64,
}

/// Counts and initial yields of one species.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpeciesYield {
    pub species: &'static str,
    pub alive_total: u64,
    pub alive_per_history: Estimate,
    /// Excitation, ionisation, attachment, solvation; counts are before electron consumption.
    pub produced_by_event: [u64; 4],
    pub g_ratio_of_totals: Option<Estimate>,
    pub g_mean_of_event_ratios: Estimate,
    pub g_by_event_ratio_of_totals: [Option<Estimate>; 4],
    pub g_by_event_mean_of_ratios: [Estimate; 4],
    /// RMS distance from the primary origin (alive species), nm.
    pub rms_radius_nm: Option<f64>,
    /// RMS displacement from the parent event, nm (solvation excluded).
    pub rms_product_displacement_nm: Option<f64>,
}

/// A same-decay product-pair distance observable.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PairDistance {
    pub pair: &'static str,
    pub count: u64,
    pub mean_nm: Option<f64>,
    pub rms_nm: Option<f64>,
}

/// Initial species scores at global time 1 ps.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PhysChemSummary {
    pub time_ps: f64,
    pub n_histories: u64,
    pub events: [u64; 4],
    pub holes_with_candidate: u64,
    pub recombinations: u64,
    pub decay_energy_ev: f64,
    pub species: Vec<SpeciesYield>,
    pub pair_distances: Vec<PairDistance>,
}

/// Fixed-order reducer for physicochemical histories.
#[derive(Debug, Clone)]
pub struct PhysChemScorer {
    n: u64,
    events: [u64; 4],
    holes_with_candidate: u64,
    recombinations: u64,
    decay_energy_ev: f64,
    alive_total: [u64; 15],
    alive: [Moments; 15],
    produced: [[u64; 15]; 4],
    g_totals: [RatioMoments; 15],
    g_each: [Moments; 15],
    event_g_totals: [[RatioMoments; 15]; 4],
    event_g_each: [[Moments; 15]; 4],
    radius_sum_sq: [f64; 15],
    displacement_count: [u64; 15],
    displacement_sum_sq: [f64; 15],
    pair_count: [u64; 2],
    pair_sum: [f64; 2],
    pair_sum_sq: [f64; 2],
}

impl Default for PhysChemScorer {
    fn default() -> Self {
        Self {
            n: 0,
            events: [0; 4],
            holes_with_candidate: 0,
            recombinations: 0,
            decay_energy_ev: 0.0,
            alive_total: [0; 15],
            alive: [Moments::default(); 15],
            produced: [[0; 15]; 4],
            g_totals: [RatioMoments::default(); 15],
            g_each: [Moments::default(); 15],
            event_g_totals: [[RatioMoments::default(); 15]; 4],
            event_g_each: [[Moments::default(); 15]; 4],
            radius_sum_sq: [0.0; 15],
            displacement_count: [0; 15],
            displacement_sum_sq: [0.0; 15],
            pair_count: [0; 2],
            pair_sum: [0.0; 2],
            pair_sum_sq: [0.0; 2],
        }
    }
}

impl PhysChemScorer {
    /// Add a completed physicochemical history at global time 1 ps, with the primary at zero.
    /// Alive counts and radius use the post-recombination population; event production and
    /// product displacement retain their distinct pre-consumption definitions.
    pub fn add(&mut self, chemistry: &HistoryChemistry, deposited_energy_ev: f64) {
        self.n += 1;
        let stats = &chemistry.stats;
        for i in 0..4 {
            self.events[i] += stats.events[i];
        }
        self.holes_with_candidate += stats.holes_with_candidate;
        self.recombinations += stats.recombinations;
        self.decay_energy_ev += stats.decay_energy_ev;
        for species in Species::ALL {
            let i = species.index();
            let alive = stats.species_counts[i];
            self.alive_total[i] += alive;
            self.alive[i].add(alive as f64);
            self.g_totals[i].add(100.0 * alive as f64, deposited_energy_ev);
            if deposited_energy_ev > 0.0 {
                self.g_each[i].add(100.0 * alive as f64 / deposited_energy_ev);
            }
            for event in 0..4 {
                let produced = stats.species_produced_by_event[event][i];
                self.produced[event][i] += produced;
                self.event_g_totals[event][i].add(100.0 * produced as f64, deposited_energy_ev);
                if deposited_energy_ev > 0.0 {
                    self.event_g_each[event][i].add(100.0 * produced as f64 / deposited_energy_ev);
                }
            }
            self.displacement_count[i] += stats.displacement_count[i];
            self.displacement_sum_sq[i] += stats.displacement_sum_sq_nm2[i];
        }
        for instance in &chemistry.species {
            let r = instance.position.norm();
            self.radius_sum_sq[instance.species.index()] += r * r;
        }
        for i in 0..2 {
            self.pair_count[i] += stats.pair_count[i];
            self.pair_sum[i] += stats.pair_sum_nm[i];
            self.pair_sum_sq[i] += stats.pair_sum_sq_nm2[i];
        }
    }

    pub fn finish(self) -> PhysChemSummary {
        let species = Species::ALL
            .into_iter()
            .map(|s| {
                let i = s.index();
                SpeciesYield {
                    species: s.chemistry_name(),
                    alive_total: self.alive_total[i],
                    alive_per_history: self.alive[i].summary(),
                    produced_by_event: self.produced.map(|row| row[i]),
                    g_ratio_of_totals: self.g_totals[i].summary(),
                    g_mean_of_event_ratios: self.g_each[i].summary(),
                    g_by_event_ratio_of_totals: self.event_g_totals.map(|row| row[i].summary()),
                    g_by_event_mean_of_ratios: self.event_g_each.map(|row| row[i].summary()),
                    rms_radius_nm: (self.alive_total[i] > 0)
                        .then(|| (self.radius_sum_sq[i] / self.alive_total[i] as f64).sqrt()),
                    rms_product_displacement_nm: (self.displacement_count[i] > 0).then(|| {
                        (self.displacement_sum_sq[i] / self.displacement_count[i] as f64).sqrt()
                    }),
                }
            })
            .collect();
        let pair_distances = ["°OH-e_aq", "°OH-H3Op"]
            .into_iter()
            .enumerate()
            .map(|(i, pair)| PairDistance {
                pair,
                count: self.pair_count[i],
                mean_nm: (self.pair_count[i] > 0)
                    .then(|| self.pair_sum[i] / self.pair_count[i] as f64),
                rms_nm: (self.pair_count[i] > 0)
                    .then(|| (self.pair_sum_sq[i] / self.pair_count[i] as f64).sqrt()),
            })
            .collect();
        PhysChemSummary {
            time_ps: 1.0,
            n_histories: self.n,
            events: self.events,
            holes_with_candidate: self.holes_with_candidate,
            recombinations: self.recombinations,
            decay_energy_ev: self.decay_energy_ev,
            species,
            pair_distances,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dna_chemistry::PhysChemStats;
    use dna_core::species::SpeciesInstance;
    use dna_core::units::Time;
    use dna_core::vec3::Vec3;

    fn history(n_oh: u64, produced_exc: u64, radii: &[f64]) -> HistoryChemistry {
        let mut counts = [0; 15];
        counts[Species::OH.index()] = n_oh;
        let mut produced = [[0; 15]; 4];
        produced[0][Species::OH.index()] = produced_exc;
        HistoryChemistry {
            species: radii
                .iter()
                .map(|&r| SpeciesInstance {
                    species: Species::OH,
                    position: Vec3::new(r, 0.0, 0.0),
                    time: Time::from_ps(1.0),
                })
                .collect(),
            stats: PhysChemStats {
                events: [1, 0, 0, 0],
                channel_counts: Vec::new(),
                holes_with_candidate: 0,
                recombinations: 0,
                decay_energy_ev: 0.0,
                species_counts: counts,
                species_produced_by_event: produced,
                displacement_count: [0; 15],
                displacement_sum_sq_nm2: [0.0; 15],
                pair_count: [0; 2],
                pair_sum_nm: [0.0; 2],
                pair_sum_sq_nm2: [0.0; 2],
            },
        }
    }

    #[test]
    fn two_g_estimators_and_rms_match_hand_calculation() {
        // Independent hand table: (N_OH, E_dep) = (1, 100), (3, 200).
        // ratio of totals = 100 * 4/300 = 4/3; mean of ratios = (1 + 1.5)/2 = 1.25.
        // radii 1, 3, 4, 0 nm -> RMS = sqrt(26/4).
        let mut scorer = PhysChemScorer::default();
        scorer.add(&history(1, 1, &[1.0]), 100.0);
        scorer.add(&history(3, 3, &[3.0, 4.0, 0.0]), 200.0);
        let summary = scorer.finish();
        for (score, species) in summary.species.iter().zip(Species::ALL) {
            assert_eq!(
                score.species.as_bytes(),
                species.chemistry_name().as_bytes()
            );
        }
        assert_eq!(summary.pair_distances[0].pair, "°OH-e_aq");
        assert_eq!(summary.pair_distances[1].pair, "°OH-H3Op");
        let oh = &summary.species[Species::OH.index()];
        assert!((oh.g_ratio_of_totals.unwrap().value - 4.0 / 3.0).abs() < 1e-14);
        assert!((oh.g_mean_of_event_ratios.value - 1.25).abs() < 1e-14);
        assert!((oh.g_by_event_ratio_of_totals[0].unwrap().value - 4.0 / 3.0).abs() < 1e-14);
        assert!((oh.rms_radius_nm.unwrap() - (26.0f64 / 4.0).sqrt()).abs() < 1e-14);
    }
}
