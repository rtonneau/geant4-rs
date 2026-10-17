//! M6 residual audit harness (ticket 01 of session 2026-10-17__m6-residual-rust-audit).
//!
//! Diagnostic only: no Geant4 agreement gate, no production code touched, no new public API.
//! Everything below is fixed BEFORE any result of this file is evaluated; none of it may be edited
//! after a run (a change needs a documented failure and evidence in docs/08).
//!
//! # Relations (per event, at the 1 ps pre-IRT boundary, seven dissociation species)
//! * **R-Q, charge balance.** `Q = N(H3O+) - N(e_aq) - N(OH-)` must equal `-1` per event. Every
//!   ionisation gives a hole (+1) and an ejected electron that solvates (-1); every dissociative
//!   attachment turns one electron into `OH-`; only the primary adds an unpaired thermalised
//!   `e_aq`. Tested residual: `mean(Q) + 1 = 0`.
//! * **R-H, hydrogen vs oxygen budget.** Every decay channel of
//!   `G4ChemDissociationChannels_option1` consumes water in the ratio 2 H : 1 O (docs/04 D.2), so
//!   `B = N(H3O+) + N(H) + 2 N(H2) - N(OH) - N(OH-) - 2 N(O)` (that is `H atoms - 2 x O atoms`)
//!   must equal `0` per event. Tested residual: `mean(B) = 0`.
//! * **R-Ch, per channel (Rust channel table only).** For every channel of every decay state the
//!   product budget `H - 2 O` is `0` and the product charge equals the charge of the state kind
//!   (excitation 0, ionisation +1, attachment -1, recombined `H2Ovib` 0). The same two sums over the
//!   run-level `produced_by_event` of each event kind must equal their channel-table expectation:
//!   `B = 0` for excitation, ionisation and attachment; charge `0`, `n_ion - n_rec`, `-n_att`
//!   (and `-n_solv` for solvation).
//! * **R-W, water budget comparison (not a conservation law).** Mean O atoms and H atoms per event
//!   of the seven species, Geant4 vs Rust.
//!
//! # Tolerance and statistics (fixed)
//! * Every test uses a 4.5 sigma bound.
//! * **Geant4 side:** the sigma of a relation `sum_i c_i m_i` is the *uncorrelated* sum-of-variances
//!   bound `sqrt(sum_i c_i^2 se_i^2)` with `se_i^2 = (sum2_i - sum_i^2/N) / (N - 1) / N` from
//!   `physchem_prechem.csv`. Species of one event are correlated and the CSV has no covariance, so
//!   this sigma can be wrong in either direction; for a relation that is an exact per-event identity
//!   it is much larger than the truth, which makes the test weaker. The table prints this
//!   limitation next to the integer residual `sum_i c_i sum_i` (an exact identity gives 0 there).
//!   Pass if `|residual| <= 4.5 sigma`.
//! * **Rust side:** `K` extra master seeds (below), all different from 20261017 (gate), 20261018
//!   (shuffle) and 20261020 (Geant4). The relation residual of a seed is an exact integer total
//!   divided by `N`; over seeds the spread is the sample standard deviation divided by `sqrt(K)`.
//!   Pass if `mean == 0` exactly or `|mean| <= 4.5 x spread`. A multi-seed species mean is compared
//!   with Geant4 by `z = (rust - g4) / hypot(g4 se, seed spread)`; `z` is printed together with
//!   `1/sqrt(N)` and `z/sqrt(N)` (N = Geant4 histories): noise gives `z` independent of N and
//!   `z/sqrt(N)` growing as `1/sqrt(N)`, a bias in per-event units gives a constant `z/sqrt(N)`.
//!   Species `z` and R-W are reported only, never asserted.
//! * **Runtime fallback:** 900 keV uses [`SEEDS_900_KEV`] seeds (2 if 5 is too slow); the multi-seed
//!   table states the seed count used at each energy.
//!
//! Entry point and configuration are those of `physchem_reference.rs` (`run_physchem_summary`,
//! `ProcessTable::option2_electron`, `PhysChemEngine::option3`).

