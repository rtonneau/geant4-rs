//! L3 comparison of the full slowing-down (milestone M3) with Geant4 11.4.1 `G4EmDNAPhysics_option2`.
//!
//! The `#[ignore]` tests need files produced by `validation/geant4/g4rs_ref` (modes `xslow` and
//! `slow`, instructions in `validation/geant4/README.md`) and committed to
//! `validation/reference/` together with `provenance_m3.toml`:
//! `xs_lowenergy.csv`, `slow_sums.csv`, `slow_spectra.csv`, `slow_radial.csv`. No Geant4 number is
//! invented or embedded here.
//!
//! **Pre-registered acceptance (fixed before any comparison):**
//! * V2 (`xs_lowenergy.csv`, cross sections per volume of all five processes): relative
//!   difference < 1e-9 at the node energies `NODE_ENERGIES` (window boundaries, the Melton table
//!   end, the Sanche edge), < 1e-6 elsewhere; a cross section that is zero on one side must be
//!   zero on the other (window agreement).
//! * V5 (means of per-event quantities of the full cascade: primary track length, total track
//!   length of all tracks, deposited energy, interactions per process and solvations, secondaries,
//!   primary range): `|a - b| < Z_CRIT sqrt(se_a^2 + se_b^2)` with `Z_CRIT = 4` (docs/07 V5 states
//!   3 combined standard errors for a single quantity; this family has 11 quantities x 7 energies
//!   = 77 comparisons, for which 3 would give a family-wise false-alarm rate of about 19 %, and
//!   4 gives 0.5 %; the choice was made after a Rust-versus-Rust self-test of the machinery showed
//!   a z = 3.3 outlier by chance, **before** any Geant4 number of this milestone was seen), and,
//!   **only when the combined
//!   standard error is below 0.7 % of the Geant4 mean** (so that the test has the power to see
//!   2 %), also `|a - b| / |b| < 2 %`. Where the reference has too few events for the 2 % cut to
//!   be meaningful (high-energy tracks, tens of events) only the z-criterion applies; the failure
//!   messages say which criterion was used.
//! * Secondary-energy spectra: two-sample chi-square (homogeneity) per primary energy and
//!   generation class on the 70 log bins plus under/overflow, bins merged until the pooled count
//!   is >= 10; Bonferroni level `alpha = 1e-3 / 28` (7 energies x 4 classes). Counts of secondaries
//!   within an event are mildly correlated, which makes this test liberal, not conservative: a
//!   failure is a real signal, a pass is weaker evidence than the p-value suggests.
//! * Radial profile: cumulative fraction of the deposited energy inside 1, 10, 100, 1000, ... nm
//!   (up to the range): z < `Z_CRIT` with the per-event variance estimated on the Rust side for both
//!   (the reference CSV holds sums only; stated assumption).
//!
//! The non-ignored tests exercise the comparison machinery on synthetic CSVs written from a Rust
//! run (Rust-versus-Rust with different seeds: a false-positive check, **not** a validation).

use dna_data::datadir::{DataDir, repo_data_dir};
use dna_physics::models::{ElasticRotation, ProcessKind};
use dna_physics::table::ProcessTable;
use dna_sim::config::RunConfig;
use dna_sim::history::{HistoryResult, RADIAL_BINS, SPECTRUM_BINS, radial_edges_nm};
use dna_sim::run::run_detailed;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Node energies of `xs_lowenergy.csv` (window boundaries etc.; `RunXsLow` in `g4rs_ref.cc`).
const NODE_ENERGIES: [f64; 8] = [2.0, 4.0, 7.4, 9.0, 11.0, 12.3, 13.0, 100.0];
const GENERATION_CLASSES: usize = 4;
/// Combined-standard-error criterion of the V5 family (see the file header).
const Z_CRIT: f64 = 4.0;
const QUANTITIES: [&str; 11] = [
    "Lprim_nm",
    "Lall_nm",
    "edep_eV",
    "n_elastic",
    "n_excitation",
    "n_ionisation",
    "n_vibexc",
    "n_attachment",
    "n_solvation",
    "n_secondaries",
    "range_nm",
];

// ------------------------------------------------------------------ machinery ------------

fn reference_dir() -> PathBuf {
    repo_data_dir().join("../validation/reference")
}

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
             validation/geant4/README.md and commit it with provenance_m3.toml",
            path.display()
        )
    });
    check_provenance();
    parse_csv(&text)
}

