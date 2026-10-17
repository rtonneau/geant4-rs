//! M6 Geant4 11.4.1 reference gate (ignored until `validation/reference/physchem.csv` exists).
//!
//! Acceptance criteria fixed before seeing reference values. Family 1 is every species-count mean
//! present in the CSV (at most 15 species x 4 energies): combined-standard-error z <= 4.5
//! (Bonferroni-normal threshold more stringent than family alpha 1e-3 for 60 tests) and relative
//! difference <= 15% when both means exceed 0.1/event. Family 2 is the corresponding RMS radius:
//! relative difference <= 20%; no z test is possible because the requested CSV contains sum(r^2)
//! but not sum[(per-event mean r^2)^2]. No tolerance may be changed after inspecting the CSV
//! without documenting the failure and evidence in docs/08.
//!
//! The committed reference includes zero-time option3 reactions at 1 ps, as established by the
//! semantic diagnostic below and the scheduler audit in docs/08. Its historical "before reactions"
//! label does not describe a dissociation-only snapshot. The original gate remains unchanged.

use dna_chemistry::PhysChemEngine;
use dna_core::species::Species;
use dna_data::datadir::{DataDir, repo_data_dir};
use dna_physics::models::ElasticRotation;
use dna_physics::table::ProcessTable;
use dna_sim::config::RunConfig;
use dna_sim::run::run_physchem_summary;
use std::collections::{BTreeMap, BTreeSet};
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
    parse_rows(&text)
}

