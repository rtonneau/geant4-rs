//! Escape scoring in a volume (docs/04 D.6), on the real tables.
//!
//! Pre-registered criteria:
//! * Infinite water: the summary has no `escape` key.
//! * A volume that contains every track: `escape` is present with zero fractions and counts.
//! * A small volume: the escape block is consistent with the per-history results and the
//!   ledger: primary fraction = primary exits / n with SE `sqrt(p(1-p)/n)`, mean escaped
//!   tracks = sum `n_escaped` / n, energy fraction mean = mean `escaped / E_in` (1e-12
//!   relative), spectrum totals add up (bins + under + overflow, primary + secondaries = all),
//!   and both primary and secondary escapes occur.

use g4_core::geometry::Volume;
use g4_core::vec3::Vec3;
use g4_data::datadir::{DataDir, repo_data_dir};
use g4_physics::models::ElasticRotation;
use g4_physics::table::ProcessTable;
use g4_sim::config::RunConfig;
use g4_sim::run::{run_detailed, run_summary};
use g4_sim::scoring::EscapeSpectrum;

fn table() -> Option<ProcessTable> {
    let root = repo_data_dir();
    if !root.join("dna").is_dir() || !root.join("MANIFEST.toml").is_file() {
        eprintln!("SKIPPED: Geant4 data not found at {}", root.display());
        return None;
    }
    let dir = DataDir::new(&root).unwrap();
    Some(ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap())
}

fn config(volume: Option<Volume>) -> RunConfig {
    let mut c = RunConfig::new(1.0e4, 200);
    c.master_seed = 77;
    c.volume = volume;
    c
}

fn total(s: &EscapeSpectrum) -> u64 {
    s.counts.iter().sum::<u64>() + s.underflow + s.overflow
}

#[test]
fn infinite_water_has_no_escape_block() {
    let Some(table) = table() else { return };
    let s = run_summary(&table, &config(None)).unwrap();
    assert!(s.escape.is_none());
    assert!(serde_json::to_value(&s).unwrap().get("escape").is_none());
}

#[test]
fn a_volume_containing_every_track_reports_zero_escapes() {
    let Some(table) = table() else { return };
    let huge = Volume::box_(Vec3::ZERO, Vec3::new(1.0e9, 1.0e9, 1.0e9)).unwrap();
    let s = run_summary(&table, &config(Some(huge))).unwrap();
    let e = s.escape.expect("escape block with a volume");
    assert_eq!(e.primary_escape_fraction.fraction, 0.0);
    assert_eq!(e.escaped_tracks_per_history.mean, 0.0);
    assert_eq!(e.escaped_energy_fraction.mean, 0.0);
    assert_eq!(e.spectra.all.n_tracks, 0);
    assert!(e.spectra.all.mean_energy_ev.is_none());
}

#[test]
fn escape_block_is_consistent_with_histories_and_ledger() {
    let Some(table) = table() else { return };
    let v = Volume::box_(Vec3::ZERO, Vec3::new(60.0, 60.0, 40.0)).unwrap();
    let c = config(Some(v));
    let histories = run_detailed(&table, &c).unwrap();
    let e = run_summary(&table, &c).unwrap().escape.unwrap();
    let n = histories.len() as f64;

    let primary: u64 = histories
        .iter()
        .map(|r| r.exits.iter().filter(|x| x.generation == 0).count() as u64)
        .sum();
    let tracks: u64 = histories.iter().map(|r| r.outcome.n_escaped).sum();
    let fraction: f64 = histories
        .iter()
        .map(|r| r.outcome.ledger.escaped / r.outcome.ledger.e_in)
        .sum::<f64>()
        / n;
    assert!(primary > 0 && tracks > primary, "needs both kinds of escape");

    let p = primary as f64 / n;
    assert_eq!(e.primary_escape_fraction.n, histories.len() as u64);
    assert_eq!(e.primary_escape_fraction.fraction, p);
    assert!((e.primary_escape_fraction.std_err - (p * (1.0 - p) / n).sqrt()).abs() < 1e-15);
    assert!((e.escaped_tracks_per_history.mean - tracks as f64 / n).abs() < 1e-12);
    assert!((e.escaped_energy_fraction.mean - fraction).abs() <= 1e-12 * fraction);

    let sp = &e.spectra;
    assert_eq!(sp.edges_ev, g4_sim::history::spectrum_edges_ev());
    assert_eq!(sp.primary.n_tracks, primary);
    assert_eq!(sp.all.n_tracks, tracks);
    assert_eq!(sp.primary.n_tracks + sp.secondaries.n_tracks, sp.all.n_tracks);
    for s in [&sp.primary, &sp.secondaries, &sp.all] {
        assert_eq!(total(s), s.n_tracks);
    }
}
