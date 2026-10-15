//! Transport-level tests on the real tables: energy ledger (V7), termination rules and the
//! first-interaction mean free path (V5-style against hand-computed `Sigma`).
//!
//! Pre-registered criteria (fixed before the first run):
//! * V7: `|E_in - sum(parts)| / E_in < 1e-9` for every one of 10^4 histories at 100 eV, 1 keV,
//!   10 keV, 100 keV and 1 MeV (docs/07 V7). The stricter project goal of docs/01 A.4 (1e-12) is
//!   reported, and asserted at the observed level of this implementation's compensated sums.
//! * mfp: the mean first free path of 10^5 histories lies within 3 standard errors of `1/Sigma`
//!   (hand value from `scripts/hand_cross_sections.py`), standard error = `mean/sqrt(n)`
//!   (exponential distribution) estimated from the sample variance.
//! * first-process fractions `Sigma_i/Sigma`: within 4 binomial standard errors.

mod common;

use approx::assert_relative_eq;
use common::{data, ev};
use dna_core::rng::{RngStreams, Stage};
use dna_core::vec3::Vec3;
use dna_physics::models::{ElasticRotation, ProcessKind};
use dna_physics::table::ProcessTable;
use dna_physics::transport::{
    HistoryOutcome, NoObserver, Primary, SecondaryRecorder, Termination, TransportConfig,
    transport_electron,
};
use rayon::prelude::*;

fn table() -> Option<ProcessTable> {
    let dir = data()?;
    Some(ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap())
}

fn run_one(
    table: &ProcessTable,
    cfg: &TransportConfig,
    streams: &RngStreams,
    id: u64,
    e: f64,
) -> HistoryOutcome {
    let mut rng = streams.stream(id, Stage::Physics).unwrap();
    transport_electron(
        table,
        cfg,
        Primary {
            energy: ev(e),
            position: Vec3::ZERO,
            direction: Vec3::new(0.0, 0.0, 1.0),
        },
        &mut rng,
        &mut NoObserver,
    )
    .unwrap()
}

#[test]
fn energy_ledger_is_conserved_over_1e4_histories_at_five_energies() {
    let Some(table) = table() else { return };
    let streams = RngStreams::new(7);
    let cfg = TransportConfig::default();
    for (case, e) in [100.0, 1.0e3, 1.0e4, 1.0e5, 1.0e6].into_iter().enumerate() {
        let results: Vec<(f64, HistoryOutcome, f64)> = (0..10_000usize)
            .into_par_iter()
            .map(|id| {
                let mut rng = streams
                    .stream((case as u64) * 100_000 + id as u64, Stage::Physics)
                    .unwrap();
                let mut rec = SecondaryRecorder::default();
                let out = transport_electron(
                    &table,
                    &cfg,
                    Primary {
                        energy: ev(e),
                        position: Vec3::ZERO,
                        direction: Vec3::new(0.0, 0.0, 1.0),
                    },
                    &mut rng,
                    &mut rec,
                )
                .unwrap();
                let recorded: f64 = rec.secondaries.iter().map(|s| s.energy.ev()).sum();
                assert_eq!(rec.secondaries.len() as u64, out.n_secondaries);
                assert!(rec.secondaries.iter().all(|s| s.position.norm().is_finite()
                    && (s.direction.norm() - 1.0).abs() < 1e-9
                    && s.energy.ev() > 0.0));
                (out.ledger.residual_relative().abs(), out, recorded)
            })
            .collect();
        let max_rel = results.iter().map(|r| r.0).fold(0.0, f64::max);
        eprintln!("E = {e} eV: max |ledger residual| / E_in = {max_rel:e}");
        assert!(max_rel < 1e-9, "V7 at {e} eV: {max_rel:e}");
        assert!(max_rel < 1e-12, "A.4 goal at {e} eV: {max_rel:e}");
        for (_, out, recorded) in &results {
            let l = &out.ledger;
            assert_eq!(l.e_in, e);
            // the recorded secondaries are exactly the secondary part of the ledger
            assert_relative_eq!(*recorded, l.secondaries, max_relative = 1e-12, epsilon = 1e-9);
            // parts are non-negative and the remainder is below the lowest inelastic threshold
            assert!(l.deposited_excitation >= 0.0 && l.deposited_binding >= 0.0);
            assert!(l.secondaries >= 0.0);
            assert!((0.0..9.0).contains(&l.remaining_untracked), "{}", l.remaining_untracked);
            assert!(matches!(
                out.termination,
                Termination::BelowLowestWindow | Termination::NoInelasticChannel
            ));
            assert_eq!(out.steps, out.counts.total());
            assert!(out.track_length_nm > 0.0 && out.track_length_nm.is_finite());
            assert!(out.range_nm() <= out.track_length_nm * (1.0 + 1e-12));
        }
    }
}

