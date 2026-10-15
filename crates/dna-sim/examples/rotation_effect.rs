//! Size of Geant4's non-unit Champion rotation axis (docs/09 Q13) on the quantities that depend on
//! directions: the primary's range, projected range and the radial extent of the deposits.
//!
//! The two conventions consume the random stream identically (the rotation changes directions
//! only, and no cross section depends on direction in an infinite medium), so for the same
//! `(seed, history)` the path lengths, energies and counts are bitwise identical and only the
//! positions differ: the per-history paired difference is the natural estimator and has a much
//! smaller error than the difference of two independent means. This example prints it.
//!
//! ```text
//! cargo run --release -p dna-sim --example rotation_effect
//! ```

use dna_data::datadir::{DataDir, repo_data_dir};
use dna_physics::models::ElasticRotation;
use dna_physics::table::ProcessTable;
use dna_sim::config::RunConfig;
use dna_sim::run::run_detailed;

fn mean_se(v: &[f64]) -> (f64, f64) {
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    let var = v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0);
    (m, (var / n).sqrt())
}

fn main() {
    let dir = DataDir::new(repo_data_dir()).expect("committed data");
    let geant4 = ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap();
    let exact = ProcessTable::option2_electron(&dir, ElasticRotation::Exact).unwrap();
    println!(
        "{:>9} {:>7} {:>12} {:>12} {:>20} {:>9} {:>22}",
        "E [eV]",
        "N",
        "range G4 [nm]",
        "range ex[nm]",
        "paired diff (ex-G4)",
        "rel [%]",
        "projected diff [nm]"
    );
    for (e, n) in [
        (100.0, 20_000u64),
        (1.0e3, 10_000),
        (1.0e4, 2_000),
        (1.0e5, 400),
        (1.0e6, 40),
    ] {
        let mut cfg = RunConfig::new(e, n);
        cfg.master_seed = 20_270_104;
        let a = run_detailed(&geant4, &cfg).unwrap();
        let b = run_detailed(&exact, &cfg).unwrap();
        // identical streams: path length and energies must agree exactly
        assert!(a.iter().zip(&b).all(|(x, y)| {
            x.outcome.track_length_nm == y.outcome.track_length_nm
                && x.outcome.ledger == y.outcome.ledger
                && x.outcome.counts == y.outcome.counts
        }));
        let ra: Vec<f64> = a.iter().map(|h| h.outcome.range_nm()).collect();
        let rb: Vec<f64> = b.iter().map(|h| h.outcome.range_nm()).collect();
        let d: Vec<f64> = ra.iter().zip(&rb).map(|(x, y)| y - x).collect();
        let pd: Vec<f64> = a
            .iter()
            .zip(&b)
            .map(|(x, y)| y.outcome.projected_range_nm() - x.outcome.projected_range_nm())
            .collect();
        let (ma, _) = mean_se(&ra);
        let (mb, _) = mean_se(&rb);
        let (md, sd) = mean_se(&d);
        let (mp, sp) = mean_se(&pd);
        println!(
            "{e:>9} {n:>7} {ma:>12.2} {mb:>12.2} {md:>+12.3} +- {sd:<6.3} {:>+9.3} {mp:>+12.3} +- {sp:<6.3}",
            md / ma * 100.0
        );
    }
}
