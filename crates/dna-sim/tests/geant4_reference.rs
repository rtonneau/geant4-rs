//! L2/L3 comparison with Geant4 11.4.1 (docs/07 V2, V3, V5).
//!
//! The four `#[ignore]` tests need CSV files produced by `validation/geant4/g4rs_ref` on a machine
//! with Geant4 and committed to `validation/reference/` together with `provenance.toml`
//! (instructions: `validation/geant4/README.md`). No Geant4 number is invented or embedded here:
//! until the files exist the tests stay ignored (`cargo test -- --ignored` fails with a message
//! naming the missing file).
//!
//! **Pre-registered acceptance (docs/07, fixed before any comparison):**
//! * V2 (cross sections per volume, `xs_per_volume.csv`): relative difference < 1e-9 at the nodes
//!   of the list `NODE_ENERGIES`, < 1e-6 at all other energies, per process.
//! * V3 (interaction-type probabilities `Sigma_i / Sigma`): relative difference < 1e-9 at the
//!   nodes, < 1e-6 elsewhere, wherever all three processes are open.
//! * V5 (means of sampled quantities): `|a - b| < 3 sqrt(se_a^2 + se_b^2)` **and** `|a - b| /
//!   |b| < 2 %` (the relative systematic tolerance). Fractions/counts of rare processes whose
//!   own statistical error exceeds 2 % are tested with the z criterion only (the 2 % cut would
//!   be below their noise); this is stated in the failure message.
//! * Quantities: mean first free path and first-interaction process fractions (`mfp`), mean track
//!   length, deposited energy, remaining energy, secondary multiplicity and secondary energy of
//!   the primary track with a 100 eV tracking cut (`track`).
//!
//! **M3 note.** These tests are the M2 gate and must keep comparing like with like: the Geant4
//! reference was generated with secondaries killed and with a tracking cut at 100 eV (track) or
//! one interaction (mfp), and its CSV lists only the elastic, excitation and ionisation
//! processes. The Rust side therefore runs with `transport_secondaries = false` and compares the
//! three processes of `M2_PROCESSES`; Sanche, Melton and solvation do not act in these set-ups
//! (primaries >= 150 eV, stopped below 100 eV) except for the Rust-only cross-section sum, which
//! is deliberately restricted to the same three processes. The full-cascade comparison is in
//! `slowing_down_reference.rs` (M3).
//!
//! The non-ignored tests exercise the comparison machinery itself on synthetic numbers and on a
//! Rust-versus-Rust run with different seeds (a false-positive check, **not** a validation).

use dna_data::datadir::{DataDir, repo_data_dir};
use dna_physics::models::{ElasticRotation, ProcessKind};
use dna_physics::table::ProcessTable;
use dna_sim::config::RunConfig;
use dna_sim::run::run_detailed;
use std::path::PathBuf;

/// The processes listed in the M2 reference CSVs.
const M2_PROCESSES: [ProcessKind; 3] = [
    ProcessKind::Elastic,
    ProcessKind::Excitation,
    ProcessKind::Ionisation,
];

/// Run configuration of the M2 reference set-up: secondaries recorded, not transported.
fn m2_config(energy: f64, n: u64) -> RunConfig {
    let mut cfg = RunConfig::new(energy, n);
    cfg.transport_secondaries = false;
    cfg
}

/// Energies (eV) of the `mfp` reference: identical to the list hard-wired in `g4rs_ref.cc`.
const MFP_ENERGIES: [f64; 9] = [150.0, 300.0, 1e3, 3e3, 1e4, 3e4, 1e5, 3e5, 9e5];
/// Energies (eV) of the `track` reference (`g4rs_ref.cc`) and its tracking cut.
const TRACK_ENERGIES: [f64; 5] = [200.0, 1e3, 1e4, 1e5, 9e5];
const TRACK_CUT_EV: f64 = 100.0;
/// Energies of `xs_per_volume.csv` that are table nodes or window boundaries (`RunXs`).
const NODE_ENERGIES: [f64; 8] = [7.4, 9.0, 11.0, 100.0, 1e3, 1e4, 1e5, 1e6];

// ------------------------------------------------------------------ machinery ------------

fn reference_dir() -> PathBuf {
    repo_data_dir().join("../validation/reference")
}

/// Rows of a CSV with a header: comment lines (`#`) skipped, fields split on commas.
fn parse_csv(text: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let mut lines = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'));
    let header = lines
        .next()
        .expect("CSV has no header")
        .split(',')
        .map(str::to_owned)
        .collect();
    let rows = lines
        .map(|l| l.split(',').map(str::to_owned).collect())
        .collect();
    (header, rows)
}

fn read_reference(name: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let path = reference_dir().join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "missing Geant4 reference {}: {e}. Generate it as described in \
             validation/geant4/README.md and commit it with provenance.toml",
            path.display()
        )
    });
    let provenance_path = reference_dir().join("provenance.toml");
    let provenance = std::fs::read_to_string(&provenance_path)
        .unwrap_or_else(|e| panic!("missing {}: {e}", provenance_path.display()));
    validate_m2_reference(name, &text, &provenance)
        .unwrap_or_else(|e| panic!("invalid Geant4 reference {name}: {e}"));
    parse_csv(&text)
}

