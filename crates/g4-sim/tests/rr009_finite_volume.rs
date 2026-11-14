//! RR-009 consumer: Rust against Geant4 `g4rs_ref finite_volume` (docs/07 "Finite-volume boundary
//! acceptance criteria", fixed 2026-11-13 before any data). Ignored: it needs the user-run reference
//! files `validation/reference/rr009_<label>_histories.csv` (validation/geant4/RUN_REQUESTS.md).
//!
//! Criteria as registered:
//! * D1 (Geant4 side, every history): `E0 = edep + escaped` to 1e-9 relative. The Rust side is the
//!   ledger check of `g4-physics/tests/boundary.rs`.
//! * S1 mean escaped-energy fraction and S2 primary escape probability per setting: two-sided
//!   Bonferroni interval on Rust minus Geant4 at family alpha 0.001 over 24 endpoints,
//!   `z = Phi^-1(1 - 0.001/48) = 4.098038123646298`. PASS if the interval lies within +-0.01,
//!   FAIL if it excludes 0 and lies wholly outside +-0.01, otherwise INCONCLUSIVE.
//!
//! Limitation (documented, not a relaxation): Geant4 draws isotropic primaries; the Rust run layer
//! has a fixed primary direction (+z). For the spheres, centred on the source, the observables are
//! rotation invariant, so +z is used. The box is not, and its six endpoints are reported as
//! `NOT RUN` until the Rust source can be isotropic; the gate cannot pass before that.

use g4_core::geometry::Volume;
use g4_core::vec3::Vec3;
use g4_data::datadir::{DataDir, repo_data_dir};
use g4_physics::models::ElasticRotation;
use g4_physics::table::ProcessTable;
use g4_sim::config::RunConfig;
use g4_sim::run::run_summary;
use std::collections::BTreeMap;
use std::path::PathBuf;

const Z_FAMILY: f64 = 4.098038123646298;
const MARGIN: f64 = 0.01;
const ENERGIES_EV: [f64; 3] = [1.0e3, 1.0e4, 1.0e5];
const HISTORIES: u64 = 10_000;

/// Endpoints accepted by user decision on 2026-11-14 after the data were seen (docs/07 amendment):
/// both INCONCLUSIVE only from precision (|diff| < 1 se, half-width 0.0107 vs margin 0.01). They
/// are reported with their statistical verdict and counted as accepted, not as PASS.
const ACCEPTED_INCONCLUSIVE: [(&str, u64, &str); 2] = [
    ("sphere_1000nm", 10_000, "S1"),
    ("sphere_1000nm", 10_000, "S2"),
];

struct Setting {
    label: &'static str,
    sphere_radius_nm: Option<f64>,
    rust_seed: u64,
}

const SETTINGS: [Setting; 4] = [
    Setting {
        label: "sphere_100nm",
        sphere_radius_nm: Some(100.0),
        rust_seed: 2026111311,
    },
    Setting {
        label: "sphere_1000nm",
        sphere_radius_nm: Some(1000.0),
        rust_seed: 2026111312,
    },
    Setting {
        label: "sphere_10000nm",
        sphere_radius_nm: Some(10000.0),
        rust_seed: 2026111313,
    },
    Setting {
        label: "box_1000x1000x100nm",
        sphere_radius_nm: None,
        rust_seed: 2026111314,
    },
];

#[derive(Default)]
struct Moments {
    n: u64,
    sum: f64,
    sum2: f64,
}

impl Moments {
    fn add(&mut self, x: f64) {
        self.n += 1;
        self.sum += x;
        self.sum2 += x * x;
    }
    fn mean(&self) -> f64 {
        self.sum / self.n as f64
    }
    fn std_err(&self) -> f64 {
        let n = self.n as f64;
        let var = (self.sum2 - self.sum * self.sum / n) / (n - 1.0);
        (var.max(0.0) / n).sqrt()
    }
}

fn reference(label: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../validation/reference")
        .join(format!("rr009_{label}_histories.csv"))
}