fn check_provenance() {
    let path = reference_dir().join("provenance_m3.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing {}: {e}", path.display()));
    let t: toml::Table = text.parse().expect("provenance_m3.toml is not valid TOML");
    for key in [
        "geant4_version",
        "g4emlow_version",
        "physics_constructor",
        "generator",
        "date",
        "command_lines",
        "fluo_auger",
    ] {
        assert!(t.contains_key(key), "provenance_m3.toml lacks `{key}`");
    }
    assert!(
        t["geant4_version"]
            .as_str()
            .is_some_and(|v| v.contains("11.4.1")),
        "reference must come from Geant4 11.4.1"
    );
    assert_eq!(
        t["physics_constructor"].as_str(),
        Some("G4EmDNAPhysics_option2")
    );
    assert_eq!(t["fluo_auger"].as_str(), Some("off"));
}

fn col(header: &[String], name: &str) -> usize {
    header
        .iter()
        .position(|h| h == name)
        .unwrap_or_else(|| panic!("column `{name}` missing in {header:?}"))
}

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

/// V5 with the M3 rule for the relative cut (see the file header).
fn v5(label: &str, a: Sample, b: Sample) -> Result<(), String> {
    let diff = (a.mean - b.mean).abs();
    let se = (a.se * a.se + b.se * b.se).sqrt();
    if se == 0.0 {
        // deterministic quantity (e.g. the deposited energy of a full cascade)
        return if diff <= 1e-9 * b.mean.abs().max(1e-300) {
            Ok(())
        } else {
            Err(format!(
                "{label}: rust {:.9e} vs geant4 {:.9e} differ although both are deterministic",
                a.mean, b.mean
            ))
        };
    }
    let z = diff / se;
    if z >= Z_CRIT {
        return Err(format!(
            "{label}: rust {:.6e} +- {:.2e} vs geant4 {:.6e} +- {:.2e}: z = {z:.2} >= {Z_CRIT}",
            a.mean, a.se, b.mean, b.se
        ));
    }
    let relative_cut_meaningful = se < 0.007 * b.mean.abs();
    if relative_cut_meaningful {
        let rel = diff / b.mean.abs();
        if rel >= 0.02 {
            return Err(format!(
                "{label}: rust {:.6e} vs geant4 {:.6e}: relative difference {:.2} % >= 2 % \
                 (combined se {:.2} %)",
                a.mean,
                b.mean,
                rel * 100.0,
                se / b.mean.abs() * 100.0
            ));
        }
    }
    Ok(())
}

/// `1 - Q(a, x)` helper: regularised upper incomplete gamma `Q(a, x)` (Numerical Recipes).
fn gamma_q(a: f64, x: f64) -> f64 {
    fn ln_gamma(x: f64) -> f64 {
        const C: [f64; 6] = [
            76.18009172947146,
            -86.50532032941677,
            24.01409824083091,
            -1.231739572450155,
            0.1208650973866179e-2,
            -0.5395239384953e-5,
        ];
        let mut y = x;
        let tmp = x + 5.5 - (x + 0.5) * (x + 5.5).ln();
        let mut ser = 1.000000000190015;
        for c in C {
            y += 1.0;
            ser += c / y;
        }
        -tmp + (2.5066282746310005 * ser / x).ln()
    }
    if x <= 0.0 {
        return 1.0;
    }
    if x < a + 1.0 {
        let (mut ap, mut sum, mut del) = (a, 1.0 / a, 1.0 / a);
        for _ in 0..1000 {
            ap += 1.0;
            del *= x / ap;
            sum += del;
            if del.abs() < sum.abs() * 1e-16 {
                break;
            }
        }
        1.0 - sum * (-x + a * x.ln() - ln_gamma(a)).exp()
    } else {
        let tiny = 1e-300;
        let mut b = x + 1.0 - a;
        let mut c = 1.0 / tiny;
        let mut d = 1.0 / b;
        let mut h = d;
        for i in 1..1000 {
            let an = -(i as f64) * (i as f64 - a);
            b += 2.0;
            d = an * d + b;
            if d.abs() < tiny {
                d = tiny;
            }
            c = b + an / c;
            if c.abs() < tiny {
                c = tiny;
            }
            d = 1.0 / d;
            let del = d * c;
            h *= del;
            if (del - 1.0).abs() < 1e-16 {
                break;
            }
        }
        (-x + a * x.ln() - ln_gamma(a)).exp() * h
    }
}

