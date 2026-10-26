//! Performance baseline (docs/08: "no optimisation; record baseline"). Never part of the normal
//! test run; execute with
//! `cargo test --release -p g4-sim --test baseline -- --ignored --nocapture`
//! and compare numbers only on the same machine. Single thread, fixed seed. Since M3 the whole
//! secondary cascade is transported (the M2 baseline was primaries only).

use g4_data::datadir::{DataDir, repo_data_dir};
use g4_physics::models::ElasticRotation;
use g4_physics::table::ProcessTable;
use g4_sim::config::RunConfig;
use g4_sim::run::run_summary;
use std::time::Instant;

#[test]
#[ignore = "timing only"]
fn single_thread_throughput() {
    let dir = DataDir::new(repo_data_dir()).unwrap();
    let t0 = Instant::now();
    let table = ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap();
    println!("table load: {:?}", t0.elapsed());
    println!(
        "{:>9} {:>7} {:>12} {:>12} {:>14}",
        "E [eV]", "N", "time [s]", "steps/hist", "steps/s"
    );
    for (e, n) in [
        (100.0, 4000u64),
        (1.0e3, 2000),
        (1.0e4, 400),
        (1.0e5, 40),
        (1.0e6, 6),
    ] {
        let mut cfg = RunConfig::new(e, n);
        cfg.threads = Some(1);
        let t = Instant::now();
        let s = run_summary(&table, &cfg).unwrap();
        let secs = t.elapsed().as_secs_f64();
        let steps: u64 = s.processes.iter().map(|p| p.interactions).sum();
        println!(
            "{e:>9.0} {n:>7} {secs:>12.3} {:>12.0} {:>14.3e}",
            steps as f64 / n as f64,
            steps as f64 / secs
        );
    }
}