#[test]
fn quirk_counters_are_consistent_with_the_events() {
    let Some(table) = table() else { return };
    let streams = RngStreams::new(11);
    let mut ion_events = 0u64;
    let mut without_secondary = 0u64;
    let mut secondaries = 0u64;
    let mut k_shell = 0u64;
    for id in 0..300 {
        let out = run_one(&table, &TransportConfig::default(), &streams, id, 1.0e4);
        ion_events += out.counts.ionisation;
        without_secondary += out.quirks.ionisation_without_secondary;
        secondaries += out.n_secondaries;
        k_shell += out.quirks.k_shell_deexcitation_skipped;
        assert_eq!(out.ionisation_shells.iter().sum::<u64>(), out.counts.ionisation);
        assert_eq!(out.excitation_levels.iter().sum::<u64>(), out.counts.excitation);
        assert_eq!(out.quirks.k_shell_deexcitation_skipped, out.ionisation_shells[4]);
    }
    // every ionisation either produced a secondary or is accounted for as "without secondary"
    // (or a no-op below the binding energy, which is unreachable in the real tables)
    assert_eq!(ion_events, secondaries + without_secondary);
    assert!(k_shell > 0, "10 keV electrons ionise the K shell sometimes");
}

#[test]
fn only_elastic_energies_terminate_immediately_and_book_the_remainder() {
    let Some(table) = table() else { return };
    let streams = RngStreams::new(3);
    for e in [7.4, 8.0, 8.999] {
        let out = run_one(&table, &TransportConfig::default(), &streams, 0, e);
        assert_eq!(out.termination, Termination::NoInelasticChannel);
        assert_eq!(out.steps, 0);
        assert_eq!(out.track_length_nm, 0.0);
        assert_eq!(out.ledger.remaining_untracked, e);
        assert_eq!(out.ledger.residual(), 0.0);
        assert_eq!(out.end_position, Vec3::ZERO);
        assert!(out.first_step.is_none());
    }
}

#[test]
fn a_9_ev_electron_excites_once_or_elastically_scatters_first_then_stops_below_7_4_ev() {
    let Some(table) = table() else { return };
    let streams = RngStreams::new(3);
    for id in 0..200 {
        let out = run_one(&table, &TransportConfig::default(), &streams, id, 9.0);
        assert!(out.counts.excitation <= 1, "9 eV allows at most one excitation");
        assert_eq!(out.counts.ionisation, 0);
        if out.counts.excitation == 1 {
            assert_eq!(out.termination, Termination::BelowLowestWindow);
            assert!(out.ledger.remaining_untracked < 7.4);
            assert!(out.ledger.deposited_excitation >= 8.22 - 1e-12);
        } else {
            // every excitation attempt at 9 eV is a no-op or never happened
            assert!(out.ledger.remaining_untracked <= 9.0);
        }
    }
}