/// Two-sample chi-square (homogeneity) p-value of two histograms; bins are merged until the pooled
/// count is >= 10. `None` if either histogram is empty or fewer than two bins remain.
fn homogeneity_p(a: &[f64], b: &[f64]) -> Option<(f64, usize, f64)> {
    let (na, nb): (f64, f64) = (a.iter().sum(), b.iter().sum());
    if na == 0.0 || nb == 0.0 {
        return None;
    }
    let (ka, kb) = ((nb / na).sqrt(), (na / nb).sqrt());
    let mut bins: Vec<(f64, f64)> = Vec::new();
    let (mut acc_a, mut acc_b) = (0.0, 0.0);
    for (&x, &y) in a.iter().zip(b) {
        acc_a += x;
        acc_b += y;
        if acc_a + acc_b >= 10.0 {
            bins.push((acc_a, acc_b));
            acc_a = 0.0;
            acc_b = 0.0;
        }
    }
    if acc_a + acc_b > 0.0 {
        match bins.last_mut() {
            Some(l) => {
                l.0 += acc_a;
                l.1 += acc_b;
            }
            None => bins.push((acc_a, acc_b)),
        }
    }
    if bins.len() < 2 {
        return None;
    }
    let chi2: f64 = bins
        .iter()
        .map(|&(x, y)| (ka * x - kb * y).powi(2) / (x + y))
        .sum();
    let dof = bins.len() - 1;
    Some((chi2, dof, gamma_q(dof as f64 / 2.0, chi2 / 2.0)))
}

// ------------------------------------------------------------- reference rows ------------

/// One primary energy of the `slow` reference.
struct SlowRow {
    energy: f64,
    n: f64,
    q: BTreeMap<&'static str, Sample>,
    /// `[class][bin]`, 72 entries each (underflow, 70 bins, overflow).
    spectra: Vec<Vec<f64>>,
    /// 42 entries (underflow, 40 shells, overflow): energy summed over events, eV.
    radial: Vec<f64>,
}

fn slow_rows(
    sums: &(Vec<String>, Vec<Vec<String>>),
    spectra: &(Vec<String>, Vec<Vec<String>>),
    radial: &(Vec<String>, Vec<Vec<String>>),
) -> Vec<SlowRow> {
    let (h, rows) = sums;
    let f = |r: &Vec<String>, name: &str| -> f64 { r[col(h, name)].parse().unwrap() };
    rows.iter()
        .map(|r| {
            let n = f(r, "n_events");
            let energy = f(r, "energy_eV");
            assert_eq!(f(r, "n_other_process"), 0.0, "{energy} eV: other processes");
            assert_eq!(
                f(r, "n_other_secondary"),
                0.0,
                "{energy} eV: non-electron secondaries"
            );
            let q = QUANTITIES
                .iter()
                .map(|&name| {
                    (
                        name,
                        Sample::from_sums(
                            n,
                            f(r, &format!("sum_{name}")),
                            f(r, &format!("sum2_{name}")),
                        ),
                    )
                })
                .collect();
            let mut sp = vec![vec![0.0; SPECTRUM_BINS + 2]; GENERATION_CLASSES];
            let (hs, rs) = spectra;
            for s in rs {
                if s[col(hs, "energy_eV")].parse::<f64>().unwrap() == energy {
                    let class: usize = s[col(hs, "generation_class")].parse().unwrap();
                    let bin: usize = s[col(hs, "bin")].parse().unwrap();
                    sp[class - 1][bin] = s[col(hs, "count")].parse().unwrap();
                }
            }
            let mut rad = vec![0.0; RADIAL_BINS + 2];
            let (hr, rr) = radial;
            for s in rr {
                if s[col(hr, "energy_eV")].parse::<f64>().unwrap() == energy {
                    let bin: usize = s[col(hr, "bin")].parse().unwrap();
                    rad[bin] = s[col(hr, "energy_eV_sum")].parse().unwrap();
                }
            }
            SlowRow {
                energy,
                n,
                q,
                spectra: sp,
                radial: rad,
            }
        })
        .collect()
}