use dna_chemistry::PhysChemEngine;
use dna_chemistry::channels::StateKind;
use dna_chemistry::species::{SpeciesTable, atoms};
use dna_core::species::Species;
use dna_data::datadir::{DataDir, repo_data_dir};
use dna_physics::models::ElasticRotation;
use dna_physics::table::ProcessTable;
use dna_sim::config::RunConfig;
use dna_sim::run::run_physchem_summary;
use std::path::PathBuf;
use std::sync::Mutex;

/// Serialises the tests so the tables of different tests are not interleaved line by line.
static OUTPUT: Mutex<()> = Mutex::new(());

/// Tolerance in sigma, fixed before any result.
const SIGMA: f64 = 4.5;
/// Extra Rust master seeds (not 20261017, 20261018 or 20261020).
const SEEDS: [u64; 5] = [20261101, 20261102, 20261103, 20261104, 20261105];
/// Number of the seeds in [`SEEDS`] used at 900 keV (5 unless the runtime is prohibitive).
const SEEDS_900_KEV: usize = 5;
/// The four reference energies (eV) and the Geant4 history counts of `physchem_prechem.csv`.
const CASES: [(f64, u64); 4] = [
    (1_000.0, 20_000),
    (10_000.0, 2_000),
    (100_000.0, 200),
    (900_000.0, 20),
];
/// The seven species of the pre-IRT snapshot.
const SEVEN: [Species; 7] = [
    Species::H3Op,
    Species::OH,
    Species::OHm,
    Species::e_aq,
    Species::H,
    Species::H2,
    Species::Oxy,
];

#[derive(Debug)]
struct G4Row {
    energy: f64,
    n: u64,
    species: Species,
    sum: f64,
    sum2: f64,
}

impl G4Row {
    fn mean(&self) -> f64 {
        self.sum / self.n as f64
    }

    fn se(&self) -> f64 {
        if self.n < 2 {
            return 0.0;
        }
        let n = self.n as f64;
        ((self.sum2 - self.sum * self.sum / n).max(0.0) / (n - 1.0) / n).sqrt()
    }
}

fn g4_rows() -> Vec<G4Row> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../validation/reference/physchem_prechem.csv");
    let text = std::fs::read_to_string(path).unwrap();
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.starts_with("energy_eV"))
        .map(|line| {
            let c: Vec<&str> = line.split(',').collect();
            G4Row {
                energy: c[0].parse().unwrap(),
                n: c[1].parse().unwrap(),
                species: Species::from_geant4_name(c[2])
                    .or_else(|| Species::from_chemistry_name(c[2]))
                    .unwrap_or_else(|| panic!("unknown Geant4 species {}", c[2])),
                sum: c[3].parse().unwrap(),
                sum2: c[4].parse().unwrap(),
            }
        })
        .collect()
}

/// A linear relation `sum_i coefficient_i x count_i + offset` that is `0` per event.
struct Relation {
    name: &'static str,
    coefficient: [i64; 15],
    offset: f64,
}

fn relations() -> [Relation; 2] {
    let table = SpeciesTable::option3().unwrap();
    let mut charge = [0i64; 15];
    let mut budget = [0i64; 15];
    for species in Species::ALL {
        charge[species.index()] = i64::from(table.get(species).charge);
        let (h, o) = atoms(species);
        budget[species.index()] = i64::from(h) - 2 * i64::from(o);
    }
    [
        Relation {
            name: "R-Q  charge + 1 (thermalised primary e_aq)",
            coefficient: charge,
            offset: 1.0,
        },
        Relation {
            name: "R-H  H atoms - 2 x O atoms",
            coefficient: budget,
            offset: 0.0,
        },
    ]
}

fn atom_weights() -> ([f64; 15], [f64; 15]) {
    let mut h = [0.0; 15];
    let mut o = [0.0; 15];
    for species in Species::ALL {
        let (hydrogen, oxygen) = atoms(species);
        h[species.index()] = f64::from(hydrogen);
        o[species.index()] = f64::from(oxygen);
    }
    (h, o)
}