/// Validate the complete M2 evidence contract before any row reaches the comparison code.
fn validate_m2_reference(name: &str, csv: &str, provenance: &str) -> Result<(), String> {
    let fail = |message: String| Err(message);
    if csv.trim().is_empty() {
        return fail("CSV is missing or empty".into());
    }
    let prov: toml::Table = provenance
        .parse()
        .map_err(|e| format!("provenance.toml is invalid TOML: {e}"))?;
    let metadata = |key: &str| -> Result<String, String> {
        prov.get(key)
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("provenance.toml lacks string `{key}`"))
    };
    for key in [
        "geant4_version",
        "geant4_commit",
        "g4emlow_version",
        "g4emlow_md5",
        "physics_constructor",
        "generator",
        "date",
        "command_lines",
        "fluo_auger",
        "compiler",
        "platform",
    ] {
        if !prov.contains_key(key) {
            return fail(format!("provenance.toml lacks `{key}`"));
        }
    }
    for key in [
        "geant4_commit",
        "g4emlow_md5",
        "generator",
        "date",
        "compiler",
        "platform",
    ] {
        if prov
            .get(key)
            .and_then(toml::Value::as_str)
            .is_none_or(str::is_empty)
        {
            return fail(format!("provenance `{key}` must be a nonempty string"));
        }
    }
    if metadata("geant4_version")? != "11.4.1" {
        return fail("provenance Geant4 version must be 11.4.1".into());
    }
    if metadata("g4emlow_version")? != "8.8" {
        return fail("provenance G4EMLOW version must be 8.8".into());
    }
    if metadata("g4emlow_md5")? != "328330009df633f7e9b3a9f445745298" {
        return fail("provenance G4EMLOW checksum must match dataset 8.8".into());
    }
    if metadata("physics_constructor")? != "G4EmDNAPhysics_option2" {
        return fail("provenance physics_constructor must be G4EmDNAPhysics_option2".into());
    }
    if metadata("fluo_auger")? != "off" {
        return fail("provenance fluo_auger must be off".into());
    }

    let mut comments = Vec::new();
    let mut data_lines = Vec::new();
    for line in csv.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with('#') {
            comments.push(line.trim_start_matches('#').trim());
        } else {
            data_lines.push(line);
        }
    }
    if data_lines.is_empty() {
        return fail("CSV has no header".into());
    }
    let header: Vec<&str> = data_lines[0].split(',').collect();
    let (mode, expected_header, energies, seed, events) = match name {
        "xs_per_volume.csv" => ("xs", XS_HEADER.as_slice(), Vec::new(), 0, 0),
        "mfp_first_interaction.csv" => (
            "mfp",
            EVENT_HEADER.as_slice(),
            MFP_ENERGIES.to_vec(),
            20261,
            1_000_000,
        ),
        "mfp_first_interaction_seed2.csv" => (
            "mfp",
            EVENT_HEADER.as_slice(),
            MFP_ENERGIES.to_vec(),
            20263,
            1_000_000,
        ),
        "track_cut100eV.csv" => (
            "track",
            EVENT_HEADER.as_slice(),
            TRACK_ENERGIES.to_vec(),
            20262,
            10_000,
        ),
        "track_cut100eV_seed2.csv" => (
            "track",
            EVENT_HEADER.as_slice(),
            TRACK_ENERGIES.to_vec(),
            20264,
            10_000,
        ),
        other => return fail(format!("unregistered M2 reference filename `{other}`")),
    };
    if header != expected_header {
        return fail(format!(
            "wrong CSV columns/units for {name}: expected {expected_header:?}, got {header:?}"
        ));
    }
    if !comments
        .iter()
        .any(|c| c.ends_with(&format!("mode {mode}")))
    {
        return fail(format!("CSV metadata has wrong or missing mode `{mode}`"));
    }
    if !comments
        .iter()
        .any(|c| c.contains("geant4_version_number: 1141"))
    {
        return fail("CSV metadata must identify Geant4 11.4.1 (version number 1141)".into());
    }
    let expected_physics = if mode == "xs" {
        "physics: G4EmDNAPhysics_option2; material G4_WATER; fluo/auger OFF; events 0; seed 0"
    } else {
        // The registered generator records these exact settings for every maintained mode.
        if mode == "mfp" {
            "physics: G4EmDNAPhysics_option2; material G4_WATER; fluo/auger OFF; events 1000000; seed"
        } else {
            "physics: G4EmDNAPhysics_option2; material G4_WATER; fluo/auger OFF; events 10000; seed"
        }
    };
    let physics = comments.iter().find(|c| c.starts_with("physics:"));
    if !physics.is_some_and(|p| p.starts_with(expected_physics)) {
        return fail(format!(
            "CSV physics/constructor/units metadata is wrong; expected `{expected_physics} ...`"
        ));
    }
    if mode != "xs" && !physics.is_some_and(|p| p.ends_with(&format!("seed {seed}"))) {
        return fail(format!("CSV seed metadata must be {seed} for {name}"));
    }
    let command_line = match name {
        "xs_per_volume.csv" => "g4rs_ref xs xs_per_volume.csv".to_owned(),
        "mfp_first_interaction.csv" => {
            "g4rs_ref mfp mfp_first_interaction.csv 1000000 20261".to_owned()
        }
        "mfp_first_interaction_seed2.csv" => {
            "g4rs_ref mfp mfp_first_interaction_seed2.csv 1000000 20263".to_owned()
        }
        "track_cut100eV.csv" => "g4rs_ref track track_cut100eV.csv 10000 20262 100".to_owned(),
        _ => "g4rs_ref track track_cut100eV_seed2.csv 10000 20264 100".to_owned(),
    };
    if !prov
        .get("command_lines")
        .and_then(toml::Value::as_array)
        .is_some_and(|v| v.iter().any(|x| x.as_str() == Some(&command_line)))
    {
        return fail(format!(
            "provenance command_lines must include `{command_line}`"
        ));
    }
    if mode == "track" && !comments.contains(&"tracking cut: 100 eV") {
        return fail("track CSV metadata must declare tracking cut 100 eV".into());
    }

    let mut rows = Vec::new();
    for (index, line) in data_lines.iter().enumerate().skip(1) {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != header.len() {
            return fail(format!(
                "row {} has {} fields; expected {}",
                index + 1,
                fields.len(),
                header.len()
            ));
        }
        rows.push(fields);
    }
    if mode == "xs" {
        validate_xs_rows(&rows)
    } else {
        validate_event_rows(name, &header, &rows, &energies, events)
    }
}

const XS_HEADER: [&str; 3] = ["energy_eV", "process", "sigma_per_volume_per_nm"];
const EVENT_HEADER: [&str; 17] = [
    "energy_eV",
    "n_events",
    "sum_L_nm",
    "sum_L2_nm2",
    "sum_edep_eV",
    "sum_edep2_eV2",
    "sum_remaining_eV",
    "sum_remaining2_eV2",
    "sum_nsec",
    "sum_nsec2",
    "sum_esec_eV",
    "sum_esec2_eV2",
    "sum_range_nm",
    "n_elastic",
    "n_excitation",
    "n_ionisation",
    "n_other",
];