fn table() -> ProcessTable {
    let dir = DataDir::new(repo_data_dir()).expect("committed data present");
    ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap()
}

/// Rust histories per Geant4 energy (cost-limited at high energy).
fn rust_histories(e: f64) -> u64 {
    match e {
        e if e <= 100.0 => 200_000,
        e if e <= 1.0e3 => 20_000,
        e if e <= 1.0e4 => 4_000,
        e if e <= 1.0e5 => 400,
        _ => 40,
    }
}

fn per_event(h: &HistoryResult, name: &str) -> f64 {
    let o = &h.outcome;
    match name {
        "Lprim_nm" => o.track_length_nm,
        "Lall_nm" => o.total_track_length_nm,
        "edep_eV" => o.ledger.deposited(),
        "n_elastic" => o.counts.get(ProcessKind::Elastic) as f64,
        "n_excitation" => o.counts.get(ProcessKind::Excitation) as f64,
        "n_ionisation" => o.counts.get(ProcessKind::Ionisation) as f64,
        "n_vibexc" => o.counts.get(ProcessKind::VibrationalExcitation) as f64,
        "n_attachment" => o.counts.get(ProcessKind::Attachment) as f64,
        "n_solvation" => o.solvations as f64,
        "n_secondaries" => o.n_secondaries as f64,
        "range_nm" => o.range_nm(),
        other => panic!("unknown quantity {other}"),
    }
}

// ---------------------------------------------------------------- comparisons ------------

fn compare_slow(
    table: &ProcessTable,
    rows: &[SlowRow],
    seed: u64,
    histories: &dyn Fn(f64) -> u64,
) -> Vec<String> {
    let mut failures = Vec::new();
    let edges = radial_edges_nm();
    for r in rows {
        let mut cfg = RunConfig::new(r.energy, histories(r.energy));
        cfg.master_seed = seed;
        let rust = run_detailed(table, &cfg).unwrap();
        let e = r.energy;
        for name in QUANTITIES {
            let vals: Vec<f64> = rust.iter().map(|h| per_event(h, name)).collect();
            if let Err(m) = v5(
                &format!("{name} at {e} eV"),
                Sample::from_values(&vals),
                r.q[name],
            ) {
                failures.push(m);
            }
        }
        // spectra by generation class
        let alpha = 1e-3 / 28.0;
        for class in 0..GENERATION_CLASSES {
            let mut rust_hist = vec![0.0; SPECTRUM_BINS + 2];
            for h in &rust {
                let sp = &h.spectra[class];
                rust_hist[0] += f64::from(sp.underflow);
                for (i, &c) in sp.counts.iter().enumerate() {
                    rust_hist[i + 1] += f64::from(c);
                }
                rust_hist[SPECTRUM_BINS + 1] += f64::from(sp.overflow);
            }
            if let Some((chi2, dof, p)) = homogeneity_p(&rust_hist, &r.spectra[class]) {
                eprintln!(
                    "[spectra] {e} eV, generation class {}: chi2 = {chi2:.1}/{dof}, p = {p:.3e}",
                    class + 1
                );
                if p < alpha {
                    failures.push(format!(
                        "secondary spectrum at {e} eV, class {}: chi2 = {chi2:.1}/{dof}, p = {p:.2e} < {alpha:.1e}",
                        class + 1
                    ));
                }
            }
        }
        // cumulative radial deposit fraction at the decade radii up to the primary's range
        let g4_total: f64 = r.radial.iter().sum();
        let ratio_n = rust.len() as f64 / r.n;
        for k in (0..=RADIAL_BINS).step_by(5) {
            let radius = edges[k];
            if radius > 10.0 * r.q["range_nm"].mean {
                break;
            }
            // bins 0..=k (underflow plus the shells below `radius`) hold r < edges[k]
            let g4_cum: f64 = r.radial[..=k].iter().sum::<f64>() / g4_total;
            let per_hist: Vec<f64> = rust
                .iter()
                .map(|h| (h.radial.underflow_ev + h.radial.energy_ev[..k].iter().sum::<f64>()) / e)
                .collect();
            let a = Sample::from_values(&per_hist);
            let b = Sample {
                mean: g4_cum,
                se: a.se * ratio_n.sqrt(),
            };
            let z = (a.mean - b.mean).abs() / (a.se * a.se + b.se * b.se).sqrt().max(1e-300);
            if z >= Z_CRIT && a.se > 0.0 {
                failures.push(format!(
                    "radial deposit fraction inside {radius:.3e} nm at {e} eV: rust {:.5} vs geant4 {:.5}, z = {z:.2}",
                    a.mean, b.mean
                ));
            }
        }
    }
    failures
}