fn mean_and_spread(values: &[f64]) -> (f64, f64) {
    let k = values.len() as f64;
    let mean = values.iter().sum::<f64>() / k;
    if values.len() < 2 {
        return (mean, 0.0);
    }
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (k - 1.0);
    (mean, (variance / k).sqrt())
}

fn process_table() -> ProcessTable {
    ProcessTable::option2_electron(
        &DataDir::new(repo_data_dir()).unwrap(),
        ElasticRotation::Geant4,
    )
    .unwrap()
}

fn kind_expected_charge(kind: StateKind) -> i64 {
    match kind {
        StateKind::Excitation | StateKind::Recombined => 0,
        StateKind::Ionisation => 1,
        StateKind::Attachment => -1,
    }
}

#[test]
#[ignore = "M6 residual audit: per-channel budgets of the Rust channel table"]
fn audit_channel_budgets() {
    let _serial = OUTPUT.lock().unwrap_or_else(|e| e.into_inner());
    let engine = PhysChemEngine::option3().unwrap();
    let table = SpeciesTable::option3().unwrap();
    let mut failures = Vec::<String>::new();
    println!("\nR-Ch per-channel budgets (Rust channel table, option3)");
    println!(
        "{:<12} {:<28} {:>9} {:>3} {:>3} {:>7} {:>8} {:>6}",
        "state", "channel", "prob", "H", "O", "charge", "H-2O", "ok"
    );
    for state in engine.channel_table().states() {
        let expected_charge = kind_expected_charge(state.kind);
        let probability: f64 = state.channels.iter().map(|c| c.probability).sum();
        if (probability - 1.0).abs() > 1e-12 {
            failures.push(format!("{}: probability sum {probability}", state.name));
        }
        for channel in &state.channels {
            let (mut h, mut o, mut q) = (0i64, 0i64, 0i64);
            for &product in &channel.products {
                let (hydrogen, oxygen) = atoms(product);
                h += i64::from(hydrogen);
                o += i64::from(oxygen);
                q += i64::from(table.get(product).charge);
            }
            let ok = h - 2 * o == 0 && q == expected_charge;
            if !ok {
                failures.push(format!(
                    "{} / {}: H={h} O={o} charge={q} (expected charge {expected_charge})",
                    state.name, channel.name
                ));
            }
            println!(
                "{:<12} {:<28} {:>9.6} {:>3} {:>3} {:>+7} {:>8} {:>6}",
                state.name,
                channel.name,
                channel.probability,
                h,
                o,
                q,
                h - 2 * o,
                if ok { "ok" } else { "FAIL" }
            );
        }
    }
    assert!(
        failures.is_empty(),
        "channel budget failures:\n{}",
        failures.join("\n")
    );
}