/// Per energy: (S1 moments of escaped/E0, S2 moments of primary_escaped).
fn geant4(label: &str) -> BTreeMap<u64, (Moments, Moments)> {
    let path = reference(label);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e} (run RR-009 first)", path.display()));
    let mut lines = text.lines().filter(|l| !l.starts_with('#'));
    assert_eq!(
        lines.next(),
        Some(
            "shape,d0_nm,d1_nm,d2_nm,event,energy_eV,edep_eV,escaped_eV,primary_escaped,escaped_tracks"
        )
    );
    let mut out: BTreeMap<u64, (Moments, Moments)> = BTreeMap::new();
    for line in lines {
        let f: Vec<&str> = line.split(',').collect();
        assert_eq!(f.len(), 10, "{line}");
        let e0: f64 = f[5].parse().unwrap();
        let edep: f64 = f[6].parse().unwrap();
        let escaped: f64 = f[7].parse().unwrap();
        let primary: f64 = f[8].parse().unwrap();
        assert!((edep + escaped - e0).abs() <= 1e-9 * e0, "D1 fails: {line}");
        let entry = out.entry(e0.round() as u64).or_default();
        entry.0.add(escaped / e0);
        entry.1.add(primary);
    }
    out
}

fn table() -> ProcessTable {
    let dir = DataDir::new(repo_data_dir()).unwrap();
    ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap()
}

fn verdict(diff: f64, se: f64) -> &'static str {
    let (lo, hi) = (diff - Z_FAMILY * se, diff + Z_FAMILY * se);
    if lo >= -MARGIN && hi <= MARGIN {
        "PASS"
    } else if (lo > 0.0 || hi < 0.0) && (lo > MARGIN || hi < -MARGIN) {
        "FAIL"
    } else {
        "INCONCLUSIVE"
    }
}

#[test]
#[ignore = "needs the user-run RR-009 reference files"]
fn rr009_escape_observables_match_geant4() {
    let table = table();
    let mut failures = Vec::new();
    let mut open = 0usize;
    for setting in &SETTINGS {
        let g4 = geant4(setting.label);
        for energy in ENERGIES_EV {
            let (s1, s2) = &g4[&(energy as u64)];
            assert_eq!(
                s1.n, HISTORIES,
                "{} {energy} eV history count",
                setting.label
            );
            let Some(radius) = setting.sphere_radius_nm else {
                println!(
                    "{} {energy} eV: S1, S2 NOT RUN (no isotropic Rust source)",
                    setting.label
                );
                open += 2;
                continue;
            };
            let mut config = RunConfig::new(energy, HISTORIES);
            config.master_seed = setting.rust_seed + energy.log10() as u64 * 100;
            config.volume = Some(
                Volume::sphere(
                    Vec3::ZERO,
                    0.0,
                    radius,
                    0.0,
                    std::f64::consts::TAU,
                    0.0,
                    std::f64::consts::PI,
                )
                .unwrap(),
            );
            let summary = run_summary(&table, &config).unwrap();
            let escape = summary.escape.expect("escape block with a volume");
            let endpoints = [
                (
                    "S1",
                    escape.escaped_energy_fraction.mean - s1.mean(),
                    escape.escaped_energy_fraction.std_err.hypot(s1.std_err()),
                ),
                (
                    "S2",
                    escape.primary_escape_fraction.fraction - s2.mean(),
                    escape.primary_escape_fraction.std_err.hypot(s2.std_err()),
                ),
            ];
            for (name, diff, se) in endpoints {
                let v = verdict(diff, se);
                println!(
                    "{} {energy} eV {name}: diff {diff:.6} se {se:.6} {v}",
                    setting.label
                );
                match v {
                    "FAIL" => failures.push(format!("{} {energy} {name}", setting.label)),
                    "INCONCLUSIVE"
                        if ACCEPTED_INCONCLUSIVE.contains(&(
                            setting.label,
                            energy as u64,
                            name,
                        )) =>
                    {
                        println!("  -> accepted by user decision 2026-11-14 (not a PASS)");
                    }
                    "INCONCLUSIVE" => open += 1,
                    _ => {}
                }
            }
        }
    }
    assert!(failures.is_empty(), "FAIL endpoints: {failures:?}");
    println!("open (INCONCLUSIVE or NOT RUN) endpoints: {open}; the gate needs 0");
}
