//! Physical sanity of the full slowing-down (milestone M3). **Not a Geant4 comparison**: these
//! checks only say that the numbers are of the right kind. Every threshold was written in
//! docs/08 ("M3 a-priori expectations") and committed before any full-cascade W-value, range or
//! solvation count was looked at.
//!
//! * Identities (exact): `E_dep = E_in` for every history, every track ends by solvation or
//!   attachment (`N_solv + N_att = 1 + N_secondaries`), no energy is left untracked.
//! * Range: the mean full track length and the mean range of the primary increase strictly with
//!   the energy 1 keV < 10 keV < 100 keV < 1 MeV, each step by more than 5 combined standard errors.
//! * W-like quantity `E_in / N_ionisation` (ratio of totals): 18 eV <= W <= 36 eV for 1, 10, 100
//!   keV and 1 MeV; at 100 eV only `W > 10.79 eV` (the lowest ionisation energy) and finite.
//! * Solvation events per 100 eV deposited: between 3 and 6 for E >= 1 keV (about 4-5 expected).
//! * The first-generation secondary spectrum falls with energy: counts per decade
//!   [10, 100) eV > [100, 1000) eV > [1 keV, 10 keV) at 100 keV.

use dna_data::datadir::{DataDir, repo_data_dir};
use dna_physics::models::{ElasticRotation, ProcessKind};
use dna_physics::table::ProcessTable;
use dna_sim::config::RunConfig;
use dna_sim::run::run_summary;
use dna_sim::scoring::RunSummary;

fn table(rotation: ElasticRotation) -> Option<ProcessTable> {
    let root = repo_data_dir();
    if !root.join("dna").is_dir() {
        eprintln!("SKIPPED: Geant4 data not found at {}", root.display());
        return None;
    }
    let dir = DataDir::new(&root).unwrap();
    Some(ProcessTable::option2_electron(&dir, rotation).unwrap())
}

/// `(energy eV, histories)`: cost-limited (a 1 MeV cascade is 2.6e6 interactions).
const CASES: [(f64, u64); 5] = [
    (100.0, 4000),
    (1.0e3, 1000),
    (1.0e4, 300),
    (1.0e5, 40),
    (1.0e6, 8),
];

fn run_cases(table: &ProcessTable, seed: u64) -> Vec<(f64, RunSummary)> {
    CASES
        .iter()
        .map(|&(e, n)| {
            let mut cfg = RunConfig::new(e, n);
            cfg.master_seed = seed;
            (e, run_summary(table, &cfg).unwrap())
        })
        .collect()
}

#[test]
fn full_slowing_down_identities_range_w_value_and_solvation_yield_are_sane() {
    let Some(table) = table(ElasticRotation::Geant4) else {
        return;
    };
    let results = run_cases(&table, 2027);
    for (e, s) in &results {
        let en = &s.energy;
        // exact identities
        assert!(en.max_abs_ledger_residual_relative < 1e-12, "{e}");
        assert!((en.deposited_total_ev.mean - e).abs() < 1e-9 * e, "{e}");
        assert_eq!(en.untracked_secondaries_ev.mean, 0.0);
        assert_eq!(en.remaining_untracked_ev.mean, 0.0);
        assert_eq!(s.terminations.completed, s.n_histories);
        let tracks = s.tracks.tracks_per_history.mean * s.n_histories as f64;
        let ends = s.solvation.events + s.processes[ProcessKind::Attachment.index()].interactions;
        assert!(
            (tracks - ends as f64).abs() < 1e-6,
            "{e}: {tracks} vs {ends}"
        );
        eprintln!(
            "E = {e:>9} eV: primary track {:>12.2} nm, range {:>12.2} nm, W = {:.3} +- {:.3} eV, \
             solvations/100 eV = {:.3}, secondaries/history = {:.1}",
            s.tracks.primary_track_length_nm.mean,
            s.tracks.range_nm.mean,
            s.w_value.w_ev_ratio_of_totals.unwrap().ratio,
            s.w_value.w_ev_ratio_of_totals.unwrap().std_err,
            s.solvation.per_100_ev.unwrap().ratio,
            s.tracks.secondaries_per_history.mean,
        );
    }

    // range: strictly increasing, well separated
    for pair in results.windows(2) {
        let (e1, a) = (&pair[0].0, &pair[0].1);
        let (e2, b) = (&pair[1].0, &pair[1].1);
        for (label, x, y) in [
            (
                "primary track length",
                a.tracks.primary_track_length_nm,
                b.tracks.primary_track_length_nm,
            ),
            ("range", a.tracks.range_nm, b.tracks.range_nm),
        ] {
            let z = (y.mean - x.mean) / (x.std_err.powi(2) + y.std_err.powi(2)).sqrt();
            assert!(
                z > 5.0,
                "{label} not increasing from {e1} to {e2} eV: {} -> {} (z = {z})",
                x.mean,
                y.mean
            );
        }
    }

    // W-like quantity
    for (e, s) in &results {
        let w = s.w_value.w_ev_ratio_of_totals.unwrap();
        assert!(
            w.ratio.is_finite() && w.ratio > 10.79,
            "W({e}) = {}",
            w.ratio
        );
        if *e >= 1.0e3 {
            assert!(
                (18.0..=36.0).contains(&w.ratio),
                "W({e}) = {} eV outside the pre-registered 18-36 eV window",
                w.ratio
            );
            let y = s.solvation.per_100_ev.unwrap().ratio;
            assert!((3.0..=6.0).contains(&y), "solvations/100 eV at {e} = {y}");
        }
    }
}

#[test]
fn first_generation_secondary_spectrum_falls_with_energy() {
    let Some(table) = table(ElasticRotation::Geant4) else {
        return;
    };
    let mut cfg = RunConfig::new(1.0e5, 30);
    cfg.master_seed = 11;
    let s = run_summary(&table, &cfg).unwrap();
    let gen1 = &s.secondary_spectra.by_generation[0];
    // 10 bins per decade starting at 0.1 eV: decade k covers bins [10 k, 10 k + 10)
    let decade = |k: usize| -> u64 { gen1.counts[10 * k..10 * k + 10].iter().sum() };
    let (d10, d100, d1000) = (decade(2), decade(3), decade(4));
    eprintln!(
        "first generation, 100 keV: [10,100) eV: {d10}, [100,1000) eV: {d100}, [1,10) keV: {d1000}"
    );
    assert!(d10 > d100 && d100 > d1000 && d1000 > 0);
}