fn parse_rows(text: &str) -> Vec<Row> {
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
#[ignore = "requires validation/reference/physchem.csv from Geant4 11.4.1"]
fn m6_reference_contains_products_requiring_zero_time_chemistry() {
    // Independently enumerated from Geant4 11.4.1, source/physics_lists/constructors/
    // electromagnetic/src/G4ChemDissociationChannels_option1.cc:194-465. Include H2Ovib:
    // electron-hole recombination changes the decay state, not the set of possible products.
    let engine = PhysChemEngine::option3().unwrap();
    let dissociation_products: BTreeSet<_> = engine
        .channel_table()
        .states()
        .iter()
        .flat_map(|state| &state.channels)
        .flat_map(|channel| &channel.products)
        .copied()
        .collect();
    assert_eq!(
        dissociation_products,
        BTreeSet::from([
            Species::H3Op,
            Species::OH,
            Species::OHm,
            Species::e_aq,
            Species::H,
            Species::H2,
            Species::Oxy,
        ])
    );

    let reference = rows();
    let reaction_only: BTreeSet<_> = reference
        .iter()
        .filter(|row| row.sum > 0.0 && !dissociation_products.contains(&row.species))
        .map(|row| row.species)
        .collect();
    assert_eq!(
        reaction_only,
        BTreeSet::from([
            Species::H2O2,
            Species::HO2,
            Species::HO2m,
            Species::Om,
            Species::O2,
            Species::O2m,
            Species::O3,
        ]),
        "the committed reference is not a dissociation-only species list"
    );
    // These four are present at every reference energy; the other reaction products are rare.
    // Their reaction paths are in G4EmDNAChemistry_option3.cc:170-187,255-287.
    for energy in [1_000.0, 10_000.0, 100_000.0, 900_000.0] {
        for species in [Species::H2O2, Species::HO2, Species::Om, Species::O2] {
            assert!(
                reference
                    .iter()
                    .any(|row| row.energy == energy && row.species == species && row.sum > 0.0),
                "{energy} eV: missing reaction-only {}",
                species.chemistry_name()
            );
        }
    }
    println!(
        "M6 snapshot diagnostic: seven option1 dissociation products; reference additionally contains \
         H2O2, HO2°, HO2m, Om (O^-), O2, O2m and O3. Scheduler call order establishes zero-time \
         chemistry before the first positive-step pre-callback; see docs/08."
    );
}

#[test]
#[ignore = "requires validation/reference/physchem.csv and provenance_m6.toml from Geant4 11.4.1"]
fn m6_initial_species_counts_and_rms_match_geant4() {
    assert!(reference_dir().join("provenance_m6.toml").is_file());
    let rows = rows();
    assert_counts_and_rms(&rows);
}

fn assert_counts_and_rms(rows: &[Row]) {
    let mut cases = BTreeMap::<u64, (f64, u64)>::new();
    for r in rows {
        cases.insert(r.energy.to_bits(), (r.energy, r.n));
    }
    let root = repo_data_dir();
    let table =
        ProcessTable::option2_electron(&DataDir::new(&root).unwrap(), ElasticRotation::Geant4)
            .unwrap();
    let engine = PhysChemEngine::option3().unwrap();
    let mut failures = Vec::<String>::new();
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
            let name = g4.species.chemistry_name();
            let rust_mean = ours.alive_per_history.value;
            if !(se == 0.0 || delta / se <= 4.5) {
                failures.push(format!(
                    "{energy} {name}: z={:.2} (g4 {g4_mean:.4}, rust {rust_mean:.4})",
                    delta / se
                ));
            }
            if g4_mean > 0.1 && rust_mean > 0.1 && delta / g4_mean > 0.15 {
                failures.push(format!(
                    "{energy} {name} count: diff={:.1}% (g4 {g4_mean:.4}, rust {rust_mean:.4})",
                    100.0 * delta / g4_mean
                ));
            }
            if g4.sum > 0.0 {
                let g4_rms = (g4.sum_r2 / g4.sum).sqrt();
                let rust_rms = ours.rms_radius_nm.unwrap();
                if (rust_rms - g4_rms).abs() / g4_rms > 0.20 {
                    failures.push(format!(
                        "{energy} {name} RMS: diff={:.1}% (g4 {g4_rms:.3} nm, rust {rust_rms:.3} nm)",
                        100.0 * (rust_rms - g4_rms).abs() / g4_rms
                    ));
                }
            }
        }
    }
    // All rows are evaluated so a failing gate reports every offending row at once.
    assert!(
        failures.is_empty(),
        "{} failing checks:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// RR-001, registered before the new Geant4 run. This is a distinct stage-matched gate:
/// every one of the seven dissociation species at 1/10/100/900 keV is compared with exactly
/// the original z <= 4.5, 15% non-small count and 20% RMS limits (at most 28 tests/family).
/// All four energies and the requested 20000/2000/200/20 histories are required. Missing species,
/// reaction-only products, wrong boundary, invalid moments or incomplete output fail validation.
/// A pass can validate the M6 boundary; a failure leaves Q21/R14 and any remaining bias open.
#[test]
#[ignore = "RR-001: awaiting physchem_prechem.csv and provenance_m6_prechem.toml, user-run Geant4 only"]
fn m6_prechem_counts_and_rms_match_geant4() {
    let csv = std::fs::read_to_string(reference_dir().join("physchem_prechem.csv"))
        .expect("RR-001 awaiting data: physchem_prechem.csv");
    assert!(csv.contains("mode physchem_prechem"));
    assert!(csv.contains("# snapshot_boundary: after_dissociation_before_irt; global_time_ps: 1; chemical_reactions: 0"));
    let provenance: toml::Value = toml::from_str(
        &std::fs::read_to_string(reference_dir().join("provenance_m6_prechem.toml"))
            .expect("RR-001 awaiting provenance_m6_prechem.toml"),
    )
    .unwrap();
    for (key, expected) in [
        ("geant4_version", "11.4.1"),
        ("g4emlow_version", "8.8"),
        ("g4emlow_md5", "328330009df633f7e9b3a9f445745298"),
        ("physics_constructor", "G4EmDNAPhysics_option2"),
        ("chemistry_constructor", "G4EmDNAChemistry_option3"),
        ("chemistry_time_step_model", "IRT"),
        ("snapshot_boundary", "after_dissociation_before_irt"),
        ("fluo_auger", "off"),
    ] {
        assert_eq!(provenance[key].as_str(), Some(expected), "{key}");
    }
    assert_eq!(provenance["scheduler_snapshot_ps"].as_float(), Some(1.0));
    assert_eq!(provenance["seed"].as_integer(), Some(20261020));
    assert_eq!(
        provenance["command_line"].as_str(),
        Some(
            "g4rs_ref physchem_prechem ../../reference/physchem_prechem.csv 20261020 1000:20000,10000:2000,100000:200,900000:20"
        )
    );
    for key in ["geant4_commit", "generator", "date", "compiler", "platform"] {
        assert!(
            provenance
                .get(key)
                .and_then(toml::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty()),
            "missing or empty {key}"
        );
    }
    assert!(provenance["generator_dirty"].as_bool().is_some());
    let rows = parse_rows(&csv);
    let expected_species = BTreeSet::from([
        Species::H3Op,
        Species::OH,
        Species::OHm,
        Species::e_aq,
        Species::H,
        Species::H2,
        Species::Oxy,
    ]);
    assert_eq!(rows.len(), 28, "seven species at each of four energies");
    for (energy, n) in [
        (1_000.0, 20_000),
        (10_000.0, 2_000),
        (100_000.0, 200),
        (900_000.0, 20),
    ] {
        let case: Vec<_> = rows.iter().filter(|r| r.energy == energy).collect();
        assert_eq!(case.len(), 7, "{energy}: incomplete or duplicate species");
        assert_eq!(
            case.iter().map(|r| r.species).collect::<BTreeSet<_>>(),
            expected_species
        );
        for row in case {
            assert_eq!(row.n, n);
            assert!(row.sum.is_finite() && row.sum > 0.0);
            assert!(row.sum2.is_finite() && row.sum2 >= 0.0);
            assert!(row.sum_r2.is_finite() && row.sum_r2 > 0.0);
            assert!(
                row.sum2 + 1e-12 * row.sum2 >= row.sum * row.sum / row.n as f64,
                "{energy} {:?}: inconsistent count moments",
                row.species
            );
        }
    }
    assert_counts_and_rms(&rows);
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

/// Q21/R14 diagnostic, not a Geant4 agreement gate. No ordering is asserted to
/// reproduce the IT scheduler. Physics is transported once per history; all six
/// conversions restart its PhysChem stream, and only a separate seed shuffles.
#[test]
#[ignore = "full-cascade ordering study at the four M6 reference energies"]
fn m6_recombination_ordering_ablation() {
    use dna_core::physchem::PhysChemKind;
    use dna_core::rng::{RngStreams, Stage};
    use dna_sim::history::{radial_edges_nm, run_history, spectrum_edges_ev};

    const MODES: [&str; 6] = [
        "event",
        "reverse",
        "shuffle",
        "excitation-first",
        "ionisation-first",
        "off",
    ];
    let root = repo_data_dir();
    let table =
        ProcessTable::option2_electron(&DataDir::new(&root).unwrap(), ElasticRotation::Geant4)
            .unwrap();
    let engine = PhysChemEngine::option3().unwrap();
    let off = engine.clone().with_recombination_enabled(false);
    let streams = RngStreams::new(20261017);
    let permutation_streams = RngStreams::new(20261018);
    let spectra = spectrum_edges_ev();
    let radial = radial_edges_nm();
    for (energy, n) in [
        (1_000.0, 20_000),
        (10_000.0, 2_000),
        (100_000.0, 200),
        (900_000.0, 20),
    ] {
        let mut config = RunConfig::new(energy, n);
        config.master_seed = 20261017;
        config.record_physchem = true;
        let mut recombinations = [0u64; 6];
        let mut candidates = [0u64; 6];
        let mut totals = [[0u64; 15]; 6];
        let mut delta = [[0.0f64; 2]; 6];
        let mut holes = 0u64;
        for h in 0..n {
            let history = run_history(&table, &config, &streams, h, &spectra, &radial).unwrap();
            let events = &history.physchem;
            let waters: Vec<_> = events
                .iter()
                .filter(|e| e.kind != PhysChemKind::Solvation)
                .cloned()
                .collect();
            let mut baseline = None;
            for (mode, _) in MODES.iter().enumerate() {
                let mut ordered = waters.clone();
                match mode {
                    1 => ordered.reverse(),
                    2 => {
                        let mut permutation =
                            permutation_streams.stream(h, Stage::PhysChem).unwrap();
                        for i in (1..ordered.len()).rev() {
                            let j = (permutation.uniform_open01() * (i + 1) as f64) as usize;
                            ordered.swap(i, j);
                        }
                    }
                    3 => ordered.sort_by_key(|e| e.kind != PhysChemKind::Excitation),
                    4 => ordered.sort_by_key(|e| e.kind != PhysChemKind::Ionisation),
                    _ => {}
                }
                // Keep electron insertion/tie order fixed: vary only water order.
                let mut input: Vec<_> = events
                    .iter()
                    .filter(|e| e.kind == PhysChemKind::Solvation)
                    .cloned()
                    .collect();
                input.extend(ordered);
                let selected_engine = if mode == 5 { &off } else { &engine };
                let mut rng = streams.stream(h, Stage::PhysChem).unwrap();
                let result = selected_engine.convert_history(&input, &mut rng).unwrap();
                if h == 0 {
                    let mut repeat_rng = streams.stream(h, Stage::PhysChem).unwrap();
                    assert_eq!(
                        result,
                        selected_engine
                            .convert_history(&input, &mut repeat_rng)
                            .unwrap()
                    );
                }
                assert!(result.species.iter().all(|s| s.time.ps() == 1.0));
                assert_eq!(result.stats.species_counts[Species::H2O2.index()], 0);
                if mode == 0 {
                    holes += result.stats.events[1];
                    baseline = Some(result.stats.clone());
                }
                let baseline = baseline.as_ref().unwrap();
                assert_eq!(result.stats.events, baseline.events);
                assert!(result.stats.recombinations <= result.stats.events[1]);
                let difference =
                    result.stats.recombinations as f64 - baseline.recombinations as f64;
                delta[mode][0] += difference;
                delta[mode][1] += difference * difference;
                recombinations[mode] += result.stats.recombinations;
                candidates[mode] += result.stats.holes_with_candidate;
                for (total, count) in totals[mode].iter_mut().zip(result.stats.species_counts) {
                    *total += count;
                }
            }
        }
        assert_eq!(recombinations[5], 0);
        println!(
            "\nQ21/R14 {:.0} keV, {n} histories, seed 20261017, shuffle seed 20261018, {holes} holes",
            energy / 1000.0
        );
        println!(
            "mode rec_percent candidate_percent paired_delta_rec_per_history +/- SE; alive means: e_aq OH H3Op H H2 Oxy OHm"
        );
        for (mode, name) in MODES.iter().enumerate() {
            let mean = delta[mode][0] / n as f64;
            let se = ((delta[mode][1] - delta[mode][0].powi(2) / n as f64).max(0.0)
                / (n - 1) as f64
                / n as f64)
                .sqrt();
            print!(
                "{name:<17} {:8.4} {:8.4} {mean:+.6} +/- {se:.6};",
                100.0 * recombinations[mode] as f64 / holes as f64,
                100.0 * candidates[mode] as f64 / holes as f64
            );
            for species in [
                Species::e_aq,
                Species::OH,
                Species::H3Op,
                Species::H,
                Species::H2,
                Species::Oxy,
                Species::OHm,
            ] {
                print!(" {:.6}", totals[mode][species.index()] as f64 / n as f64);
            }
            println!();
        }
    }
}
