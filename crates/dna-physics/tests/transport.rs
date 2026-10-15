//! Transport-level tests on the real tables: energy ledger of the whole cascade (V7), termination
//! rules and the first-interaction mean free path (V5-style against hand-computed `Sigma`).
//!
//! Pre-registered criteria (fixed before the first run):
//! * V7: `|E_in - sum(parts)| / E_in < 1e-9` for every history at 100 eV, 1 keV, 10 keV, 100 keV
//!   and 1 MeV (docs/07 V7), with the **full secondary cascade transported**: 10^4 histories at
//!   100 eV and 1 keV, 2000 at 10 keV, 200 at 100 keV, 20 at 1 MeV in the normal test run (cost:
//!   a 1 MeV cascade is 2.6e6 interactions, about 3 s of one core); the `--ignored` variant runs
//!   10^4 / 10^4 / 10^4 / 2000 / 200 (about 5 min on 4 cores; result in docs/08). The ledger is
//!   exact per history, so the count only enlarges the sample of random paths. The stricter project goal
//!   of docs/01 A.4 (1e-12) is asserted as well; in the infinite medium every history must end
//!   with everything deposited (`remaining_untracked = untracked_secondaries = escaped = 0`).
//! * The same identities hold for the M2 "record secondaries, do not transport" mode.
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
    transport_electron(table, cfg, primary(e, id), &mut rng, &mut NoObserver).unwrap()
}

fn primary(e: f64, id: u64) -> Primary {
    Primary {
        energy: ev(e),
        position: Vec3::ZERO,
        direction: Vec3::new(0.0, 0.0, 1.0),
        history_id: id,
    }
}

/// `(energy, histories)` of the V7 test in the normal run.
const LEDGER_CASES: [(f64, usize); 5] = [
    (100.0, 10_000),
    (1.0e3, 10_000),
    (1.0e4, 2_000),
    (1.0e5, 200),
    (1.0e6, 20),
];

/// `(energy, histories)` of the `--ignored` variant.
const LEDGER_CASES_FULL: [(f64, usize); 5] = [
    (100.0, 10_000),
    (1.0e3, 10_000),
    (1.0e4, 10_000),
    (1.0e5, 2_000),
    (1.0e6, 200),
];

#[test]
fn energy_ledger_is_conserved_for_the_whole_cascade_at_five_energies() {
    check_cascade_ledger(&LEDGER_CASES);
}

#[test]
#[ignore = "about 5 minutes on 4 cores: 10^4 histories at 100 eV - 10 keV, 2000 at 100 keV, 200 at 1 MeV"]
fn energy_ledger_is_conserved_for_the_whole_cascade_full_statistics() {
    check_cascade_ledger(&LEDGER_CASES_FULL);
}