#[test]
#[ignore = "M6 residual audit: relations on validation/reference/physchem_prechem.csv"]
fn audit_relations_geant4_csv() {
    let _serial = OUTPUT.lock().unwrap_or_else(|e| e.into_inner());
    let rows = g4_rows();
    let (h_weight, o_weight) = atom_weights();
    let mut failures = Vec::<String>::new();
    println!(
        "\nGeant4 relations (physchem_prechem.csv, seed 20261020). LIMITATION: sigma is the \
         UNCORRELATED sum-of-variances bound; species of one event are correlated and the CSV has \
         no covariance, so sigma is only a bound on the order of the noise and an exact identity \
         is tested more weakly than the integer residual shows."
    );
    println!(
        "{:>8} {:>7} {:<44} {:>14} {:>13} {:>12} {:>9} {:>5}",
        "E (keV)",
        "N",
        "relation",
        "int. residual",
        "mean resid.",
        "sigma(bound)",
        "|r|/sigma",
        "ok"
    );
    for (energy, n) in CASES {
        let case: Vec<&G4Row> = rows.iter().filter(|r| r.energy == energy).collect();
        assert_eq!(case.len(), 7, "{energy}: expected seven species");
        assert!(case.iter().all(|r| r.n == n));
        for relation in relations() {
            let mut integer = 0.0;
            let mut variance = 0.0;
            for row in &case {
                let c = relation.coefficient[row.species.index()] as f64;
                integer += c * row.sum;
                variance += c * c * row.se() * row.se();
            }
            let mean = integer / n as f64 + relation.offset;
            let sigma = variance.sqrt();
            let ok = mean == 0.0 || mean.abs() <= SIGMA * sigma;
            if !ok {
                failures.push(format!(
                    "{energy} {}: mean {mean} sigma {sigma}",
                    relation.name
                ));
            }
            println!(
                "{:>8.0} {:>7} {:<44} {:>14.1} {:>13.3e} {:>12.3e} {:>9.3} {:>5}",
                energy / 1000.0,
                n,
                relation.name,
                integer + relation.offset * n as f64,
                mean,
                sigma,
                if sigma > 0.0 { mean.abs() / sigma } else { 0.0 },
                if ok { "ok" } else { "FAIL" }
            );
        }
        // R-W: water budget, reported per event.
        let (mut o_mean, mut h_mean, mut o_var, mut h_var) = (0.0, 0.0, 0.0, 0.0);
        for row in &case {
            let (h, o) = (h_weight[row.species.index()], o_weight[row.species.index()]);
            h_mean += h * row.mean();
            o_mean += o * row.mean();
            h_var += h * h * row.se() * row.se();
            o_var += o * o * row.se() * row.se();
        }
        println!(
            "{:>8.0} {:>7} {:<44} O atoms/event {o_mean:.4} +/- {:.4} (bound); H atoms/event {h_mean:.4} +/- {:.4} (bound)",
            energy / 1000.0,
            n,
            "R-W  water budget (reported only)",
            o_var.sqrt(),
            h_var.sqrt()
        );
    }
    assert!(
        failures.is_empty(),
        "Geant4 relation failures:\n{}",
        failures.join("\n")
    );
}

/// Per-seed Rust results at one energy.
struct SeedRun {
    n: u64,
    totals: [u64; 15],
    means: [f64; 15],
    events: [u64; 4],
    recombinations: u64,
    /// `produced_by_event[kind][species]`.
    produced: [[u64; 15]; 4],
}

fn run_seed(
    table: &ProcessTable,
    engine: &PhysChemEngine,
    energy: f64,
    n: u64,
    seed: u64,
) -> SeedRun {
    let mut config = RunConfig::new(energy, n);
    config.master_seed = seed;
    let summary = run_physchem_summary(table, engine, &config).unwrap().1;
    assert_eq!(summary.n_histories, n);
    let mut run = SeedRun {
        n,
        totals: [0; 15],
        means: [0.0; 15],
        events: summary.events,
        recombinations: summary.recombinations,
        produced: [[0; 15]; 4],
    };
    for species in Species::ALL {
        let s = &summary.species[species.index()];
        run.totals[species.index()] = s.alive_total;
        run.means[species.index()] = s.alive_per_history.value;
        for kind in 0..4 {
            run.produced[kind][species.index()] = s.produced_by_event[kind];
        }
    }
    run
}

