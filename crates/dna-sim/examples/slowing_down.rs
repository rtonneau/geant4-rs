//! Full slowing-down of electrons in liquid water (milestone M3), run with both conventions of
//! Geant4's Champion elastic rotation (docs/09 Q13) and written as JSON with provenance.
//!
//! ```text
//! cargo run --release -p dna-sim --example slowing_down -- <out_dir> [energy_eV:histories ...]
//! ```
//! Default cases: 100 eV x 20000, 1 keV x 10000, 10 keV x 2000, 100 keV x 200, 1 MeV x 20
//! (about 10 minutes on 4 cores, dominated by 1 MeV: 2.6e6 interactions per history). One file
//! `slowing_down_<rotation>_<energy>eV.json` per case and rotation, plus a one-line-per-case
//! summary table on stdout. The seed is fixed (`SEED`), so a rerun on the same toolchain and
//! data reproduces the files bit for bit.

use dna_core::manifest::Manifest;
use dna_data::datadir::{DataDir, repo_data_dir};
use dna_physics::models::ElasticRotation;
use dna_physics::table::ProcessTable;
use dna_sim::config::RunConfig;
use dna_sim::run::run;
use std::path::PathBuf;
use std::time::Instant;

const SEED: u64 = 20_270_103;
const DEFAULT_CASES: [(f64, u64); 5] = [
    (100.0, 20_000),
    (1.0e3, 10_000),
    (1.0e4, 2_000),
    (1.0e5, 200),
    (1.0e6, 20),
];

fn main() {
    let mut args = std::env::args().skip(1);
    let out_dir = PathBuf::from(
        args.next()
            .expect("usage: slowing_down <out_dir> [E:N ...]"),
    );
    let mut cases: Vec<(f64, u64)> = args
        .map(|a| {
            let (e, n) = a.split_once(':').expect("case must be energy_eV:histories");
            (e.parse().expect("energy"), n.parse().expect("histories"))
        })
        .collect();
    if cases.is_empty() {
        cases = DEFAULT_CASES.to_vec();
    }
    std::fs::create_dir_all(&out_dir).expect("create output directory");

    let root = repo_data_dir();
    let dir = DataDir::new(&root).expect("committed data");
    let manifest = Manifest::load(&root.join("MANIFEST.toml")).expect("manifest");

    println!(
        "{:<8} {:>9} {:>7} {:>11} {:>13} {:>10} {:>11} {:>9} {:>8}",
        "rotation",
        "E [eV]",
        "N",
        "track [nm]",
        "range [nm]",
        "W [eV]",
        "solv/100eV",
        "sec/hist",
        "time [s]"
    );
    for rotation in [ElasticRotation::Geant4, ElasticRotation::Exact] {
        let table = ProcessTable::option2_electron(&dir, rotation).expect("process table");
        for &(energy, n) in &cases {
            let mut config = RunConfig::new(energy, n);
            config.master_seed = SEED;
            let t = Instant::now();
            let output = run(&table, &config, &manifest).expect("run");
            let secs = t.elapsed().as_secs_f64();
            let s = &output.summary;
            println!(
                "{:<8} {:>9} {:>7} {:>11.2} {:>13.2} {:>10.3} {:>11.3} {:>9.1} {:>8.1}",
                format!("{rotation:?}"),
                energy,
                n,
                s.tracks.primary_track_length_nm.mean,
                s.tracks.range_nm.mean,
                s.w_value.w_ev_ratio_of_totals.map_or(f64::NAN, |w| w.ratio),
                s.solvation.per_100_ev.map_or(f64::NAN, |w| w.ratio),
                s.tracks.secondaries_per_history.mean,
                secs
            );
            let path = out_dir.join(format!(
                "slowing_down_{}_{}eV.json",
                format!("{rotation:?}").to_lowercase(),
                energy
            ));
            output.write_json(&path).expect("write json");
        }
    }
}