fn check_cascade_ledger(cases: &[(f64, usize)]) {
    let Some(table) = table() else { return };
    let streams = RngStreams::new(7);
    let cfg = TransportConfig::default();
    for (case, &(e, n)) in cases.iter().enumerate() {
        let started = std::time::Instant::now();
        let results: Vec<(HistoryOutcome, SecondaryRecorder)> = (0..n)
            .into_par_iter()
            .map(|id| {
                let hid = (case as u64) * 100_000 + id as u64;
                let mut rng = streams.stream(hid, Stage::Physics).unwrap();
                let mut rec = SecondaryRecorder::default();
                let out =
                    transport_electron(&table, &cfg, primary(e, hid), &mut rng, &mut rec).unwrap();
                (out, rec)
            })
            .collect();
        let max_rel = results
            .iter()
            .map(|r| r.0.ledger.residual_relative().abs())
            .fold(0.0, f64::max);
        eprintln!(
            "E = {e} eV ({n} histories, {:.1} s, {:.0} interactions/history): max |ledger residual| / E_in = {max_rel:e}",
            started.elapsed().as_secs_f64(),
            results.iter().map(|r| r.0.steps).sum::<u64>() as f64 / n as f64
        );
        assert!(max_rel < 1e-9, "V7 at {e} eV: {max_rel:e}");
        assert!(max_rel < 1e-12, "A.4 goal at {e} eV: {max_rel:e}");
        for (out, rec) in &results {
            let l = &out.ledger;
            assert_eq!(l.e_in, e);
            // infinite medium, everything transported: nothing is left over or lost
            assert_eq!(out.termination, Termination::Completed);
            assert_eq!(l.escaped, 0.0);
            assert_eq!(l.untracked_secondaries, 0.0);
            assert_eq!(l.remaining_untracked, 0.0);
            assert_relative_eq!(l.deposited(), e, max_relative = 1e-12);
            for part in [
                l.deposited_excitation,
                l.deposited_vibrational,
                l.deposited_binding,
                l.deposited_attachment,
                l.deposited_solvation,
            ] {
                assert!(part >= 0.0);
            }
            // every track ends by solvation or attachment; every secondary is a track
            assert_eq!(out.n_tracks, 1 + out.n_secondaries);
            assert_eq!(
                out.solvations + out.counts.get(ProcessKind::Attachment),
                out.n_tracks
            );
            assert_eq!(out.steps, out.counts.total());
            // the recorded secondaries are exactly the created ones
            assert_eq!(rec.secondaries.len() as u64, out.n_secondaries);
            assert!(rec.secondaries.iter().all(|s| s.position.norm().is_finite()
                && (s.direction.norm() - 1.0).abs() < 1e-9
                && s.energy.ev() > 0.0
                && s.generation >= 1));
            assert!(out.track_length_nm > 0.0 && out.track_length_nm.is_finite());
            assert!(out.total_track_length_nm >= out.track_length_nm);
            assert!(out.range_nm() <= out.track_length_nm * (1.0 + 1e-12));
        }
    }
}