fn validate_xs_rows(rows: &[Vec<&str>]) -> Result<(), String> {
    // This is the generator's registered grid: eight boundary/node values plus its 200-point
    // logarithmic sweep. Endpoint overlap is intentional in the existing source and evidence.
    let mut grid = vec![7.4, 9.0, 11.0, 100.0, 1e3, 1e4, 1e5, 1e6];
    grid.extend((0..200).map(|i| 7.4 * (1e6_f64 / 7.4).powf(i as f64 / 199.0)));
    let expected: Vec<(f64, &str)> = grid
        .iter()
        .flat_map(|e| M2_PROCESS_NAMES.map(move |p| (*e, p)))
        .collect();
    let mut found = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let e = parse_nonnegative(row[0], i, "energy_eV")?;
        if e == 0.0 {
            return Err(format!("cross-section row {} has zero energy", i + 1));
        }
        let process = row[1];
        if !M2_PROCESS_NAMES.contains(&process) {
            return Err(format!(
                "cross-section row {} has unknown process `{process}`",
                i + 1
            ));
        }
        let sigma = parse_nonnegative(row[2], i, "sigma_per_volume_per_nm")?;
        found.push((e, process, sigma));
    }
    found.sort_by(|a, b| a.1.cmp(b.1).then_with(|| a.0.total_cmp(&b.0)));
    let mut sorted_expected = expected;
    sorted_expected.sort_by(|a, b| a.1.cmp(b.1).then_with(|| a.0.total_cmp(&b.0)));
    if found.len() != sorted_expected.len()
        || found
            .iter()
            .zip(&sorted_expected)
            .any(|(actual, expected)| {
                actual.1 != expected.1
                    || (actual.0 - expected.0).abs() > 1e-12 * expected.0.abs().max(1.0)
            })
    {
        return Err(format!(
            "cross-section grid/process coverage differs: expected {} rows (208 registered energy entries × 3 processes), found {}",
            sorted_expected.len(),
            found.len()
        ));
    }
    for (index, pair) in sorted_expected.windows(2).enumerate() {
        if pair[0] == pair[1] {
            let left = found[index].2;
            let right = found[index + 1].2;
            if (left - right).abs() > 1e-12 * left.abs().max(right.abs()).max(1.0) {
                return Err(format!(
                    "repeated cross-section endpoint has conflicting values for {} at {} eV",
                    pair[0].1, pair[0].0
                ));
            }
        }
    }
    Ok(())
}

const M2_PROCESS_NAMES: [&str; 3] = ["elastic", "excitation", "ionisation"];

fn validate_event_rows(
    name: &str,
    header: &[&str],
    rows: &[Vec<&str>],
    energies: &[f64],
    expected_events: u64,
) -> Result<(), String> {
    if rows.len() != energies.len() {
        return Err(format!(
            "{name} has {} rows; expected {} registered energies",
            rows.len(),
            energies.len()
        ));
    }
    let mut seen = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let number = |column: &str| -> Result<f64, String> {
            parse_nonnegative(
                row[header.iter().position(|h| *h == column).unwrap()],
                i,
                column,
            )
        };
        let energy = number("energy_eV")?;
        if !energies.contains(&energy) || seen.contains(&energy) {
            return Err(format!(
                "{name} row {} has unexpected or duplicate energy {energy}",
                i + 1
            ));
        }
        seen.push(energy);
        let n = number("n_events")?;
        if n != expected_events as f64 {
            return Err(format!(
                "{name} at {energy} eV has n_events={n}; expected {expected_events}"
            ));
        }
        let moments = [
            ("sum_L_nm", "sum_L2_nm2"),
            ("sum_edep_eV", "sum_edep2_eV2"),
            ("sum_remaining_eV", "sum_remaining2_eV2"),
            ("sum_nsec", "sum_nsec2"),
            ("sum_esec_eV", "sum_esec2_eV2"),
        ];
        for (sum_name, square_name) in moments {
            let sum = number(sum_name)?;
            let square = number(square_name)?;
            if square + 1e-10 * square.abs().max(1.0) < sum * sum / n {
                return Err(format!(
                    "{name} row {} has invalid count/moment pair {sum_name}/{square_name}: sum of squares violates Cauchy bound",
                    i + 1
                ));
            }
        }
        number("sum_range_nm")?;
        for col in ["n_elastic", "n_excitation", "n_ionisation", "n_other"] {
            let v = number(col)?;
            if v.fract() != 0.0 {
                return Err(format!(
                    "{name} row {} has non-integer count `{col}`",
                    i + 1
                ));
            }
        }
        if number("n_other")? != 0.0 {
            return Err(format!(
                "{name} row {} contains n_other interactions",
                i + 1
            ));
        }
        if name.starts_with("mfp_")
            && ["n_elastic", "n_excitation", "n_ionisation"]
                .iter()
                .map(|c| number(c))
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .sum::<f64>()
                != n
        {
            return Err(format!(
                "{name} row {} process counts do not sum to n_events",
                i + 1
            ));
        }
    }
    if seen != energies {
        return Err(format!(
            "{name} energy list differs from its registered order"
        ));
    }
    Ok(())
}

fn parse_nonnegative(raw: &str, row: usize, column: &str) -> Result<f64, String> {
    let value = raw
        .parse::<f64>()
        .map_err(|_| format!("row {} column `{column}` is not a number: `{raw}`", row + 2))?;
    if !value.is_finite() || value < 0.0 {
        return Err(format!(
            "row {} column `{column}` must be finite and nonnegative, got `{raw}`",
            row + 2
        ));
    }
    Ok(value)
}

fn col(header: &[String], name: &str) -> usize {
    header
        .iter()
        .position(|h| h == name)
        .unwrap_or_else(|| panic!("column `{name}` missing in {header:?}"))
}

/// Mean and standard error from `n`, `sum x`, `sum x^2` (unbiased variance).
#[derive(Debug, Clone, Copy)]
struct Sample {
    mean: f64,
    se: f64,
}

