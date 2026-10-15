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
//! The non-ignored tests exercise the comparison machinery itself on synthetic numbers and on a
//! Rust-versus-Rust run with different seeds (a false-positive check, **not** a validation).

use dna_data::datadir::{DataDir, repo_data_dir};
use dna_physics::models::{ElasticRotation, ProcessKind};
use dna_physics::table::ProcessTable;
use dna_sim::config::RunConfig;
use dna_sim::run::run_detailed;
use std::path::PathBuf;

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
    check_provenance();
    parse_csv(&text)
}

/// `provenance.toml` must exist and name the Geant4 version, dataset and physics constructor.
fn check_provenance() {
    let path = reference_dir().join("provenance.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing {}: {e}", path.display()));
    let t: toml::Table = text.parse().expect("provenance.toml is not valid TOML");
    for key in [
        "geant4_version",
        "g4emlow_version",
        "physics_constructor",
        "generator",
        "date",
        "command_lines",
    ] {
        assert!(t.contains_key(key), "provenance.toml lacks `{key}`");
    }
    assert!(
        t["geant4_version"]
            .as_str()
            .is_some_and(|v| v.starts_with("11.4.1") || v.contains("11.4.1")),
        "reference must come from Geant4 11.4.1"
    );
    assert_eq!(
        t["physics_constructor"].as_str(),
        Some("G4EmDNAPhysics_option2")
    );
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
    let diff = (a.mean - b.mean).abs();
    let se = (a.se * a.se + b.se * b.se).sqrt();
    let z = if se > 0.0 { diff / se } else { 0.0 };
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
    let mut by_energy: std::collections::BTreeMap<u64, [Option<f64>; 3]> = Default::default();
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
        for p in ProcessKind::ALL {
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
            let g_tot: f64 = g4.iter().map(|x| x.unwrap()).sum();
            for p in ProcessKind::ALL {
                let pg = g4[p.index()].unwrap() / g_tot;
                let pr = rust.get(p) / rust.total();
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
        let mut cfg = RunConfig::new(r.energy, n_rust);
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
        for p in ProcessKind::ALL {
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
        let mut cfg = RunConfig::new(r.energy, r.n as u64);
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
                s(&|h| h.outcome.ledger.secondaries),
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
        for p in ProcessKind::ALL {
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
    let mut cfg = RunConfig::new(energy, n);
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
    let esec = |h: &dna_sim::history::HistoryResult| h.outcome.ledger.secondaries;
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
