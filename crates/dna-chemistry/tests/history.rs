//! The physicochemical stage of one history: ordering, electron-hole recombination, random-number consumption and
//! thread-count independence. Scripted generators make every decision of the stage reproducible by hand.
//!
//! Onsager radius used below: 0.7116353983042855 nm (independent Python evaluation, tests/fixtures/hand_values.json);
//! recombination probability `p(d) = 1 - exp(-r_O/d)`: p(1 nm) = 0.5091591795, p(2 nm) = 0.2993996714.

mod common;

use dna_chemistry::PhysChemEngine;
use dna_core::physchem::{PhysChemEvent, PhysChemKind};
use dna_core::rng::{RngStreams, Stage};
use dna_core::species::{Species, SpeciesInstance};
use dna_core::testing::ScriptedRng;
use dna_core::units::Energy;
use dna_core::vec3::Vec3;
use rayon::prelude::*;

fn ev(kind: PhysChemKind, index: u8, p: [f64; 3]) -> PhysChemEvent {
    PhysChemEvent {
        kind,
        level_or_shell: index,
        position: Vec3::new(p[0], p[1], p[2]),
        deposited_energy: Energy::from_ev(10.0),
        parent_history: 0,
    }
}

fn tags(species: &[SpeciesInstance]) -> Vec<Species> {
    species.iter().map(|s| s.species).collect()
}

const ORIGIN: [f64; 3] = [0.0, 0.0, 0.0];

#[test]
fn solvation_creates_an_e_aq_at_the_event_position_without_random_numbers() {
    let engine = PhysChemEngine::option3().unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let r = engine
        .convert_history(
            &[ev(PhysChemKind::Solvation, 0, [1.5, -2.0, 0.25])],
            &mut rng,
        )
        .unwrap();
    assert_eq!(rng.consumed(), 0);
    assert_eq!(r.species.len(), 1);
    assert_eq!(r.species[0].species, Species::EAq);
    assert_eq!(r.species[0].position, Vec3::new(1.5, -2.0, 0.25));
    assert_eq!(r.species[0].time.ps(), 1.0);
    assert_eq!(r.stats.events, [0, 0, 0, 1]);
}

#[test]
fn draw_counts_per_event_follow_the_documented_order() {
    let engine = PhysChemEngine::option3()
        .unwrap()
        .with_recombination_enabled(false);
    // excitation level 0 (A^1B_1): channel u = 0.5 -> 0.35 <= 0.5 < 1.0 -> dissociation (OH + H): 1 + 4 draws
    let mut rng = ScriptedRng::new(&[0.5, 0.3, 0.4, 0.5, 0.6]);
    let r = engine
        .convert_history(&[ev(PhysChemKind::Excitation, 0, ORIGIN)], &mut rng)
        .unwrap();
    assert_eq!(rng.consumed(), 5);
    assert_eq!(tags(&r.species), vec![Species::Oh, Species::H]);
    // the same level, u = 0.2 -> relaxation: one draw, no species, 8.22 eV booked by the chemistry-stage decay
    let mut rng = ScriptedRng::new(&[0.2]);
    let r = engine
        .convert_history(&[ev(PhysChemKind::Excitation, 0, ORIGIN)], &mut rng)
        .unwrap();
    assert_eq!(rng.consumed(), 1);
    assert!(r.species.is_empty());
    assert_eq!(r.stats.decay_energy_ev, 8.22);
    // ionisation, recombination disabled: u_rec (always drawn) + channel u + displacer (u + 4 + 4) = 11
    let mut rng = ScriptedRng::new(&[0.5; 11]);
    let r = engine
        .convert_history(&[ev(PhysChemKind::Ionisation, 2, ORIGIN)], &mut rng)
        .unwrap();
    assert_eq!(rng.consumed(), 11);
    assert_eq!(tags(&r.species), vec![Species::H3Op, Species::Oh]);
    // attachment: channel u + triple(0.8) + triple(1.1) for the single OH = 1 + 8
    let mut rng = ScriptedRng::new(&[0.5; 9]);
    let r = engine
        .convert_history(&[ev(PhysChemKind::Attachment, 0, ORIGIN)], &mut rng)
        .unwrap();
    assert_eq!(rng.consumed(), 9);
    assert_eq!(
        tags(&r.species),
        vec![Species::H2, Species::Ohm, Species::Oh]
    );
}

