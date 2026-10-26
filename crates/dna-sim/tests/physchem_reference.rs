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

/// Read the RR-001 evidence as a complete, stage-specific set before running transport.
fn read_prechem_rows(csv: &str, provenance_text: &str) -> Result<Vec<Row>, String> {
    const COMMAND: &str = "g4rs_ref physchem_prechem ../../reference/physchem_prechem.csv 20261020 1000:20000,10000:2000,100000:200,900000:20";
    let provenance: toml::Value = toml::from_str(provenance_text).map_err(|e| e.to_string())?;
    let expected = [
        ("geant4_version", "11.4.1"),
        ("g4emlow_version", "8.8"),
        ("g4emlow_md5", "328330009df633f7e9b3a9f445745298"),
        ("physics_constructor", "G4EmDNAPhysics_option2"),
        ("chemistry_constructor", "G4EmDNAChemistry_option3"),
        ("chemistry_time_step_model", "IRT"),
        ("snapshot_boundary", "after_dissociation_before_irt"),
        ("fluo_auger", "off"),
        ("command_line", COMMAND),
    ];
    for (key, value) in expected {
        if provenance.get(key).and_then(toml::Value::as_str) != Some(value) {
            return Err(format!("invalid or missing provenance {key}"));
        }
    }
    if provenance
        .get("scheduler_snapshot_ps")
        .and_then(toml::Value::as_float)
        != Some(1.0)
        || provenance.get("seed").and_then(toml::Value::as_integer) != Some(20261020)
    {
        return Err("invalid scheduler time or seed".into());
    }
    for key in [
        "geant4_commit",
        "snapshot",
        "generator",
        "data_commit",
        "request",
        "date",
        "compiler",
        "platform",
    ] {
        if !provenance
            .get(key)
            .and_then(toml::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
        {
            return Err(format!("missing or empty provenance {key}"));
        }
    }
    if provenance
        .get("generator_dirty")
        .and_then(toml::Value::as_bool)
        .is_none()
    {
        return Err("missing generator_dirty".into());
    }

    let mut lines = csv.lines();
    for required in [
        "# generator: g4rs_ref (validation/geant4/g4rs_ref), mode physchem_prechem",
        "# geant4_version_number: 1141 ($Name: geant4-11-04-patch-01 [MT]$)",
        "# physics: G4EmDNAPhysics_option2; material G4_WATER; fluo/auger OFF; events 0; seed 20261020",
        "# G4EmDNAChemistry_option3; IRT selected; no reaction table or process changes",
        "# snapshot_boundary: after_dissociation_before_irt; global_time_ps: 1; chemical_reactions: 0",
        "energy_eV,n_events,species,sum_count,sum_count2,sum_r2_nm2",
    ] {
        if lines.next() != Some(required) {
            return Err(format!("missing or invalid CSV metadata: {required}"));
        }
    }
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    let expected_species = BTreeSet::from([
        Species::H3Op,
        Species::OH,
        Species::OHm,
        Species::e_aq,
        Species::H,
        Species::H2,
        Species::Oxy,
    ]);
    let cases = [
        (1_000.0, 20_000),
        (10_000.0, 2_000),
        (100_000.0, 200),
        (900_000.0, 20),
    ];
    for (index, line) in lines.enumerate() {
        let columns: Vec<_> = line.split(',').collect();
        if columns.len() != 6 {
            return Err(format!("row {index}: expected six columns"));
        }
        let parse_float = |column: usize| -> Result<f64, String> {
            columns[column]
                .parse::<f64>()
                .map_err(|_| format!("row {index}: invalid column {column}"))
        };
        let energy = parse_float(0)?;
        let n = columns[1]
            .parse::<u64>()
            .map_err(|_| format!("row {index}: invalid count"))?;
        let species = species(columns[2]).ok_or_else(|| format!("row {index}: unknown species"))?;
        let sum = parse_float(3)?;
        let sum2 = parse_float(4)?;
        let sum_r2 = parse_float(5)?;
        if !cases
            .iter()
            .any(|&(e, histories)| energy == e && n == histories)
            || !expected_species.contains(&species)
        {
            return Err(format!(
                "row {index}: unexpected energy, history count or species"
            ));
        }
        if !seen.insert((energy.to_bits(), species)) {
            return Err(format!("row {index}: duplicate species-energy"));
        }
        if !sum.is_finite()
            || sum <= 0.0
            || sum.fract() != 0.0
            || !sum2.is_finite()
            || sum2 < sum
            || sum2.fract() != 0.0
            || !sum_r2.is_finite()
            || sum_r2 <= 0.0
            || sum2 + 1e-12 * sum2 < sum * sum / n as f64
            || !(sum_r2 / sum).sqrt().is_finite()
        {
            return Err(format!("row {index}: invalid count or spatial moments"));
        }
        rows.push(Row {
            energy,
            n,
            species,
            sum,
            sum2,
            sum_r2,
        });
    }
    if rows.len() != cases.len() * expected_species.len() {
        return Err(format!(
            "incomplete species-energy grid: {} rows",
            rows.len()
        ));
    }
    Ok(rows)
}

fn count_z_passes(g4_mean: f64, rust_mean: f64, g4_se: f64, rust_se: f64) -> bool {
    if ![g4_mean, rust_mean, g4_se, rust_se]
        .iter()
        .all(|x| x.is_finite())
        || g4_se < 0.0
        || rust_se < 0.0
    {
        return false;
    }
    let se = g4_se.hypot(rust_se);
    let delta = (rust_mean - g4_mean).abs();
    se.is_finite()
        && delta.is_finite()
        && (if se == 0.0 {
            delta == 0.0
        } else {
            delta / se <= 4.5
        })
}

#[test]
fn m6_prechem_reader_rejects_invalid_evidence() {
    let csv = include_str!("../../../validation/reference/physchem_prechem.csv");
    let provenance = include_str!("../../../validation/reference/provenance_m6_prechem.toml");
    assert_eq!(read_prechem_rows(csv, provenance).unwrap().len(), 28);

    let first = "1000,20000,H3O^1,805954,32718374,969247673.05992162";
    let second = "1000,20000,H^0,213896,2554108,265375101.95173156";
    for changed in [
        csv.replace("after_dissociation_before_irt", "after_zero_time_reactions"),
        csv.replace("G4EmDNAChemistry_option3", "G4EmDNAChemistry_option1"),
        csv.replace("seed 20261020", "seed 20261021"),
        csv.replace(first, ""),
        csv.replace(second, first),
        csv.replace(first, "1000,20000,H3O^1,-1,32718374,969247673.05992162"),
        csv.replace(first, "1000,20000,H3O^1,805954,1,969247673.05992162"),
        csv.replace(first, "1000,20000,H3O^1,805954,32718374,NaN"),
        csv.replace(first, "1000,20000,H3O^1,805954,32718374,-1"),
        csv.replace(first, "1000,20000,H3O^1,805954,32718374"),
        csv.replace("sum_r2_nm2", "sum_r_nm"),
    ] {
        assert!(
            read_prechem_rows(&changed, provenance).is_err(),
            "accepted altered CSV"
        );
    }
    for changed in [
        provenance.replace("after_dissociation_before_irt", "after_zero_time_reactions"),
        provenance.replace("G4EmDNAChemistry_option3", "G4EmDNAChemistry_option1"),
        provenance.replace(
            "seed                     = 20261020",
            "seed                     = 20261021",
        ),
        provenance.replace(
            "compiler                 = \"MSVC 19.50.35729 x64, Ninja, RelWithDebInfo, link /STACK:536870912\"",
            "compiler                 = \"\"",
        ),
        provenance.replace("data_commit              =", "missing_data_commit      ="),
        provenance.replace(
            "generator_dirty          = false",
            "generator_dirty          = \"false\"",
        ),
        provenance.replace(
            "scheduler_snapshot_ps    = 1.0",
            "scheduler_snapshot_ps    = 0.0",
        ),
    ] {
        assert!(
            read_prechem_rows(csv, &changed).is_err(),
            "accepted altered provenance: {changed}"
        );
    }
}

#[test]
fn m6_count_z_requires_finite_quantities_and_exact_zero_error_equality() {
    assert!(count_z_passes(1.0, 1.0, 0.0, 0.0));
    assert!(!count_z_passes(1.0, 2.0, 0.0, 0.0));
    assert!(count_z_passes(1.0, 1.1, 0.1, 0.0));
    assert!(!count_z_passes(1.0, 2.0, 0.1, 0.0));
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for values in [
            [bad, 1.0, 0.1, 0.1],
            [1.0, bad, 0.1, 0.1],
            [1.0, 1.0, bad, 0.1],
            [1.0, 1.0, 0.1, bad],
        ] {
            assert!(!count_z_passes(values[0], values[1], values[2], values[3]));
        }
    }
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
            if !count_z_passes(g4_mean, rust_mean, g4_se, ours.alive_per_history.std_err) {
                failures.push(format!(
                    "{energy} {name}: z={:.2} (g4 {g4_mean:.4}, rust {rust_mean:.4})",
                    delta / se
                ));
            }
            if (!g4_mean.is_finite() || !rust_mean.is_finite())
                || (g4_mean > 0.1 && rust_mean > 0.1 && delta / g4_mean > 0.15)
            {
                failures.push(format!(
                    "{energy} {name} count: diff={:.1}% (g4 {g4_mean:.4}, rust {rust_mean:.4})",
                    100.0 * delta / g4_mean
                ));
            }
            if g4.sum > 0.0 {
                let g4_rms = (g4.sum_r2 / g4.sum).sqrt();
                let rust_rms = ours.rms_radius_nm.unwrap();
                if !g4_rms.is_finite()
                    || !rust_rms.is_finite()
                    || (rust_rms - g4_rms).abs() / g4_rms > 0.20
                {
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
    let provenance = std::fs::read_to_string(reference_dir().join("provenance_m6_prechem.toml"))
        .expect("RR-001 awaiting provenance_m6_prechem.toml");
    let rows = read_prechem_rows(&csv, &provenance).expect("invalid RR-001 prechem evidence");
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

/// Diagnostic production sample, never an M6 acceptance gate. Run explicitly with
/// `cargo test --release -p dna-sim m6_spatial_diagnostic_900kev -- --ignored --nocapture`.
/// Output defaults to the session scratch directory; an optional
/// DNA_SPATIAL_DIAGNOSTIC_CSV path must also be inside that directory. Existing
/// files are never replaced. Geant4 is neither required nor invoked.
#[test]
#[ignore = "900 keV spatial diagnostic: five preregistered seeds x 40 complete histories"]
fn m6_spatial_diagnostic_900kev() {
    use dna_sim::run::run_physchem_summary_with_diagnostics;
    use dna_sim::spatial_diagnostic::{
        ANALYSIS_SEED, BOOTSTRAP_REPLICATES, Boundary, CsvSpatialDiagnosticWriter,
        DIAGNOSTIC_SEEDS, DiagnosticDataset, Implementation, SamplePlan,
    };
    use std::io::{BufReader, BufWriter, Write};

    let scratch = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.scratch/tests/2026-10-19__m6-900kev-spatial-rms-investigation");
    std::fs::create_dir_all(&scratch).unwrap();
    let scratch = scratch.canonicalize().unwrap();
    let output = std::env::var_os("DNA_SPATIAL_DIAGNOSTIC_CSV")
        .map(PathBuf::from)
        .unwrap_or_else(|| scratch.join("spatial_rust_900kev_40.csv"));
    let parent = output
        .parent()
        .expect("CSV output requires a parent directory")
        .canonicalize()
        .unwrap();
    assert!(
        parent.starts_with(&scratch),
        "diagnostic artifacts must stay in session scratch"
    );
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .expect("choose a new scratch CSV path; never overwrite existing evidence");
    let mut writer = CsvSpatialDiagnosticWriter::new(BufWriter::new(file)).unwrap();
    let table = ProcessTable::option2_electron(
        &DataDir::new(repo_data_dir()).unwrap(),
        ElasticRotation::Geant4,
    )
    .unwrap();
    let engine = PhysChemEngine::option3().unwrap();
    let plan = SamplePlan::production(40).unwrap();
    for seed in DIAGNOSTIC_SEEDS {
        let mut config = RunConfig::new(900_000.0, 40);
        config.master_seed = seed;
        // A full 900 keV cascade has many entity rows; hold only two histories.
        config.batch_size = 2;
        run_physchem_summary_with_diagnostics(&table, &engine, &config, Some(&mut writer)).unwrap();
        println!("Completed Rust spatial diagnostic seed {seed}: 40 histories at 900000 eV");
    }
    writer.into_inner().flush().unwrap();
    let dataset = DiagnosticDataset::read_csv(
        BufReader::new(std::fs::File::open(&output).unwrap()),
        Implementation::Rust,
        &plan,
    )
    .unwrap();
    let summary = dataset.summarize().unwrap();
    let precision = dataset.pooled_rms_precision().unwrap();
    println!("CSV: {}", output.display());
    println!(
        "Seeds: {DIAGNOSTIC_SEEDS:?}; histories/seed: 40; analysis seed: {ANALYSIS_SEED}; bootstrap replicates: {BOOTSTRAP_REPLICATES}"
    );
    println!(
        "Rust component of primary estimand (pooled seven-species survivor RMS): {:.9} nm; 95% component interval [{:.9}, {:.9}] nm; achieved component relative half-width: {:.6} percentage points",
        precision.pooled_rms_nm,
        precision.lower_nm,
        precision.upper_nm,
        100.0 * precision.relative_half_width
    );
    println!(
        "Primary Rust/Geant4 ratio and its achieved precision: unavailable until the independent Geant4 diagnostic exists. Rust-only precision does not trigger sample expansion."
    );
    for boundary in Boundary::ALL {
        println!(
            "Secondary pooled {}: {:?}",
            boundary.name(),
            summary.boundaries[boundary.index()]
        );
    }
    for ((boundary, category), population) in &summary.categories {
        println!(
            "Secondary {} / {}: {population:?}",
            boundary.name(),
            category.name()
        );
    }
    for history in &summary.histories {
        println!(
            "Secondary seed {} history {}: {:?}",
            history.seed, history.history, history.boundaries
        );
    }
}

/// RR-002 consumer for the five-boundary 900 keV diagnostic. Geant4 data and
/// provenance are produced only by the project lead; Rust is generated locally
/// from the same frozen five-seed x 40-history design when this test is invoked.
#[test]
#[ignore = "RR-002: awaiting spatial_geant4_900kev_40.csv and provenance_m6_spatial_900kev_40.toml"]
fn m6_900kev_spatial_diagnostic() {
    use dna_sim::run::run_physchem_summary_with_diagnostics;
    use dna_sim::spatial_diagnostic::{
        Boundary, CsvSpatialDiagnosticWriter, DIAGNOSTIC_SEEDS, DiagnosticDataset, Implementation,
        PrecisionDecision, SamplePlan, analyze,
    };
    use std::fs::File;
    use std::io::{BufRead, BufReader, BufWriter, Write};

    const CSV_NAME: &str = "spatial_geant4_900kev_80_local.csv";
    const PROVENANCE_NAME: &str = "provenance_m6_spatial_900kev_80_local.toml";
    const COMMAND: &str = "build-msvc-e\\g4rs_ref.exe physchem_spatial_diagnostic ..\\..\\..\\.scratch\\rr-002-80\\spatial_geant4_900kev_80_local.csv 80 20261021,20261022,20261023,20261024,20261025";
    let scratch = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/rr-002-80");
    let csv_path = scratch.join(CSV_NAME);
    let provenance_path = scratch.join(PROVENANCE_NAME);
    assert!(
        csv_path.is_file() && provenance_path.is_file(),
        "RR-002 awaiting data: {CSV_NAME} and {PROVENANCE_NAME}; project lead must run Geant4 and commit both complete new files"
    );

    let provenance: toml::Value = toml::from_str(
        &std::fs::read_to_string(&provenance_path)
            .unwrap_or_else(|e| panic!("cannot read {PROVENANCE_NAME}: {e}")),
    )
    .unwrap_or_else(|e| panic!("invalid {PROVENANCE_NAME}: {e}"));
    for (key, expected) in [
        ("geant4_version", "11.4.1"),
        ("g4emlow_version", "8.8"),
        ("physics_constructor", "G4EmDNAPhysics_option2"),
        ("chemistry_constructor", "G4EmDNAChemistry_option3"),
        ("chemistry_time_step_model", "IRT"),
        ("diagnostic_stage", "global_1_ps_pre_irt"),
        ("fluo_auger", "off"),
    ] {
        assert_eq!(provenance[key].as_str(), Some(expected), "{key}");
    }
    assert_eq!(provenance["primary_energy_eV"].as_integer(), Some(900_000));
    assert_eq!(provenance["histories_per_seed"].as_integer(), Some(80));
    assert_eq!(provenance["analysis_seed"].as_integer(), Some(20261026));
    assert_eq!(
        provenance["bootstrap_replicates"].as_integer(),
        Some(20_000)
    );
    assert_eq!(provenance["command_line"].as_str(), Some(COMMAND));
    assert!(provenance["generator_dirty"].as_bool().is_some());
    for key in ["geant4_commit", "generator", "date", "compiler", "platform"] {
        assert!(
            provenance[key]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "missing provenance field {key}"
        );
    }
    let seeds: Vec<_> = provenance["seeds"]
        .as_array()
        .expect("provenance seeds must be an array")
        .iter()
        .map(|value| value.as_integer().expect("seed must be an integer") as u64)
        .collect();
    assert_eq!(seeds, DIAGNOSTIC_SEEDS);

    let comments: Vec<_> = BufReader::new(File::open(&csv_path).unwrap())
        .lines()
        .map_while(Result::ok)
        .filter(|line| line.starts_with('#'))
        .collect();
    for marker in [
        "mode physchem_spatial_diagnostic",
        "primary_energy_eV: 900000",
        "global_time_ps: 1; chemical_reactions: 0",
        "histories_per_seed: 80; expected_seed_count: 5",
        "complete: true; seeds: 20261021,20261022,20261023,20261024,20261025; histories_per_seed: 80",
    ] {
        assert!(
            comments.iter().any(|line| line.contains(marker)),
            "incomplete or wrong-stage Geant4 CSV: missing marker '{marker}'"
        );
    }
    let plan = SamplePlan::production(80).unwrap();
    let geant4 = DiagnosticDataset::read_csv(
        BufReader::new(File::open(&csv_path).unwrap()),
        Implementation::Geant4,
        &plan,
    )
    .expect("RR-002 Geant4 CSV is malformed, partial, or has wrong seed/history coverage");

    std::fs::create_dir_all(&scratch).unwrap();
    let rust_path = scratch.join("spatial_rust_900kev_80_local.csv");
    let rust_file = std::fs::File::create(&rust_path).unwrap();
    let mut writer = CsvSpatialDiagnosticWriter::new(BufWriter::new(rust_file)).unwrap();
    let table = ProcessTable::option2_electron(
        &DataDir::new(repo_data_dir()).unwrap(),
        ElasticRotation::Geant4,
    )
    .unwrap();
    let engine = PhysChemEngine::option3().unwrap();
    for seed in DIAGNOSTIC_SEEDS {
        let mut config = RunConfig::new(900_000.0, 80);
        config.master_seed = seed;
        config.batch_size = 2;
        run_physchem_summary_with_diagnostics(&table, &engine, &config, Some(&mut writer)).unwrap();
    }
    writer.into_inner().flush().unwrap();
    let rust = DiagnosticDataset::read_csv(
        BufReader::new(std::fs::File::open(&rust_path).unwrap()),
        Implementation::Rust,
        &plan,
    )
    .unwrap();
    let analysis = analyze(&rust, &geant4).unwrap();
    println!(
        "Primary pooled survivor RMS ratio Rust/Geant4: {:.9}; 95% interval [{:.9}, {:.9}]; relative half-width {:.3}%",
        analysis.primary_ratio,
        analysis.primary_interval.lower,
        analysis.primary_interval.upper,
        100.0 * analysis.relative_half_width
    );
    println!("Simultaneous five-boundary intervals:");
    for interval in analysis.boundaries {
        println!(
            "  {} ratio {:.9}, interval [{:.9}, {:.9}]",
            interval.boundary.name(),
            interval.ratio,
            interval.interval.lower,
            interval.interval.upper
        );
    }
    println!("{}", analysis.divergence_statement());
    println!(
        "Precision-only decision: {:?}",
        analysis.precision_decision().unwrap()
    );
    for boundary in Boundary::ALL {
        println!(
            "Secondary {}: Rust {:?}; Geant4 {:?}",
            boundary.name(),
            analysis.rust.boundaries[boundary.index()],
            analysis.geant4.boundaries[boundary.index()]
        );
    }
    for ((boundary, category), rust_population) in &analysis.rust.categories {
        println!(
            "Secondary {} / {}: Rust {:?}; Geant4 {:?}",
            boundary.name(),
            category.name(),
            rust_population,
            analysis
                .geant4
                .categories
                .get(&(*boundary, *category))
                .unwrap()
        );
    }
    match analysis.precision_decision().unwrap() {
        PrecisionDecision::ExpandToHistoriesPerSeed(80) => println!(
            "More histories required solely by precision: rerun both implementations at 80 histories per seed."
        ),
        PrecisionDecision::ExpandToHistoriesPerSeed(160) => println!(
            "More histories required solely by precision: rerun both implementations at 160 histories per seed."
        ),
        PrecisionDecision::TargetReached => println!(
            "No expansion: primary relative interval half-width is at most 4 percentage points."
        ),
        PrecisionDecision::CapReached => println!(
            "Precision cap reached at 160 histories per seed; report the interval without further expansion."
        ),
        decision => panic!("invalid preregistered precision decision: {decision:?}"),
    }
}