#[test]
fn tracking_cut_stops_below_the_cut_and_is_strict() {
    let Some(table) = table() else { return };
    let streams = RngStreams::new(5);
    let cut = |c: f64| TransportConfig {
        tracking_cut: Some(ev(c)),
        ..TransportConfig::default()
    };
    for id in 0..300 {
        let out = run_one(&table, &cut(100.0), &streams, id, 1.0e3);
        assert_eq!(out.termination, Termination::TrackingCut);
        assert!(out.ledger.remaining_untracked < 100.0);
        assert!(out.ledger.deposited() + out.ledger.secondaries > 700.0 - 1e-9);
        assert_relative_eq!(out.ledger.residual(), 0.0, epsilon = 1e-9);
    }
    // `E < cut` is strict: a primary exactly at the cut still interacts
    let at = run_one(&table, &cut(1000.0), &streams, 0, 1.0e3);
    assert!(at.steps >= 1);
    let above = run_one(&table, &cut(f64::from_bits(1000f64.to_bits() + 1)), &streams, 0, 1.0e3);
    assert_eq!(above.steps, 0);
    assert_eq!(above.termination, Termination::TrackingCut);
}

#[test]
fn interaction_limit_stops_after_n_interactions() {
    let Some(table) = table() else { return };
    let streams = RngStreams::new(5);
    for n in [1u64, 3] {
        let cfg = TransportConfig {
            max_interactions: Some(n),
            ..TransportConfig::default()
        };
        for id in 0..50 {
            let out = run_one(&table, &cfg, &streams, id, 5.0e3);
            assert_eq!(out.termination, Termination::InteractionLimit);
            assert_eq!(out.steps, n);
            assert_eq!(out.counts.total(), n);
            assert_relative_eq!(out.ledger.residual(), 0.0, epsilon = 1e-9);
        }
    }
}

/// `(E eV, Sigma_total /nm, [Sigma_el, Sigma_ex, Sigma_io] /nm)` from the hand computation.
const HAND: [(f64, f64, [f64; 3]); 3] = [
    (
        100.0,
        1.6388335261447915,
        [0.8141190997411056, 0.0223282710194591, 0.8023861553842268],
    ),
    (
        4321.0,
        0.12516957003369536,
        [0.04782758037698331, 0.0011415633581586882, 0.07620042629855338],
    ),
    (
        250000.0,
        0.004613586760458186,
        [0.0008298780439745434, 5.037236935827414e-05, 0.0037333363471253682],
    ),
];

#[test]
fn mean_first_free_path_equals_one_over_the_hand_computed_sigma() {
    let Some(table) = table() else { return };
    let streams = RngStreams::new(2025);
    let cfg = TransportConfig {
        max_interactions: Some(1),
        ..TransportConfig::default()
    };
    const N: usize = 100_000;
    for (case, (e, sigma, parts)) in HAND.into_iter().enumerate() {
        let outs: Vec<HistoryOutcome> = (0..N)
            .into_par_iter()
            .map(|id| run_one(&table, &cfg, &streams, (case * N + id) as u64, e))
            .collect();
        let lengths: Vec<f64> = outs.iter().map(|o| o.first_step.unwrap().length_nm).collect();
        let mean = lengths.iter().sum::<f64>() / N as f64;
        let var = lengths.iter().map(|l| (l - mean) * (l - mean)).sum::<f64>() / (N as f64 - 1.0);
        let se = (var / N as f64).sqrt();
        let want = 1.0 / sigma;
        let z = (mean - want) / se;
        eprintln!("E = {e} eV: mean free path {mean:.6} +- {se:.6} nm, hand {want:.6} nm, z = {z:+.2}");
        assert!(z.abs() < 3.0, "E = {e}: z = {z}");
        // the standard error of an exponential sample is mean/sqrt(n): consistency of the estimator
        assert_relative_eq!(se, want / (N as f64).sqrt(), max_relative = 0.02);

        // first-interaction fractions are Sigma_i / Sigma
        let mut counts = [0usize; 3];
        for o in &outs {
            counts[o.first_step.unwrap().process.index()] += 1;
        }
        for p in ProcessKind::ALL {
            let frac = parts[p.index()] / sigma;
            let sd = (frac * (1.0 - frac) / N as f64).sqrt();
            let zf = (counts[p.index()] as f64 / N as f64 - frac) / sd;
            assert!(zf.abs() < 4.0, "E = {e}, {p:?}: z = {zf}");
        }
    }
}