#[test]
fn recombination_decision_follows_the_onsager_probability() {
    let engine = PhysChemEngine::option3().unwrap();
    let events = [
        ev(PhysChemKind::Solvation, 0, [1.0, 0.0, 0.0]),
        ev(PhysChemKind::Ionisation, 0, ORIGIN),
    ];
    // p(1 nm) = 0.50916: u_rec = 0.50 recombines. H2Ovib channel u = 0.2 (cumulative 0.1365, 0.494) -> OH + H.
    let mut rng = ScriptedRng::new(&[0.50, 0.20, 0.3, 0.4, 0.5, 0.6]);
    let r = engine.convert_history(&events, &mut rng).unwrap();
    assert_eq!(rng.consumed(), 6); // u_rec + channel + triple(2.4)
    assert_eq!(
        tags(&r.species),
        vec![Species::Oh, Species::H],
        "the e_aq is consumed"
    );
    assert_eq!(r.stats.recombinations, 1);
    assert_eq!(r.stats.holes_with_candidate, 1);
    let ovib = engine.channel_table().recombined_index();
    assert_eq!(r.stats.channel_counts[ovib], vec![0, 1, 0, 0]);
    assert_eq!(r.stats.species_counts[Species::EAq.index()], 0);

    // u_rec = 0.52 > p: no recombination, normal Ioni5 decay (1 + 1 + 9 draws)
    let mut rng = ScriptedRng::new(&[0.52, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);
    let r = engine.convert_history(&events, &mut rng).unwrap();
    assert_eq!(rng.consumed(), 11);
    assert_eq!(
        tags(&r.species),
        vec![Species::EAq, Species::H3Op, Species::Oh]
    );
    assert_eq!(r.stats.recombinations, 0);
    assert_eq!(r.stats.holes_with_candidate, 1);
}

#[test]
fn the_nearest_alive_electron_is_consumed_first() {
    let engine = PhysChemEngine::option3().unwrap();
    // two electrons at 2 nm (listed first) and 1 nm; two holes at the origin
    let events = [
        ev(PhysChemKind::Solvation, 0, [2.0, 0.0, 0.0]),
        ev(PhysChemKind::Solvation, 0, [0.0, 1.0, 0.0]),
        ev(PhysChemKind::Ionisation, 0, ORIGIN),
        ev(PhysChemKind::Ionisation, 1, ORIGIN),
    ];
    // hole 1: u_rec 0.01 < p(1 nm) -> recombines with the 1 nm electron; H2Ovib channel u 0.2 -> OH + H (4 draws)
    // hole 2: u_rec 0.40 > p(2 nm) = 0.2994 -> no recombination; Ioni4 decay (1 + 9 draws)
    let mut script = vec![0.01, 0.2, 0.3, 0.4, 0.5, 0.6];
    script.extend([0.40, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);
    let mut rng = ScriptedRng::new(&script);
    let r = engine.convert_history(&events, &mut rng).unwrap();
    assert_eq!(rng.consumed(), script.len());
    let eaq: Vec<&SpeciesInstance> = r
        .species
        .iter()
        .filter(|s| s.species == Species::EAq)
        .collect();
    assert_eq!(eaq.len(), 1);
    assert_eq!(
        eaq[0].position,
        Vec3::new(2.0, 0.0, 0.0),
        "the 1 nm electron was the one consumed"
    );
    assert_eq!(r.stats.recombinations, 1);
    assert_eq!(r.stats.holes_with_candidate, 2);

    // with u_rec small for both holes the 2 nm electron is consumed by the second hole
    let mut script = vec![0.01, 0.2, 0.3, 0.4, 0.5, 0.6, 0.01, 0.2, 0.3, 0.4, 0.5, 0.6];
    script.extend([0.5; 4]);
    let mut rng = ScriptedRng::new(&script);
    let r = engine.convert_history(&events, &mut rng).unwrap();
    assert_eq!(r.stats.recombinations, 2);
    assert!(r.species.iter().all(|s| s.species != Species::EAq));
}

#[test]
fn search_radius_is_ten_onsager_radii_and_a_coincident_electron_always_recombines() {
    let engine = PhysChemEngine::option3().unwrap();
    let radius = engine.params().recombination.search_radius.nm();
    assert!((radius - 7.116353983042855).abs() < 1e-9);
    let draws = vec![0.999; 12];
    for (distance, candidate) in [(radius - 0.01, true), (radius + 0.01, false)] {
        let events = [
            ev(PhysChemKind::Solvation, 0, [distance, 0.0, 0.0]),
            ev(PhysChemKind::Ionisation, 0, ORIGIN),
        ];
        let mut rng = ScriptedRng::new(&draws);
        let r = engine.convert_history(&events, &mut rng).unwrap();
        assert_eq!(
            r.stats.holes_with_candidate,
            u64::from(candidate),
            "distance {distance}"
        );
        // u_rec = 0.999 > p(7.1 nm) = 0.095: never recombines at this distance
        assert_eq!(r.stats.recombinations, 0);
    }
    // d == 0: probability 1, recombines for any u_rec < 1
    let events = [
        ev(PhysChemKind::Solvation, 0, ORIGIN),
        ev(PhysChemKind::Ionisation, 0, ORIGIN),
    ];
    let mut rng = ScriptedRng::new(&[0.999999, 0.2, 0.3, 0.4, 0.5, 0.6]);
    let r = engine.convert_history(&events, &mut rng).unwrap();
    assert_eq!(r.stats.recombinations, 1);
}

#[test]
fn the_recombination_switch_changes_only_the_decay_state_that_is_scanned() {
    let on = PhysChemEngine::option3().unwrap();
    let off = PhysChemEngine::option3()
        .unwrap()
        .with_recombination_enabled(false);
    let events = [
        ev(PhysChemKind::Solvation, 0, ORIGIN),
        ev(PhysChemKind::Ionisation, 0, ORIGIN),
    ];
    // u_rec = 0.01 < p(0) = 1 is drawn first in both runs; then the channel number 0.5.
    let script = [0.01, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5];
    let mut a = ScriptedRng::new(&script);
    let mut b = ScriptedRng::new(&script);
    let ra = on.convert_history(&events, &mut a).unwrap();
    let rb = off.convert_history(&events, &mut b).unwrap();
    // on: recombined, H2Ovib cumulative 0.1365, 0.494, 0.65, 1.0 -> u = 0.5 selects O + 2 H (two triples): 1 + 1 + 8 draws
    assert_eq!(ra.stats.recombinations, 1);
    assert_eq!(tags(&ra.species), vec![Species::O, Species::H, Species::H]);
    assert_eq!(a.consumed(), 10);
    // off: Ioni5, H3O+ + OH: 1 + 1 + 9 draws, and the solvated electron survives
    assert_eq!(rb.stats.recombinations, 0);
    assert_eq!(
        tags(&rb.species),
        vec![Species::EAq, Species::H3Op, Species::Oh]
    );
    assert_eq!(b.consumed(), 11);
}

#[test]
fn an_auto_ionisation_electron_is_a_candidate_for_later_holes() {
    let engine = PhysChemEngine::option3().unwrap();
    // B^1A_1 (level 1): channel u = 0.5 selects the auto-ionisation channel (cumulative 0.175, 0.2075, 0.7075).
    let prefix = [
        0.5, 0.31, 0.32, 0.33, 0.34, 0.71, 0.41, 0.42, 0.43, 0.44, 0.45, 0.46, 0.47, 0.48, 0.49,
        0.51, 0.52, 0.53, 0.54,
    ];
    let first = engine
        .convert_history(
            &[ev(PhysChemKind::Excitation, 1, ORIGIN)],
            &mut ScriptedRng::new(&prefix),
        )
        .unwrap();
    assert_eq!(
        first.stats.channel_counts[engine
            .channel_table()
            .state_index(PhysChemKind::Excitation, 1)
            .unwrap()][2],
        1
    );
    let eaq = first
        .species
        .iter()
        .find(|s| s.species == Species::EAq)
        .expect("auto-ionisation makes an e_aq");
    // a hole placed exactly on that electron recombines for any u_rec < 1 (d == 0): the electron was available
    let events = [
        ev(PhysChemKind::Excitation, 1, ORIGIN),
        ev(
            PhysChemKind::Ionisation,
            4,
            [eaq.position.x, eaq.position.y, eaq.position.z],
        ),
    ];
    let mut script = prefix.to_vec();
    script.extend([0.999, 0.2, 0.3, 0.4, 0.5, 0.6]);
    let r = engine
        .convert_history(&events, &mut ScriptedRng::new(&script))
        .unwrap();
    assert_eq!(r.stats.recombinations, 1);
    assert_eq!(
        r.stats.species_counts[Species::EAq.index()],
        0,
        "the auto-ionisation electron was consumed"
    );
}

#[test]
fn species_are_born_at_one_picosecond_and_creation_order_is_solvation_first() {
    let engine = PhysChemEngine::option3()
        .unwrap()
        .with_recombination_enabled(false);
    let events = [
        ev(PhysChemKind::Excitation, 0, [10.0, 0.0, 0.0]),
        ev(PhysChemKind::Solvation, 0, [0.0, 5.0, 0.0]),
    ];
    let mut rng = ScriptedRng::new(&[0.5; 5]);
    let r = engine.convert_history(&events, &mut rng).unwrap();
    assert_eq!(
        r.species[0].species,
        Species::EAq,
        "solvated electrons exist when the stage starts"
    );
    assert_eq!(
        tags(&r.species),
        vec![Species::EAq, Species::Oh, Species::H]
    );
    assert!(r.species.iter().all(|s| s.time.ps() == 1.0));
}

#[test]
fn the_event_list_is_not_modified() {
    let engine = PhysChemEngine::option3().unwrap();
    let events = vec![
        ev(PhysChemKind::Ionisation, 0, [1.0, 2.0, 3.0]),
        ev(PhysChemKind::Solvation, 0, ORIGIN),
    ];
    let copy = events.clone();
    let mut rng = RngStreams::new(5).stream(0, Stage::PhysChem).unwrap();
    engine.convert_history(&events, &mut rng).unwrap();
    assert_eq!(events, copy);
    let deposited: f64 = events.iter().map(|e| e.deposited_energy.ev()).sum();
    assert_eq!(
        deposited,
        copy.iter().map(|e| e.deposited_energy.ev()).sum::<f64>()
    );
}

fn bits(r: &dna_chemistry::HistoryChemistry) -> Vec<u64> {
    let mut v = Vec::new();
    for s in &r.species {
        v.extend([
            s.species.index() as u64,
            s.position.x.to_bits(),
            s.position.y.to_bits(),
            s.position.z.to_bits(),
            s.time.ps().to_bits(),
        ]);
    }
    v.extend(r.stats.events);
    v.extend(r.stats.channel_counts.iter().flatten().copied());
    v.extend([
        r.stats.recombinations,
        r.stats.holes_with_candidate,
        r.stats.decay_energy_ev.to_bits(),
    ]);
    v
}

/// A reproducible synthetic cascade of `n` events around the origin, drawn from the Physics stream of the history.
fn synthetic_events(streams: &RngStreams, history: u64) -> Vec<PhysChemEvent> {
    use dna_core::sampling::uniform;
    let mut rng = streams.stream(history, Stage::Physics).unwrap();
    (0..80)
        .map(|_| {
            let u = uniform(&mut rng);
            let kind = if u < 0.35 {
                PhysChemKind::Ionisation
            } else if u < 0.55 {
                PhysChemKind::Excitation
            } else if u < 0.57 {
                PhysChemKind::Attachment
            } else {
                PhysChemKind::Solvation
            };
            let index = match kind {
                PhysChemKind::Excitation | PhysChemKind::Ionisation => {
                    (uniform(&mut rng) * 5.0) as u8
                }
                _ => 0,
            };
            ev(
                kind,
                index,
                [
                    20.0 * (uniform(&mut rng) - 0.5),
                    20.0 * (uniform(&mut rng) - 0.5),
                    20.0 * (uniform(&mut rng) - 0.5),
                ],
            )
        })
        .collect()
}

#[test]
fn species_lists_are_bitwise_identical_for_any_thread_count() {
    let engine = PhysChemEngine::option3().unwrap();
    let streams = RngStreams::new(20261015);
    let run = |threads: usize| -> Vec<Vec<u64>> {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                (0..300u64)
                    .into_par_iter()
                    .map(|h| {
                        let events = synthetic_events(&streams, h);
                        let mut rng = streams.stream(h, Stage::PhysChem).unwrap();
                        bits(&engine.convert_history(&events, &mut rng).unwrap())
                    })
                    .collect()
            })
    };
    let one = run(1);
    assert_eq!(one, run(2));
    assert_eq!(one, run(8));
    assert!(
        one.iter().any(|b| b.len() > 40),
        "the synthetic histories are not trivial"
    );
    // different histories differ, the same history repeats
    assert_ne!(one[0], one[1]);
}