/// V2 on rows `(energy, process, sigma per volume per nm)` of all five processes.
fn compare_low_energy_cross_sections(
    table: &ProcessTable,
    rows: &[(f64, String, f64)],
) -> Vec<String> {
    let mut failures = Vec::new();
    for (e, process, sigma) in rows {
        let p = match process.as_str() {
            "elastic" => ProcessKind::Elastic,
            "excitation" => ProcessKind::Excitation,
            "ionisation" => ProcessKind::Ionisation,
            "vibexc" => ProcessKind::VibrationalExcitation,
            "attachment" => ProcessKind::Attachment,
            other => panic!("unknown process `{other}`"),
        };
        let tol = if NODE_ENERGIES.contains(e) {
            1e-9
        } else {
            1e-6
        };
        let r = table
            .macroscopic(dna_core::units::Energy::from_ev(*e))
            .unwrap()
            .get(p);
        let g = *sigma;
        if g == 0.0 || r == 0.0 {
            if g != r {
                failures.push(format!(
                    "V2 {p:?} at {e} eV: rust {r:e} vs geant4 {g:e} (window)"
                ));
            }
            continue;
        }
        let rel = (r - g).abs() / g;
        if rel >= tol {
            failures.push(format!(
                "V2 {p:?} at {e} eV: rust {r:e} vs geant4 {g:e}, rel {rel:e} >= {tol:e}"
            ));
        }
    }
    failures
}

// ------------------------------------------------------------ the ignored tests ----------