#[test]
#[ignore = "M6 residual audit: 5 extra Rust master seeds at 1, 10, 100 and 900 keV (long)"]
fn audit_multiseed_rust() {
    let _serial = OUTPUT.lock().unwrap_or_else(|e| e.into_inner());
    let table = process_table();
    let engine = PhysChemEngine::option3().unwrap();
    let rows = g4_rows();
    let (h_weight, o_weight) = atom_weights();
    let mut failures = Vec::<String>::new();
    // z[species][energy], for the closing summary.
    let mut z_summary = vec![[f64::NAN; 4]; 7];

    for (case_index, (energy, n)) in CASES.into_iter().enumerate() {
        let seed_count = if energy == 900_000.0 {
            SEEDS_900_KEV
        } else {
            SEEDS.len()
        };
        let started = std::time::Instant::now();
        let runs: Vec<SeedRun> = SEEDS[..seed_count]
            .iter()
            .map(|&seed| run_seed(&table, &engine, energy, n, seed))
            .collect();
        println!(
            "\n=== {:.0} keV, {n} histories per seed, {seed_count} seeds {:?}{} ({:.1} s) ===",
            energy / 1000.0,
            &SEEDS[..seed_count],
            if seed_count < SEEDS.len() {
                " (REDUCED seed count: 900 keV runtime fallback)"
            } else {
                ""
            },
            started.elapsed().as_secs_f64()
        );

        // Reaction-only species cannot exist before the first chemical reaction.
        for run in &runs {
            for species in Species::ALL {
                if !SEVEN.contains(&species) {
                    assert_eq!(
                        run.totals[species.index()],
                        0,
                        "{}",
                        species.chemistry_name()
                    );
                }
            }
        }

        println!(
            "Rust relations over seeds (exact integer residual / N; pass if mean == 0 or |mean| <= {SIGMA} x spread)"
        );
        println!(
            "{:<44} {:>13} {:>12} {:>5}   per-seed integer residuals",
            "relation", "mean", "spread", "ok"
        );
        for relation in relations() {
            let integers: Vec<i64> = runs
                .iter()
                .map(|run| {
                    Species::ALL
                        .iter()
                        .map(|s| relation.coefficient[s.index()] * run.totals[s.index()] as i64)
                        .sum::<i64>()
                        + relation.offset as i64 * run.n as i64
                })
                .collect();
            let per_event: Vec<f64> = integers.iter().map(|&i| i as f64 / n as f64).collect();
            let (mean, spread) = mean_and_spread(&per_event);
            let ok = mean == 0.0 || mean.abs() <= SIGMA * spread;
            if !ok {
                failures.push(format!(
                    "{energy} Rust {}: mean {mean} spread {spread}",
                    relation.name
                ));
            }
            println!(
                "{:<44} {:>13.3e} {:>12.3e} {:>5}   {:?}",
                relation.name,
                mean,
                spread,
                if ok { "ok" } else { "FAIL" },
                integers
            );
        }

        // R-Ch at run level: produced_by_event budgets per event kind.
        println!("R-Ch run-level budgets per event kind (summed over seeds; exc, ion, att, solv)");
        let table_charge = SpeciesTable::option3().unwrap();
        for (kind, label) in ["excitation", "ionisation", "attachment", "solvation"]
            .into_iter()
            .enumerate()
        {
            let (mut budget, mut charge, mut events, mut recombinations) = (0i64, 0i64, 0i64, 0i64);
            for run in &runs {
                events += run.events[kind] as i64;
                recombinations += run.recombinations as i64;
                for species in Species::ALL {
                    let (h, o) = atoms(species);
                    let produced = run.produced[kind][species.index()] as i64;
                    budget += (i64::from(h) - 2 * i64::from(o)) * produced;
                    charge += i64::from(table_charge.get(species).charge) * produced;
                }
            }
            let expected_charge = match kind {
                0 => 0,
                1 => events - recombinations,
                2 => -events,
                _ => -events,
            };
            let ok = budget == 0 && charge == expected_charge;
            if !ok {
                failures.push(format!(
                    "{energy} Rust run-level {label}: H-2O {budget}, charge {charge} (expected {expected_charge})"
                ));
            }
            println!(
                "  {label:<11} events {events:>9}  H-2O {budget:>3}  charge {charge:>+9} (expected {expected_charge:>+9})  {}",
                if ok { "ok" } else { "FAIL" }
            );
        }

        // R-W water budget.
        let o_per_seed: Vec<f64> = runs
            .iter()
            .map(|r| {
                Species::ALL
                    .iter()
                    .map(|s| o_weight[s.index()] * r.means[s.index()])
                    .sum()
            })
            .collect();
        let h_per_seed: Vec<f64> = runs
            .iter()
            .map(|r| {
                Species::ALL
                    .iter()
                    .map(|s| h_weight[s.index()] * r.means[s.index()])
                    .sum()
            })
            .collect();
        let case_rows: Vec<&G4Row> = rows.iter().filter(|r| r.energy == energy).collect();
        let g4_o: f64 = case_rows
            .iter()
            .map(|r| o_weight[r.species.index()] * r.mean())
            .sum();
        let g4_o_sigma = case_rows
            .iter()
            .map(|r| (o_weight[r.species.index()] * r.se()).powi(2))
            .sum::<f64>()
            .sqrt();
        let g4_h: f64 = case_rows
            .iter()
            .map(|r| h_weight[r.species.index()] * r.mean())
            .sum();
        let g4_h_sigma = case_rows
            .iter()
            .map(|r| (h_weight[r.species.index()] * r.se()).powi(2))
            .sum::<f64>()
            .sqrt();
        let (rust_o, rust_o_se) = mean_and_spread(&o_per_seed);
        let (rust_h, rust_h_se) = mean_and_spread(&h_per_seed);
        println!(
            "R-W (reported): O atoms/event rust {rust_o:.4} +/- {rust_o_se:.4}, G4 {g4_o:.4} +/- {g4_o_sigma:.4} (bound), z {:+.2}; H atoms/event rust {rust_h:.4} +/- {rust_h_se:.4}, G4 {g4_h:.4} +/- {g4_h_sigma:.4} (bound), z {:+.2}",
            (rust_o - g4_o) / rust_o_se.hypot(g4_o_sigma),
            (rust_h - g4_h) / rust_h_se.hypot(g4_h_sigma)
        );

        // Per-species multi-seed mean vs Geant4.
        println!(
            "Per-species mean +/- seed spread vs Geant4 (N = {n}, 1/sqrt(N) = {:.4e})",
            1.0 / (n as f64).sqrt()
        );
        println!(
            "{:<8} {:>12} {:>12} {:>12} {:>12} {:>9} {:>9} {:>11}",
            "species", "G4 mean", "G4 se", "Rust mean", "seed spread", "rel (%)", "z", "z/sqrt(N)"
        );
        for (si, species) in SEVEN.into_iter().enumerate() {
            let g4 = case_rows.iter().find(|r| r.species == species).unwrap();
            let values: Vec<f64> = runs.iter().map(|r| r.means[species.index()]).collect();
            let (mean, spread) = mean_and_spread(&values);
            let z = (mean - g4.mean()) / spread.hypot(g4.se());
            z_summary[si][case_index] = z;
            println!(
                "{:<8} {:>12.5} {:>12.5} {:>12.5} {:>12.5} {:>+9.3} {:>+9.2} {:>+11.4}",
                species.chemistry_name(),
                g4.mean(),
                g4.se(),
                mean,
                spread,
                100.0 * (mean - g4.mean()) / g4.mean(),
                z,
                z / (n as f64).sqrt()
            );
        }
    }

    println!(
        "\nz versus 1/sqrt(N) (N = 20000, 2000, 200, 20; 1/sqrt(N) = 0.0071, 0.0224, 0.0707, 0.2236)"
    );
    println!(
        "{:<8} {:>9} {:>9} {:>9} {:>9}   | z/sqrt(N): {:>9} {:>9} {:>9} {:>9}",
        "species", "1 keV", "10 keV", "100 keV", "900 keV", "1 keV", "10 keV", "100 keV", "900 keV"
    );
    for (si, species) in SEVEN.into_iter().enumerate() {
        let z = z_summary[si];
        println!(
            "{:<8} {:>+9.2} {:>+9.2} {:>+9.2} {:>+9.2}   | {:>20.4} {:>9.4} {:>9.4} {:>9.4}",
            species.chemistry_name(),
            z[0],
            z[1],
            z[2],
            z[3],
            z[0] / 20_000f64.sqrt(),
            z[1] / 2_000f64.sqrt(),
            z[2] / 200f64.sqrt(),
            z[3] / 20f64.sqrt()
        );
    }
    println!(
        "Species z and R-W are reported only; asserted: Rust relations R-Q, R-H and run-level R-Ch."
    );
    assert!(
        failures.is_empty(),
        "Rust relation failures:\n{}",
        failures.join("\n")
    );
}