#[test]
fn energy_ledger_is_conserved_when_secondaries_are_recorded_but_not_transported() {
    let Some(table) = table() else { return };
    let streams = RngStreams::new(8);
    let cfg = TransportConfig {
        transport_secondaries: false,
        ..TransportConfig::default()
    };
    for (case, e) in [100.0, 1.0e3, 1.0e4, 1.0e5, 1.0e6].into_iter().enumerate() {
        let results: Vec<(HistoryOutcome, f64)> = (0..2_000usize)
            .into_par_iter()
            .map(|id| {
                let hid = (case as u64) * 100_000 + id as u64;
                let mut rng = streams.stream(hid, Stage::Physics).unwrap();
                let mut rec = SecondaryRecorder::default();
                let out =
                    transport_electron(&table, &cfg, primary(e, hid), &mut rng, &mut rec).unwrap();
                let recorded = rec.secondaries.iter().map(|s| s.energy.ev()).sum();
                (out, recorded)
            })
            .collect();
        let max_rel = results
            .iter()
            .map(|r| r.0.ledger.residual_relative().abs())
            .fold(0.0, f64::max);
        assert!(max_rel < 1e-12, "{e} eV: {max_rel:e}");
        for (out, recorded) in &results {
            assert_eq!(out.n_tracks, 1);
            assert_eq!(out.termination, Termination::Completed);
            assert_relative_eq!(
                *recorded,
                out.ledger.untracked_secondaries,
                max_relative = 1e-12,
                epsilon = 1e-9
            );
            assert_eq!(out.ledger.remaining_untracked, 0.0);
            assert_eq!(
                out.solvations + out.counts.get(ProcessKind::Attachment),
                1,
                "the primary ends by solvation or attachment"
            );
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
        ion_events += out.counts.get(ProcessKind::Ionisation);
        without_secondary += out.quirks.ionisation_without_secondary;
        secondaries += out.n_secondaries;
        k_shell += out.quirks.k_shell_deexcitation_skipped;
        assert_eq!(
            out.ionisation_shells.iter().sum::<u64>(),
            out.counts.get(ProcessKind::Ionisation)
        );
        assert_eq!(
            out.excitation_levels.iter().sum::<u64>(),
            out.counts.get(ProcessKind::Excitation)
        );
        assert_eq!(
            out.quirks.k_shell_deexcitation_skipped,
            out.ionisation_shells[4]
        );
    }
    // every ionisation either produced a secondary or is accounted for as "without secondary"
    // (or a no-op below the binding energy, which is unreachable in the real tables)
    assert_eq!(ion_events, secondaries + without_secondary);
    assert!(k_shell > 0, "10 keV electrons ionise the K shell sometimes");
}

#[test]
fn electrons_at_or_below_7_4_ev_are_solvated_at_once() {
    let Some(table) = table() else { return };
    let streams = RngStreams::new(3);
    for e in [2.0, 5.0, 7.4] {
        let out = run_one(&table, &TransportConfig::default(), &streams, 0, e);
        assert_eq!(out.termination, Termination::Completed);
        assert_eq!(out.steps, 0, "no free path is drawn");
        assert_eq!(out.solvations, 1);
        assert_eq!(out.track_length_nm, 0.0);
        assert_eq!(out.ledger.deposited_solvation, e);
        assert_eq!(out.ledger.residual(), 0.0);
        assert_eq!(out.end_position, Vec3::ZERO);
        assert!(out.first_step.is_none());
    }
}

#[test]
fn electrons_between_7_4_and_9_ev_are_removed_by_the_low_energy_processes() {
    // Q14 closed: without excitation (9 eV) and ionisation (11 eV) these electrons used to scatter
    // elastically for ever; Sanche (energy loss <= 0.835 eV) walks them down to 7.4 eV, Melton
    // absorbs some of them.
    let Some(table) = table() else { return };
    let streams = RngStreams::new(3);
    let mut attached = 0u64;
    let mut solvated = 0u64;
    for id in 0..400 {
        let out = run_one(&table, &TransportConfig::default(), &streams, id, 8.0);
        assert_eq!(out.termination, Termination::Completed);
        assert_eq!(out.n_secondaries, 0);
        assert_eq!(out.counts.get(ProcessKind::Excitation), 0);
        assert_eq!(out.counts.get(ProcessKind::Ionisation), 0);
        assert_eq!(out.solvations + out.counts.get(ProcessKind::Attachment), 1);
        attached += out.counts.get(ProcessKind::Attachment);
        solvated += out.solvations;
        assert_relative_eq!(out.ledger.residual(), 0.0, epsilon = 1e-12);
        assert_relative_eq!(out.ledger.deposited(), 8.0, max_relative = 1e-12);
    }
    assert_eq!(attached + solvated, 400);
    // Melton (~0.003/nm) against Sanche (~0.59/nm) from 8 eV down to 7.4 eV: attachment is a
    // minority branch but not negligible over the ~10 vibrational events needed
    assert!(solvated > 300, "{solvated}");
}

#[test]
fn a_9_ev_electron_excites_at_most_once() {
    let Some(table) = table() else { return };
    let streams = RngStreams::new(3);
    for id in 0..200 {
        let out = run_one(&table, &TransportConfig::default(), &streams, id, 9.0);
        assert!(
            out.counts.get(ProcessKind::Excitation) <= 1,
            "9 eV allows at most one excitation"
        );
        assert_eq!(out.counts.get(ProcessKind::Ionisation), 0);
        if out.counts.get(ProcessKind::Excitation) == 1 {
            assert!(out.ledger.deposited_excitation >= 8.22 - 1e-12);
        }
        assert_relative_eq!(out.ledger.deposited(), 9.0, max_relative = 1e-12);
    }
}

#[test]
fn tracking_cut_stops_below_the_cut_and_is_strict() {
    let Some(table) = table() else { return };
    let streams = RngStreams::new(5);
    let cut = |c: f64| TransportConfig {
        tracking_cut: Some(ev(c)),
        transport_secondaries: false,
        ..TransportConfig::default()
    };
    for id in 0..300 {
        let out = run_one(&table, &cut(100.0), &streams, id, 1.0e3);
        assert_eq!(out.termination, Termination::TrackingCut);
        assert!(out.ledger.remaining_untracked < 100.0);
        assert!(out.ledger.deposited() + out.ledger.untracked_secondaries > 700.0 - 1e-9);
        assert_relative_eq!(out.ledger.residual(), 0.0, epsilon = 1e-9);
    }
    // with the cascade transported the primary is still cut, its secondaries are followed down
    let full = TransportConfig {
        tracking_cut: Some(ev(100.0)),
        ..TransportConfig::default()
    };
    for id in 0..100 {
        let out = run_one(&table, &full, &streams, id, 1.0e3);
        assert_eq!(out.termination, Termination::TrackingCut);
        assert!(out.ledger.remaining_untracked < 100.0);
        assert_eq!(out.ledger.untracked_secondaries, 0.0);
        assert_relative_eq!(out.ledger.residual(), 0.0, epsilon = 1e-9);
        assert!(out.n_tracks == 1 + out.n_secondaries);
    }
    // `E < cut` is strict: a primary exactly at the cut still interacts
    let at = run_one(&table, &cut(1000.0), &streams, 0, 1.0e3);
    assert!(at.steps >= 1);
    let above = run_one(
        &table,
        &cut(f64::from_bits(1000f64.to_bits() + 1)),
        &streams,
        0,
        1.0e3,
    );
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

/// `(E eV, Sigma_total /nm, [Sigma_el, Sigma_ex, Sigma_io, Sigma_vib, Sigma_att] /nm)` from the
/// hand computation (`scripts/hand_cross_sections.py`). At exactly 100 eV the Sanche window
/// `[2, 100] eV` is still open, so the vibrational term is non-zero there.
const HAND: [(f64, f64, [f64; 5]); 3] = [
    (
        100.0,
        1.9249770001097,
        [
            0.8141190997411056,
            0.0223282710194591,
            0.8023861553842268,
            0.2861434739649086,
            0.0,
        ],
    ),
    (
        4321.0,
        0.12516957003369536,
        [
            0.04782758037698331,
            0.0011415633581586882,
            0.07620042629855338,
            0.0,
            0.0,
        ],
    ),
    (
        250000.0,
        0.004613586760458186,
        [
            0.0008298780439745434,
            5.037236935827414e-05,
            0.0037333363471253682,
            0.0,
            0.0,
        ],
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
        let lengths: Vec<f64> = outs
            .iter()
            .map(|o| o.first_step.unwrap().length_nm)
            .collect();
        let mean = lengths.iter().sum::<f64>() / N as f64;
        let var = lengths.iter().map(|l| (l - mean) * (l - mean)).sum::<f64>() / (N as f64 - 1.0);
        let se = (var / N as f64).sqrt();
        let want = 1.0 / sigma;
        let z = (mean - want) / se;
        eprintln!(
            "E = {e} eV: mean free path {mean:.6} +- {se:.6} nm, hand {want:.6} nm, z = {z:+.2}"
        );
        assert!(z.abs() < 3.0, "E = {e}: z = {z}");
        // the standard error of an exponential sample is mean/sqrt(n): consistency of the estimator
        assert_relative_eq!(se, want / (N as f64).sqrt(), max_relative = 0.02);

        // first-interaction fractions are Sigma_i / Sigma
        let mut counts = [0usize; ProcessKind::COUNT];
        for o in &outs {
            counts[o.first_step.unwrap().process.index()] += 1;
        }
        for p in ProcessKind::ALL {
            let frac = parts[p.index()] / sigma;
            if frac == 0.0 {
                assert_eq!(counts[p.index()], 0, "E = {e}, {p:?} is closed");
                continue;
            }
            let sd = (frac * (1.0 - frac) / N as f64).sqrt();
            let zf = (counts[p.index()] as f64 / N as f64 - frac) / sd;
            assert!(zf.abs() < 4.0, "E = {e}, {p:?}: z = {zf}");
        }
    }
}