impl Sample {
    fn from_sums(n: f64, s: f64, s2: f64) -> Self {
        let mean = s / n;
        let var = ((s2 / n - mean * mean) * n / (n - 1.0)).max(0.0);
        Self {
            mean,
            se: (var / n).sqrt(),
        }
    }
    fn from_values(v: &[f64]) -> Self {
        let n = v.len() as f64;
        Self::from_sums(n, v.iter().sum(), v.iter().map(|x| x * x).sum())
    }
}

thread_local! {
    /// Margins of every comparison made on this thread: `(label, used)` with `used` the fraction
    /// of the tolerance consumed (1.0 = at the limit; for V5 the larger of `z / 3` and
    /// `rel / 2 %`). Printed by `print_margins` (visible with `--nocapture`).
    static MARGINS: std::cell::RefCell<Vec<(String, f64)>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn record_margin(label: &str, used: f64) {
    MARGINS.with(|m| m.borrow_mut().push((label.to_owned(), used)));
}

/// Print and clear the recorded margins: count, how many are within 50 % of the limit, and the
/// five tightest comparisons.
fn print_margins(title: &str) {
    let mut v = MARGINS.with(|m| std::mem::take(&mut *m.borrow_mut()));
    v.retain(|(_, u)| u.is_finite());
    v.sort_by(|a, b| b.1.total_cmp(&a.1));
    let over_half = v.iter().filter(|(_, u)| *u > 0.5).count();
    eprintln!(
        "[margins] {title}: {} comparisons, {over_half} above 50 % of tolerance; tightest:",
        v.len()
    );
    for (label, used) in v.iter().take(5) {
        eprintln!("[margins]   {:5.1} %  {label}", used * 100.0);
    }
}

/// V5: `|a-b| < 3 combined se` and (when `with_relative`) `|a-b|/|b| < 2 %`.
fn v5(label: &str, a: Sample, b: Sample, with_relative: bool) -> Result<(), String> {
    if !a.mean.is_finite()
        || !b.mean.is_finite()
        || !a.se.is_finite()
        || !b.se.is_finite()
        || a.se < 0.0
        || b.se < 0.0
    {
        return Err(format!("{label}: nonfinite mean or invalid standard error"));
    }
    let diff = (a.mean - b.mean).abs();
    let se = (a.se * a.se + b.se * b.se).sqrt();
    let z = if se > 0.0 {
        diff / se
    } else if diff == 0.0 {
        0.0
    } else {
        f64::INFINITY
    };
    let rel = diff / b.mean.abs();
    let used_rel = if with_relative && rel.is_finite() {
        rel / 0.02
    } else {
        0.0
    };
    record_margin(label, (z / 3.0).max(used_rel));
    if z >= 3.0 {
        return Err(format!(
            "{label}: rust {:.6e} +- {:.2e} vs geant4 {:.6e} +- {:.2e}: z = {z:.2} >= 3",
            a.mean, a.se, b.mean, b.se
        ));
    }
    if with_relative && rel >= 0.02 {
        return Err(format!(
            "{label}: rust {:.6e} vs geant4 {:.6e}: relative difference {:.2} % >= 2 %",
            a.mean,
            b.mean,
            rel * 100.0
        ));
    }
    Ok(())
}

fn table() -> (ProcessTable, DataDir) {
    let dir = DataDir::new(repo_data_dir()).expect("committed data present");
    (
        ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap(),
        dir,
    )
}

// ------------------------------------------------------------- reference rows ------------

/// One energy of an `mfp` or `track` CSV.
struct EventRow {
    energy: f64,
    n: f64,
    l: Sample,
    edep: Sample,
    rem: Sample,
    nsec: Sample,
    esec: Sample,
    counts: [f64; 3],
}

fn event_rows(header: &[String], rows: &[Vec<String>]) -> Vec<EventRow> {
    let f = |r: &Vec<String>, name: &str| -> f64 { r[col(header, name)].parse().unwrap() };
    rows.iter()
        .map(|r| {
            let n = f(r, "n_events");
            EventRow {
                energy: f(r, "energy_eV"),
                n,
                l: Sample::from_sums(n, f(r, "sum_L_nm"), f(r, "sum_L2_nm2")),
                edep: Sample::from_sums(n, f(r, "sum_edep_eV"), f(r, "sum_edep2_eV2")),
                rem: Sample::from_sums(n, f(r, "sum_remaining_eV"), f(r, "sum_remaining2_eV2")),
                nsec: Sample::from_sums(n, f(r, "sum_nsec"), f(r, "sum_nsec2")),
                esec: Sample::from_sums(n, f(r, "sum_esec_eV"), f(r, "sum_esec2_eV2")),
                counts: [
                    f(r, "n_elastic"),
                    f(r, "n_excitation"),
                    f(r, "n_ionisation"),
                ],
            }
        })
        .collect()
}

// ---------------------------------------------------------------- comparisons ------------

/// V2 and V3 on rows `(energy, process, sigma per volume per nm)`.
fn compare_cross_sections(table: &ProcessTable, rows: &[(f64, String, f64)]) -> Vec<String> {
    let mut failures = Vec::new();
    let mut by_energy: std::collections::BTreeMap<u64, [Option<f64>; ProcessKind::COUNT]> =
        Default::default();
    for (e, process, sigma) in rows {
        let p = match process.as_str() {
            "elastic" => ProcessKind::Elastic,
            "excitation" => ProcessKind::Excitation,
            "ionisation" => ProcessKind::Ionisation,
            other => panic!("unknown process `{other}`"),
        };
        by_energy.entry(e.to_bits()).or_default()[p.index()] = Some(*sigma);
    }
    for (bits, g4) in by_energy {
        let e = f64::from_bits(bits);
        let node = NODE_ENERGIES.contains(&e);
        let tol = if node { 1e-9 } else { 1e-6 };
        let rust = table
            .macroscopic(dna_core::units::Energy::from_ev(e))
            .unwrap();
        let mut all_open = true;
        let rust_total: f64 = M2_PROCESSES.iter().map(|&p| rust.get(p)).sum();
        for p in M2_PROCESSES {
            let g = g4[p.index()].expect("all three processes listed for every energy");
            let r = rust.get(p);
            if g == 0.0 || r == 0.0 {
                if g != r {
                    failures.push(format!(
                        "V2 {p:?} at {e} eV: rust {r:e} vs geant4 {g:e} (window)"
                    ));
                }
                all_open = false;
                continue;
            }
            let rel = (r - g).abs() / g;
            record_margin(&format!("V2 {p:?} at {e} eV"), rel / tol);
            if rel >= tol {
                failures.push(format!(
                    "V2 {p:?} at {e} eV: rust {r:e} vs geant4 {g:e}, rel {rel:e} >= {tol:e}"
                ));
            }
        }
        if all_open {
            let g_tot: f64 = M2_PROCESSES.iter().map(|&p| g4[p.index()].unwrap()).sum();
            for p in M2_PROCESSES {
                let pg = g4[p.index()].unwrap() / g_tot;
                let pr = rust.get(p) / rust_total;
                let rel = (pr - pg).abs() / pg;
                record_margin(&format!("V3 {p:?} at {e} eV"), rel / tol);
                if rel >= tol {
                    failures.push(format!(
                        "V3 {p:?} at {e} eV: rust {pr:e} vs geant4 {pg:e}, rel {rel:e} >= {tol:e}"
                    ));
                }
            }
        }
    }
    failures
}

/// V5 on first-interaction free path and process fractions.
fn compare_mfp(table: &ProcessTable, rows: &[EventRow], n_rust: u64, seed: u64) -> Vec<String> {
    let mut failures = Vec::new();
    for r in rows {
        let mut cfg = m2_config(r.energy, n_rust);
        cfg.master_seed = seed;
        cfg.max_interactions = Some(1);
        let rust = run_detailed(table, &cfg).unwrap();
        let lengths: Vec<f64> = rust
            .iter()
            .map(|h| h.outcome.first_step.expect("one step").length_nm)
            .collect();
        let a = Sample::from_values(&lengths);
        if let Err(m) = v5(&format!("mfp at {} eV", r.energy), a, r.l, true) {
            failures.push(m);
        }
        // fractions: binomial standard errors on both sides
        let total_g4: f64 = r.counts.iter().sum();
        assert!(
            rust.iter()
                .all(|h| M2_PROCESSES.contains(&h.outcome.first_step.unwrap().process)),
            "Sanche/Melton cannot act at the energies of the mfp reference"
        );
        for p in M2_PROCESSES {
            let fr = rust
                .iter()
                .filter(|h| h.outcome.first_step.unwrap().process == p)
                .count() as f64
                / n_rust as f64;
            let fg = r.counts[p.index()] / total_g4;
            let se = |f: f64, n: f64| (f * (1.0 - f) / n).sqrt();
            let a = Sample {
                mean: fr,
                se: se(fr, n_rust as f64),
            };
            let b = Sample {
                mean: fg,
                se: se(fg, total_g4),
            };
            let rare = (a.se / a.mean.max(1e-300)) > 0.02;
            if let Err(m) = v5(
                &format!("first-process fraction {p:?} at {} eV", r.energy),
                a,
                b,
                !rare,
            ) {
                failures.push(m);
            }
        }
    }
    failures
}

/// V5 on the primary track with a tracking cut.
fn compare_track(table: &ProcessTable, rows: &[EventRow], seed: u64) -> Vec<String> {
    let mut failures = Vec::new();
    for r in rows {
        let mut cfg = m2_config(r.energy, r.n as u64);
        cfg.master_seed = seed;
        cfg.tracking_cut_ev = Some(TRACK_CUT_EV);
        let rust = run_detailed(table, &cfg).unwrap();
        let s = |f: &dyn Fn(&dna_sim::history::HistoryResult) -> f64| -> Sample {
            Sample::from_values(&rust.iter().map(f).collect::<Vec<_>>())
        };
        let e = r.energy;
        let pairs = [
            ("track length (nm)", s(&|h| h.outcome.track_length_nm), r.l),
            (
                "deposited energy (eV)",
                s(&|h| h.outcome.ledger.deposited()),
                r.edep,
            ),
            (
                "remaining energy (eV)",
                s(&|h| h.outcome.ledger.remaining_untracked),
                r.rem,
            ),
            (
                "secondaries per primary",
                s(&|h| h.outcome.n_secondaries as f64),
                r.nsec,
            ),
            (
                "secondary energy (eV)",
                s(&|h| h.outcome.ledger.untracked_secondaries),
                r.esec,
            ),
        ];
        for (label, a, b) in pairs {
            if let Err(m) = v5(&format!("{label} at {e} eV"), a, b, true) {
                failures.push(m);
            }
        }
        // interaction counts per event: the Geant4 CSV holds totals only, so the standard error
        // of both means is taken from the Rust sample (equal-variance assumption, stated)
        for p in M2_PROCESSES {
            let a = s(&|h| h.outcome.counts.get(p) as f64);
            let b = Sample {
                mean: r.counts[p.index()] / r.n,
                se: a.se,
            };
            if let Err(m) = v5(
                &format!("{p:?} interactions per primary at {e} eV"),
                a,
                b,
                true,
            ) {
                failures.push(m);
            }
        }
    }
    failures
}

// ------------------------------------------------------------ the ignored tests ----------

#[test]
#[ignore = "needs validation/reference/xs_per_volume.csv from Geant4 (validation/geant4/README.md)"]
fn v2_v3_cross_sections_per_volume_match_geant4() {
    let (header, rows) = read_reference("xs_per_volume.csv");
    let (ie, ip, is) = (
        col(&header, "energy_eV"),
        col(&header, "process"),
        col(&header, "sigma_per_volume_per_nm"),
    );
    let parsed: Vec<(f64, String, f64)> = rows
        .iter()
        .map(|r| {
            (
                r[ie].parse().unwrap(),
                r[ip].clone(),
                r[is].parse().unwrap(),
            )
        })
        .collect();
    let (table, _) = table();
    let failures = compare_cross_sections(&table, &parsed);
    print_margins("V2/V3 cross sections");
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
#[ignore = "needs validation/reference/mfp_first_interaction.csv from Geant4 (validation/geant4/README.md)"]
fn v5_first_interaction_mean_free_path_matches_geant4() {
    let (header, rows) = read_reference("mfp_first_interaction.csv");
    let rows = event_rows(&header, &rows);
    let energies: Vec<f64> = rows.iter().map(|r| r.energy).collect();
    assert_eq!(
        energies, MFP_ENERGIES,
        "energy list differs from g4rs_ref.cc"
    );
    let (table, _) = table();
    let failures = compare_mfp(&table, &rows, 200_000, 20_261);
    print_margins("V5 mfp, seed 1");
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
#[ignore = "needs validation/reference/track_cut100eV.csv from Geant4 (validation/geant4/README.md)"]
fn v5_primary_track_with_100_ev_cut_matches_geant4() {
    let (header, rows) = read_reference("track_cut100eV.csv");
    let rows = event_rows(&header, &rows);
    let energies: Vec<f64> = rows.iter().map(|r| r.energy).collect();
    assert_eq!(
        energies, TRACK_ENERGIES,
        "energy list differs from g4rs_ref.cc"
    );
    let (table, _) = table();
    let failures = compare_track(&table, &rows, 20_262);
    print_margins("V5 track, seed 1");
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Seed-2 consistency: the second, independently seeded Geant4 runs must (a) agree with the
/// first Geant4 runs under the V5 criteria (the reference is reproducible and its statistical
/// errors are honest) and (b) agree with Rust run with a different seed than the seed-1 tests.
/// A criterion that passes for seed 1 but not for seed 2 would indicate a lucky draw, not
/// agreement.
#[test]
#[ignore = "needs validation/reference/*_seed2.csv from Geant4 (validation/geant4/README.md)"]
fn seed2_geant4_runs_are_consistent_with_seed1_and_with_rust() {
    let (table, _) = table();
    let mut failures = Vec::new();

    let load = |name: &str| {
        let (h, r) = read_reference(name);
        event_rows(&h, &r)
    };
    let mfp1 = load("mfp_first_interaction.csv");
    let mfp2 = load("mfp_first_interaction_seed2.csv");
    let trk1 = load("track_cut100eV.csv");
    let trk2 = load("track_cut100eV_seed2.csv");

    // (a) Geant4 seed 1 versus Geant4 seed 2, same criteria
    for (kind, r1, r2) in [("mfp", &mfp1, &mfp2), ("track", &trk1, &trk2)] {
        assert_eq!(r1.len(), r2.len(), "{kind}: different energy lists");
        for (a, b) in r1.iter().zip(r2) {
            assert_eq!(a.energy, b.energy, "{kind}: energy lists differ");
            let e = a.energy;
            let pairs = [
                ("length (nm)", a.l, b.l),
                ("deposited energy (eV)", a.edep, b.edep),
                ("remaining energy (eV)", a.rem, b.rem),
                ("secondaries per event", a.nsec, b.nsec),
                ("secondary energy (eV)", a.esec, b.esec),
            ];
            for (label, x, y) in pairs {
                if let Err(m) = v5(
                    &format!("G4 seed1 vs seed2, {kind} {label} at {e} eV"),
                    x,
                    y,
                    true,
                ) {
                    failures.push(m);
                }
            }
        }
    }
    print_margins("Geant4 seed 1 vs Geant4 seed 2");

    // (b) Rust (seeds distinct from the seed-1 tests) versus the seed-2 Geant4 runs
    failures.extend(compare_mfp(&table, &mfp2, 200_000, 20_263));
    print_margins("V5 mfp, Rust vs Geant4 seed 2");
    failures.extend(compare_track(&table, &trk2, 20_264));
    print_margins("V5 track, Rust vs Geant4 seed 2");

    assert!(
        failures.is_empty(),
        "{} failures:
{}",
        failures.len(),
        failures.join(
            "
"
        )
    );
}

// ------------------------------------------------------- machinery self-tests ------------

#[test]
fn csv_parser_skips_comments_and_blank_lines() {
    let (h, rows) = parse_csv("# a comment\nx,y\n\n1,2\n# mid\n3,4\n");
    assert_eq!(h, ["x", "y"]);
    assert_eq!(rows, [["1", "2"], ["3", "4"]]);
    assert_eq!(col(&h, "y"), 1);
}

#[test]
fn sample_from_sums_matches_direct_computation() {
    let v = [1.0, 2.0, 4.0, 8.0, 16.0];
    let a = Sample::from_values(&v);
    assert!((a.mean - 6.2).abs() < 1e-12);
    // sample variance 37.2 -> se = sqrt(37.2 / 5)
    assert!((a.se - (37.2f64 / 5.0).sqrt()).abs() < 1e-12);
}

#[test]
fn v5_needs_both_the_z_test_and_the_relative_tolerance() {
    let s = |mean, se| Sample { mean, se };
    // 1 % apart, tiny errors: fails z; 1 % apart with big errors: passes both
    assert!(v5("t", s(1.01, 0.001), s(1.0, 0.001), true).is_err());
    assert!(v5("t", s(1.01, 0.02), s(1.0, 0.02), true).is_ok());
    // 3 % apart with huge errors: passes z, fails the 2 % systematic cut
    assert!(v5("t", s(1.03, 0.5), s(1.0, 0.5), true).is_err());
    // ... unless the relative cut is switched off for noisy rare-process fractions
    assert!(v5("t", s(1.03, 0.5), s(1.0, 0.5), false).is_ok());
    // identical means always pass
    assert!(v5("t", s(2.0, 0.1), s(2.0, 0.1), true).is_ok());
}

/// Build a reference CSV row (the same sums `g4rs_ref` writes) from a Rust run: the comparison
/// code path is then run Rust-versus-Rust with independent seeds, which must pass: it checks the
/// column mapping and the false-positive behaviour of the criteria. **Not a validation.**
fn csv_from_rust_run(
    table: &ProcessTable,
    energy: f64,
    n: u64,
    seed: u64,
    cut: Option<f64>,
    one_step: bool,
) -> String {
    let mut cfg = m2_config(energy, n);
    cfg.master_seed = seed;
    cfg.tracking_cut_ev = cut;
    cfg.max_interactions = one_step.then_some(1);
    let rs = run_detailed(table, &cfg).unwrap();
    let sum = |f: &dyn Fn(&dna_sim::history::HistoryResult) -> f64| rs.iter().map(f).sum::<f64>();
    let sum2 = |f: &dyn Fn(&dna_sim::history::HistoryResult) -> f64| {
        rs.iter().map(|h| f(h) * f(h)).sum::<f64>()
    };
    let l = |h: &dna_sim::history::HistoryResult| {
        if one_step {
            h.outcome.first_step.unwrap().length_nm
        } else {
            h.outcome.track_length_nm
        }
    };
    let edep = |h: &dna_sim::history::HistoryResult| h.outcome.ledger.deposited();
    let rem = |h: &dna_sim::history::HistoryResult| h.outcome.ledger.remaining_untracked;
    let nsec = |h: &dna_sim::history::HistoryResult| h.outcome.n_secondaries as f64;
    let esec = |h: &dna_sim::history::HistoryResult| h.outcome.ledger.untracked_secondaries;
    let count = |p: ProcessKind| -> u64 { rs.iter().map(|h| h.outcome.counts.get(p)).sum() };
    format!(
        "# synthetic self-test\nenergy_eV,n_events,sum_L_nm,sum_L2_nm2,sum_edep_eV,sum_edep2_eV2,sum_remaining_eV,sum_remaining2_eV2,sum_nsec,sum_nsec2,sum_esec_eV,sum_esec2_eV2,sum_range_nm,n_elastic,n_excitation,n_ionisation,n_other\n{energy},{n},{},{},{},{},{},{},{},{},{},{},0,{},{},{},0\n",
        sum(&l),
        sum2(&l),
        sum(&edep),
        sum2(&edep),
        sum(&rem),
        sum2(&rem),
        sum(&nsec),
        sum2(&nsec),
        sum(&esec),
        sum2(&esec),
        count(ProcessKind::Elastic),
        count(ProcessKind::Excitation),
        count(ProcessKind::Ionisation)
    )
}

#[test]
fn comparison_machinery_passes_rust_against_rust_with_other_seeds() {
    let root = repo_data_dir();
    if !root.join("dna").is_dir() {
        eprintln!("SKIPPED: Geant4 data not found");
        return;
    }
    let (table, _) = table();
    // mfp: "reference" = 100k first steps with seed A, compared with 200k with seed B
    let text = csv_from_rust_run(&table, 1.0e3, 100_000, 111, None, true);
    let (h, r) = parse_csv(&text);
    let rows = event_rows(&h, &r);
    let failures = compare_mfp(&table, &rows, 200_000, 222);
    assert!(failures.is_empty(), "{failures:?}");
    // track: 300 tracks of 1 keV with a 100 eV cut
    let text = csv_from_rust_run(&table, 1.0e3, 300, 333, Some(TRACK_CUT_EV), false);
    let (h, r) = parse_csv(&text);
    let rows = event_rows(&h, &r);
    let failures = compare_track(&table, &rows, 444);
    // 300-event samples carry ~3 % noise on some means, so only the z-criterion is meaningful
    // here: the relative 2 % cut can legitimately trip. Everything reported must be that cut.
    assert!(
        failures.iter().all(|m| m.contains("relative difference")),
        "unexpected z failure(s): {failures:?}"
    );
}

#[test]
fn cross_section_comparison_detects_a_perturbed_reference() {
    let root = repo_data_dir();
    if !root.join("dna").is_dir() {
        eprintln!("SKIPPED: Geant4 data not found");
        return;
    }
    let (table, _) = table();
    let make = |scale: f64| -> Vec<(f64, String, f64)> {
        let mut v = Vec::new();
        for e in [100.0, 1234.5] {
            let xs = table
                .macroscopic(dna_core::units::Energy::from_ev(e))
                .unwrap();
            for (p, name) in [
                (ProcessKind::Elastic, "elastic"),
                (ProcessKind::Excitation, "excitation"),
                (ProcessKind::Ionisation, "ionisation"),
            ] {
                let s = if p == ProcessKind::Ionisation {
                    scale
                } else {
                    1.0
                };
                v.push((e, name.to_owned(), xs.get(p) * s));
            }
        }
        v
    };
    assert!(compare_cross_sections(&table, &make(1.0)).is_empty());
    // a 1e-7 error passes off-node (1e-6) but fails at the node 100 eV (1e-9)
    let f = compare_cross_sections(&table, &make(1.0 + 1e-7));
    assert!(f.iter().any(|m| m.contains("at 100 eV")), "{f:?}");
    assert!(
        !f.iter()
            .any(|m| m.contains("at 1234.5 eV") && m.contains("V2")),
        "{f:?}"
    );
    // a 1e-5 error fails everywhere
    assert!(compare_cross_sections(&table, &make(1.0 + 1e-5)).len() >= 2);
}

#[test]
fn m2_reference_validator_accepts_a_complete_in_memory_mfp_fixture() {
    let csv = mfp_fixture();
    assert_eq!(
        validate_m2_reference("mfp_first_interaction.csv", &csv, VALID_PROVENANCE),
        Ok(())
    );
}

#[test]
fn m2_reference_validator_rejects_malformed_evidence() {
    let valid = mfp_fixture();
    let cases = [
        ("empty file", "".to_owned()),
        ("missing column", valid.replacen("sum_L_nm,", "", 1)),
        (
            "malformed row",
            valid.replacen("150,1000000,", "150,ten,", 1),
        ),
        (
            "duplicate row",
            valid.clone() + valid.lines().last().unwrap() + "\n",
        ),
        (
            "missing energy",
            valid.replace("150,1000000,", "151,1000000,"),
        ),
        ("nonfinite value", valid.replacen(",20,", ",NaN,", 1)),
        ("negative value", valid.replacen(",20,", ",-1,", 1)),
        ("invalid moments", valid.replacen(",20,40,", ",20,0,", 1)),
        ("wrong mode", valid.replacen("mode mfp", "mode track", 1)),
        (
            "wrong constructor",
            valid.replacen("G4EmDNAPhysics_option2", "G4EmDNAPhysics_option4", 1),
        ),
        ("wrong seed", valid.replacen("seed 20261", "seed 7", 1)),
        ("wrong units", valid.replacen("sum_L_nm", "sum_L_um", 1)),
        ("wrong provenance constructor", valid.clone()),
        ("wrong provenance", valid.clone()),
    ];
    for (label, text) in cases {
        let provenance = match label {
            "wrong provenance" => VALID_PROVENANCE.replace("11.4.1", "11.3.0"),
            "wrong provenance constructor" => {
                VALID_PROVENANCE.replace("G4EmDNAPhysics_option2", "G4EmDNAPhysics_option4")
            }
            _ => VALID_PROVENANCE.to_owned(),
        };
        assert!(
            validate_m2_reference("mfp_first_interaction.csv", &text, &provenance).is_err(),
            "accepted {label}"
        );
    }
}

#[test]
fn m2_cross_section_validator_requires_exact_grid_and_process_coverage() {
    let valid = xs_fixture();
    assert_eq!(
        validate_m2_reference("xs_per_volume.csv", &valid, XS_PROVENANCE),
        Ok(())
    );
    let unknown_process = valid.replacen(",elastic,", ",unknown,", 1);
    assert!(validate_m2_reference("xs_per_volume.csv", &unknown_process, XS_PROVENANCE).is_err());
    let missing_row = valid
        .lines()
        .take(valid.lines().count() - 1)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(validate_m2_reference("xs_per_volume.csv", &missing_row, XS_PROVENANCE).is_err());
}

#[test]
fn m2_cross_section_grid_accepts_one_ulp_rounding_of_a_registered_energy() {
    let mut lines: Vec<String> = xs_fixture().lines().map(str::to_owned).collect();
    let mut first: Vec<String> = lines[4].split(',').map(str::to_owned).collect();
    let energy: f64 = first[0].parse().unwrap();
    first[0] = f64::from_bits(energy.to_bits() + 1).to_string();
    lines[4] = first.join(",");
    assert_eq!(
        validate_m2_reference("xs_per_volume.csv", &lines.join("\n"), XS_PROVENANCE),
        Ok(())
    );
}

#[test]
fn m2_cross_section_grid_rejects_conflicting_repeated_endpoint_values() {
    let mut lines: Vec<String> = xs_fixture().lines().map(str::to_owned).collect();
    let repeated: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let fields: Vec<&str> = line.split(',').collect();
            (fields.len() == 3
                && fields[0].parse::<f64>().ok() == Some(7.4)
                && fields[1] == "elastic")
                .then_some(index)
        })
        .collect();
    assert_eq!(repeated.len(), 2);
    let mut fields: Vec<&str> = lines[repeated[1]].split(',').collect();
    fields[2] = "1";
    lines[repeated[1]] = fields.join(",");
    assert!(validate_m2_reference("xs_per_volume.csv", &lines.join("\n"), XS_PROVENANCE).is_err());
}

#[test]
fn m2_deterministic_unequal_means_fail_the_v5_check() {
    let exact = |mean| Sample { mean, se: 0.0 };
    assert!(v5("equal", exact(2.0), exact(2.0), false).is_ok());
    assert!(v5("unequal", exact(2.0), exact(3.0), false).is_err());
}

const VALID_PROVENANCE: &str = r#"
geant4_version = "11.4.1"
geant4_commit = "test-revision"
g4emlow_version = "8.8"
g4emlow_md5 = "328330009df633f7e9b3a9f445745298"
physics_constructor = "G4EmDNAPhysics_option2"
generator = "g4rs_ref"
date = "2026-10-26"
command_lines = ["g4rs_ref mfp mfp_first_interaction.csv 1000000 20261"]
fluo_auger = "off"
compiler = "test compiler"
platform = "test platform"
"#;

const XS_PROVENANCE: &str = r#"
geant4_version = "11.4.1"
geant4_commit = "test-revision"
g4emlow_version = "8.8"
g4emlow_md5 = "328330009df633f7e9b3a9f445745298"
physics_constructor = "G4EmDNAPhysics_option2"
generator = "g4rs_ref"
date = "2026-10-26"
command_lines = ["g4rs_ref xs xs_per_volume.csv"]
fluo_auger = "off"
compiler = "test compiler"
platform = "test platform"
"#;

fn mfp_fixture() -> String {
    let mut csv = String::from(
        "# generator: g4rs_ref (validation/geant4/g4rs_ref), mode mfp\n\
         # geant4_version_number: 1141\n\
         # physics: G4EmDNAPhysics_option2; material G4_WATER; fluo/auger OFF; events 1000000; seed 20261\n\
         energy_eV,n_events,sum_L_nm,sum_L2_nm2,sum_edep_eV,sum_edep2_eV2,sum_remaining_eV,sum_remaining2_eV2,sum_nsec,sum_nsec2,sum_esec_eV,sum_esec2_eV2,sum_range_nm,n_elastic,n_excitation,n_ionisation,n_other\n",
    );
    for energy in MFP_ENERGIES {
        csv.push_str(&format!(
            "{energy},1000000,20,40,0,0,0,0,0,0,0,0,20,300000,200000,500000,0\n"
        ));
    }
    csv
}

fn xs_fixture() -> String {
    let mut csv = String::from(
        "# generator: g4rs_ref (validation/geant4/g4rs_ref), mode xs\n\
         # geant4_version_number: 1141\n\
         # physics: G4EmDNAPhysics_option2; material G4_WATER; fluo/auger OFF; events 0; seed 0\n\
         energy_eV,process,sigma_per_volume_per_nm\n",
    );
    let mut energies = vec![7.4, 9.0, 11.0, 100.0, 1e3, 1e4, 1e5, 1e6];
    energies.extend((0..200).map(|i| 7.4 * (1e6_f64 / 7.4).powf(i as f64 / 199.0)));
    for energy in energies {
        for process in M2_PROCESS_NAMES {
            csv.push_str(&format!("{energy:.17},{process},0\n"));
        }
    }
    csv
}