#[test]
#[ignore = "needs validation/reference/xs_lowenergy.csv from Geant4 (validation/geant4/README.md)"]
fn m3_v2_low_energy_cross_sections_match_geant4() {
    let (header, rows) = read_reference("xs_lowenergy.csv");
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
    assert!(parsed.len() >= 5 * 200, "{} rows", parsed.len());
    let failures = compare_low_energy_cross_sections(&table(), &parsed);
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
#[ignore = "needs validation/reference/slow_{sums,spectra,radial}.csv from Geant4 (validation/geant4/README.md)"]
fn m3_v5_full_slowing_down_matches_geant4() {
    let sums = read_reference("slow_sums.csv");
    let spectra = read_reference("slow_spectra.csv");
    let radial = read_reference("slow_radial.csv");
    let rows = slow_rows(&sums, &spectra, &radial);
    assert!(!rows.is_empty());
    let failures = compare_slow(&table(), &rows, 20_271, &rust_histories);
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ------------------------------------------------------- machinery self-tests ------------

/// Write the three `slow` CSVs from a Rust run, in the format `g4rs_ref` writes.
fn csvs_from_rust_run(
    table: &ProcessTable,
    energy: f64,
    n: u64,
    seed: u64,
) -> [(Vec<String>, Vec<Vec<String>>); 3] {
    let mut cfg = RunConfig::new(energy, n);
    cfg.master_seed = seed;
    let rs = run_detailed(table, &cfg).unwrap();
    let mut header = vec!["energy_eV".to_owned(), "n_events".to_owned()];
    let mut row = vec![energy.to_string(), n.to_string()];
    for name in QUANTITIES {
        let v: Vec<f64> = rs.iter().map(|h| per_event(h, name)).collect();
        header.push(format!("sum_{name}"));
        header.push(format!("sum2_{name}"));
        row.push(v.iter().sum::<f64>().to_string());
        row.push(v.iter().map(|x| x * x).sum::<f64>().to_string());
    }
    for extra in ["sum_nsolv_x_edep", "n_other_process", "n_other_secondary"] {
        header.push(extra.to_owned());
        row.push("0".to_owned());
    }
    let sums = (header, vec![row]);

    let mut srows = Vec::new();
    for class in 0..GENERATION_CLASSES {
        let mut hist = vec![0.0; SPECTRUM_BINS + 2];
        for h in &rs {
            let sp = &h.spectra[class];
            hist[0] += f64::from(sp.underflow);
            for (i, &c) in sp.counts.iter().enumerate() {
                hist[i + 1] += f64::from(c);
            }
            hist[SPECTRUM_BINS + 1] += f64::from(sp.overflow);
        }
        for (b, c) in hist.iter().enumerate() {
            srows.push(vec![
                energy.to_string(),
                (class + 1).to_string(),
                b.to_string(),
                c.to_string(),
            ]);
        }
    }
    let spectra = (
        ["energy_eV", "generation_class", "bin", "count"]
            .map(str::to_owned)
            .to_vec(),
        srows,
    );

    let mut rad = vec![0.0; RADIAL_BINS + 2];
    for h in &rs {
        rad[0] += h.radial.underflow_ev;
        for (i, &v) in h.radial.energy_ev.iter().enumerate() {
            rad[i + 1] += v;
        }
        rad[RADIAL_BINS + 1] += h.radial.overflow_ev;
    }
    let radial = (
        ["energy_eV", "bin", "energy_eV_sum"]
            .map(str::to_owned)
            .to_vec(),
        rad.iter()
            .enumerate()
            .map(|(b, v)| vec![energy.to_string(), b.to_string(), v.to_string()])
            .collect(),
    );
    [sums, spectra, radial]
}

#[test]
fn comparison_machinery_passes_rust_against_rust_with_other_seeds() {
    let root = repo_data_dir();
    if !root.join("dna").is_dir() {
        eprintln!("SKIPPED: Geant4 data not found");
        return;
    }
    let table = table();
    for (energy, n) in [(8.0, 20_000u64), (100.0, 10_000), (1.0e3, 1_000)] {
        let [sums, spectra, radial] = csvs_from_rust_run(&table, energy, n, 111);
        let rows = slow_rows(&sums, &spectra, &radial);
        let failures = compare_slow(&table, &rows, 222, &|_| n);
        assert!(failures.is_empty(), "{energy} eV: {failures:?}");
    }
}

#[test]
fn comparison_machinery_detects_a_perturbed_reference() {
    let root = repo_data_dir();
    if !root.join("dna").is_dir() {
        eprintln!("SKIPPED: Geant4 data not found");
        return;
    }
    let table = table();
    let [sums, spectra, radial] = csvs_from_rust_run(&table, 1.0e3, 10_000, 5);
    let mut rows = slow_rows(&sums, &spectra, &radial);
    // 3 % more vibrational excitations: far beyond the statistical error of 10^4 events
    let q = rows[0].q.get_mut("n_vibexc").unwrap();
    q.mean *= 1.03;
    let failures = compare_slow(&table, &rows, 6, &|_| 10_000);
    assert!(
        failures.iter().any(|m| m.contains("n_vibexc")),
        "{failures:?}"
    );
}

#[test]
fn v5_uses_the_relative_cut_only_when_it_has_power() {
    let s = |mean, se| Sample { mean, se };
    // precise samples 1 % apart: z fails
    assert!(v5("t", s(1.01, 0.001), s(1.0, 0.001)).is_err());
    // precise samples 3 % apart but z < 3 impossible; make z pass with larger se and se < 0.7 %:
    // se 0.004 each -> combined 0.0057 < 0.007; a 2.5 % difference has z = 4.4 -> fails on z
    assert!(v5("t", s(1.025, 0.004), s(1.0, 0.004)).is_err());
    // noisy samples (5 % se) 3 % apart: z passes and the 2 % cut is not applied
    assert!(v5("t", s(1.03, 0.05), s(1.0, 0.05)).is_ok());
    // deterministic quantity must match exactly
    assert!(v5("t", s(100.0, 0.0), s(100.0, 0.0)).is_ok());
    assert!(v5("t", s(100.0, 0.0), s(100.001, 0.0)).is_err());
}

#[test]
fn homogeneity_test_accepts_equal_and_rejects_different_histograms() {
    let a = vec![100.0, 200.0, 300.0, 50.0, 10.0];
    let b: Vec<f64> = a.iter().map(|x| x * 3.0).collect(); // same shape, other total
    let (_, _, p) = homogeneity_p(&a, &b).unwrap();
    assert!(p > 0.99, "{p}");
    let c = vec![300.0, 200.0, 100.0, 50.0, 10.0];
    let (_, _, p) = homogeneity_p(&a, &c).unwrap();
    assert!(p < 1e-6, "{p}");
    assert!(homogeneity_p(&a, &[0.0; 5]).is_none());
}
