//! M6 Geant4 11.4.1 reference gate (ignored until `validation/reference/physchem.csv` exists).
//!
//! Acceptance criteria fixed before seeing reference values. Family 1 is every species-count mean
//! present in the CSV (at most 15 species x 4 energies): combined-standard-error z <= 4.5
//! (Bonferroni-normal threshold more stringent than family alpha 1e-3 for 60 tests) and relative
//! difference <= 15% when both means exceed 0.1/event. Family 2 is the corresponding RMS radius:
//! relative difference <= 20%; no z test is possible because the requested CSV contains sum(r^2)
//! but not sum[(per-event mean r^2)^2]. No tolerance may be changed after inspecting the CSV
//! without documenting the failure and evidence in docs/08.

use dna_chemistry::PhysChemEngine;
use dna_core::species::Species;
use dna_data::datadir::{DataDir, repo_data_dir};
use dna_physics::models::ElasticRotation;
use dna_physics::table::ProcessTable;
use dna_sim::config::RunConfig;
use dna_sim::run::run_physchem_summary;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug)]
struct Row {
    energy: f64,
    n: u64,
    species: Species,
    sum: f64,
    sum2: f64,
    sum_r2: f64,
}

fn reference_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../validation/reference")
}

/// The generator writes `G4Molecule::GetName()` (`<definition>^<charge>`, e.g. `H3O^1`, `°OH^0`);
/// exact Geant4 configuration keys are accepted too.
fn species(name: &str) -> Option<Species> {
    Species::from_geant4_name(name).or_else(|| Species::from_chemistry_name(name))
}

fn rows() -> Vec<Row> {
    let text = std::fs::read_to_string(reference_dir().join("physchem.csv")).unwrap();
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.starts_with("energy_eV"))
        .map(|line| {
            let c: Vec<&str> = line.split(',').collect();
            Row {
                energy: c[0].parse().unwrap(),
                n: c[1].parse().unwrap(),
                species: species(c[2]).unwrap_or_else(|| panic!("unknown Geant4 species {}", c[2])),
                sum: c[3].parse().unwrap(),
                sum2: c[4].parse().unwrap(),
                sum_r2: c[5].parse().unwrap(),
            }
        })
        .collect()
}

#[test]
#[ignore = "requires validation/reference/physchem.csv and provenance_m6.toml from Geant4 11.4.1"]
fn m6_initial_species_counts_and_rms_match_geant4() {
    assert!(reference_dir().join("provenance_m6.toml").is_file());
    let rows = rows();
    let mut cases = BTreeMap::<u64, (f64, u64)>::new();
    for r in &rows {
        cases.insert(r.energy.to_bits(), (r.energy, r.n));
    }
    let root = repo_data_dir();
    let table =
        ProcessTable::option2_electron(&DataDir::new(&root).unwrap(), ElasticRotation::Geant4)
            .unwrap();
    let engine = PhysChemEngine::option3().unwrap();
    for (_, (energy, n)) in cases {
        let mut config = RunConfig::new(energy, n);
        config.master_seed = 20261017;
        let rust = run_physchem_summary(&table, &engine, &config).unwrap().1;
        for g4 in rows.iter().filter(|r| r.energy == energy) {
            let ours = &rust.species[g4.species.index()];
            let g4_mean = g4.sum / g4.n as f64;
            let g4_se = if g4.n > 1 {
                ((g4.sum2 - g4.sum * g4.sum / g4.n as f64).max(0.0)
                    / (g4.n - 1) as f64
                    / g4.n as f64)
                    .sqrt()
            } else {
                0.0
            };
            let se = (g4_se * g4_se + ours.alive_per_history.std_err.powi(2)).sqrt();
            let delta = (ours.alive_per_history.value - g4_mean).abs();
            assert!(
                se == 0.0 || delta / se <= 4.5,
                "{energy} {}: z={}",
                g4.species.chemistry_name(),
                delta / se
            );
            if g4_mean > 0.1 && ours.alive_per_history.value > 0.1 {
                assert!(
                    delta / g4_mean <= 0.15,
                    "{energy} {} count",
                    g4.species.chemistry_name()
                );
            }
            if g4.sum > 0.0 {
                let g4_rms = (g4.sum_r2 / g4.sum).sqrt();
                let rust_rms = ours.rms_radius_nm.unwrap();
                assert!(
                    (rust_rms - g4_rms).abs() / g4_rms <= 0.20,
                    "{energy} {} RMS",
                    g4.species.chemistry_name()
                );
            }
        }
    }
}

fn report_metric(value: Option<f64>, signed: bool) -> String {
    match value {
        Some(value) if signed => format!("{value:+.3}"),
        Some(value) => format!("{value:.3}"),
        None => "N/A".to_owned(),
    }
}

#[test]
#[ignore = "requires validation/reference/physchem.csv and provenance_m6.toml from Geant4 11.4.1"]
fn m6_initial_species_comparison_report() {
    let reference = rows();
    let root = repo_data_dir();
    let table =
        ProcessTable::option2_electron(&DataDir::new(&root).unwrap(), ElasticRotation::Geant4)
            .unwrap();
    let engine = PhysChemEngine::option3().unwrap();

    for energy in [1_000.0, 10_000.0, 100_000.0, 900_000.0] {
        let energy_rows: Vec<&Row> = reference
            .iter()
            .filter(|row| row.energy == energy)
            .collect();
        let n = energy_rows
            .first()
            .unwrap_or_else(|| panic!("no Geant4 rows for {energy} eV"))
            .n;
        let mut config = RunConfig::new(energy, n);
        config.master_seed = 20261017;
        let rust = run_physchem_summary(&table, &engine, &config).unwrap().1;

        println!("\n{:.0} keV ({} histories)", energy / 1_000.0, n);
        println!(
            "{:<8} {:>27} {:>27} {:>9} {:>12} {:>12} {:>12}",
            "Species",
            "Geant4 mean +/- SE",
            "Rust mean +/- SE",
            "z",
            "Rel diff (%)",
            "G4 RMS (nm)",
            "Rust RMS (nm)"
        );
        for species in Species::ALL {
            let g4 = energy_rows
                .iter()
                .find(|row| row.species == species)
                .copied();
            let (g4_mean, g4_se, g4_rms) = if let Some(row) = g4 {
                let mean = row.sum / row.n as f64;
                let se = if row.n > 1 {
                    ((row.sum2 - row.sum * row.sum / row.n as f64).max(0.0)
                        / (row.n - 1) as f64
                        / row.n as f64)
                        .sqrt()
                } else {
                    0.0
                };
                let rms = (row.sum > 0.0).then(|| (row.sum_r2 / row.sum).sqrt());
                (mean, se, rms)
            } else {
                (0.0, 0.0, None)
            };
            let ours = &rust.species[species.index()];
            let rust_mean = ours.alive_per_history.value;
            let rust_se = ours.alive_per_history.std_err;
            let combined_se = g4_se.hypot(rust_se);
            let z = (combined_se > 0.0).then(|| (rust_mean - g4_mean) / combined_se);
            let relative_percent =
                (g4_mean != 0.0).then(|| 100.0 * (rust_mean - g4_mean) / g4_mean);
            println!(
                "{:<8} {:>11.4e} +/- {:>11.4e} {:>11.4e} +/- {:>11.4e} {:>9} {:>12} {:>12} {:>12}",
                species.chemistry_name(),
                g4_mean,
                g4_se,
                rust_mean,
                rust_se,
                report_metric(z, true),
                report_metric(relative_percent, true),
                report_metric(g4_rms, false),
                report_metric(ours.rms_radius_nm, false),
            );
        }
    }
}
