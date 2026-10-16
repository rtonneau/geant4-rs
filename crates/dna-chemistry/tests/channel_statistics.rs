//! V8-style test of the channel selection: the sampled channel of every decay state against the table.
//!
//! Acceptance criteria, fixed before the first run:
//! * family: the 6 states with more than one channel (A^1B_1, B^1A_1, Exci3rdLayer, Exci2ndLayer, Exci1stLayer, H2Ovib),
//!   Pearson chi-square with `dof = n_channels - 1`, `N = 400 000` decays each, master seed 20261015, stream
//!   `(seed, state, PhysChem)`;
//! * a state passes if `p > 1e-3 / 6 = 1.67e-4` (Bonferroni, overall false-alarm probability 1e-3);
//! * single-channel states (Ioni1..5, DissociAttachment_ch1) must always give channel 0 and the documented products;
//! * the energy booked by relaxation channels equals (number of relaxations) x (excitation energy of the level) to a
//!   relative 1e-12 (floating-point summation).

mod common;

use common::chi2_pvalue;
use dna_chemistry::PhysChemEngine;
use dna_chemistry::channels::StateKind;
use dna_core::physchem::{PhysChemEvent, PhysChemKind};
use dna_core::rng::{RngStreams, Stage};
use dna_core::species::Species;
use dna_core::testing::ScriptedRng;
use dna_core::units::Energy;
use dna_core::vec3::Vec3;

const N: u64 = 400_000;
const SEED: u64 = 20261015;
const ALPHA: f64 = 1.0e-3 / 6.0;

/// Catch dispatch, product-order, energy, or birth-time regressions in every branch,
/// including rare B1A1 and H2Ovib channels. Expected values come from the independent
/// C++ extraction, rather than the Rust table used by the engine.
#[test]
fn every_cpp_channel_produces_its_products_at_one_picosecond() {
    let cpp = common::fixture("cpp_extract.json");
    let engine = PhysChemEngine::option3().unwrap();
    let mut checked = 0;
    for (si, state) in engine.channel_table().states().iter().enumerate() {
        let mut cumulative = 0.0;
        for (ci, expected) in cpp["states"][&state.name]["channels"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let probability = expected["probability"].as_f64().unwrap();
            let mut draws = vec![0.37; 32];
            draws[0] = cumulative + probability / 2.0;
            cumulative += probability;
            let mut rng = ScriptedRng::new(&draws);
            let result = match state.kind {
                StateKind::Recombined => engine.decay_in_state(si, Vec3::ZERO, &mut rng),
                kind => engine
                    .decay_event(
                        &PhysChemEvent {
                            kind: match kind {
                                StateKind::Excitation => PhysChemKind::Excitation,
                                StateKind::Ionisation => PhysChemKind::Ionisation,
                                StateKind::Attachment => PhysChemKind::Attachment,
                                StateKind::Recombined => unreachable!(),
                            },
                            level_or_shell: state.index,
                            position: Vec3::ZERO,
                            deposited_energy: Energy::from_ev(0.0),
                            parent_history: 0,
                        },
                        &mut rng,
                    )
                    .unwrap(),
            };
            assert_eq!((result.state, result.channel), (si, ci), "{}", state.name);
            let products: Vec<_> = result
                .species
                .iter()
                .map(|s| s.species.chemistry_name())
                .collect();
            let expected_products: Vec<_> = expected["products"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p.as_str().unwrap())
                .collect();
            assert_eq!(products, expected_products, "{}/{}", state.name, ci);
            assert_eq!(
                result.decay_energy.ev(),
                expected["energy_ev"].as_f64().unwrap()
            );
            assert!(result.species.iter().all(|s| s.time.ps() == 1.0));
            checked += 1;
        }
    }
    assert_eq!(checked, 23);
}

#[test]
fn the_selected_channel_follows_the_table() {
    let engine = PhysChemEngine::option3().unwrap();
    let streams = RngStreams::new(SEED);
    let mut n_tests = 0;
    for (si, state) in engine.channel_table().states().iter().enumerate() {
        if state.channels.len() < 2 {
            continue;
        }
        n_tests += 1;
        let mut rng = streams.stream(si as u64, Stage::PhysChem).unwrap();
        let mut counts = vec![0u64; state.channels.len()];
        for _ in 0..N {
            counts[engine.decay_in_state(si, Vec3::ZERO, &mut rng).channel] += 1;
        }
        let chi2: f64 = state
            .channels
            .iter()
            .zip(&counts)
            .map(|(c, &o)| {
                let e = c.probability * N as f64;
                (o as f64 - e).powi(2) / e
            })
            .sum();
        let p = chi2_pvalue(chi2, state.channels.len() - 1);
        eprintln!(
            "{:>14}: chi2 = {chi2:7.3} (dof {}), p = {p:.4}",
            state.name,
            state.channels.len() - 1
        );
        assert!(
            p > ALPHA,
            "{}: chi2 = {chi2}, p = {p} <= {ALPHA}",
            state.name
        );
    }
    assert_eq!(n_tests, 6);
}

#[test]
fn single_channel_states_are_deterministic_and_the_event_maps_to_the_right_state() {
    let engine = PhysChemEngine::option3().unwrap();
    let streams = RngStreams::new(SEED);
    let mut rng = streams.stream(99, Stage::PhysChem).unwrap();
    let names = ["Ioni5", "Ioni4", "Ioni3", "Ioni2", "Ioni1"];
    for (shell, name) in names.iter().enumerate() {
        let ev = PhysChemEvent {
            kind: PhysChemKind::Ionisation,
            level_or_shell: shell as u8,
            position: Vec3::new(1.0, 2.0, 3.0),
            deposited_energy: Energy::from_ev(10.0),
            parent_history: 0,
        };
        for _ in 0..1000 {
            let d = engine.decay_event(&ev, &mut rng).unwrap();
            assert_eq!(engine.channel_table().states()[d.state].name, *name);
            assert_eq!(d.channel, 0);
            let tags: Vec<Species> = d.species.iter().map(|s| s.species).collect();
            assert_eq!(tags, vec![Species::H3Op, Species::OH]);
            assert_eq!(d.decay_energy.ev(), 0.0);
            assert!(d.species.iter().all(|s| s.time.ps() == 1.0));
        }
    }
    let att = PhysChemEvent {
        kind: PhysChemKind::Attachment,
        level_or_shell: 0,
        position: Vec3::ZERO,
        deposited_energy: Energy::from_ev(5.0),
        parent_history: 0,
    };
    let d = engine.decay_event(&att, &mut rng).unwrap();
    assert_eq!(
        d.species.iter().map(|s| s.species).collect::<Vec<_>>(),
        vec![Species::H2, Species::OHm, Species::OH]
    );
    assert_eq!(
        engine.channel_table().states()[d.state].kind,
        StateKind::Attachment
    );
}

#[test]
fn relaxation_channels_book_the_excitation_energy_of_their_level() {
    let engine = PhysChemEngine::option3().unwrap();
    let streams = RngStreams::new(SEED);
    let mut rng = streams.stream(7, Stage::PhysChem).unwrap();
    let expected = [8.22, 10.0, 11.24, 12.61, 13.77];
    for (level, e_exc) in expected.iter().enumerate() {
        let ev = PhysChemEvent {
            kind: PhysChemKind::Excitation,
            level_or_shell: level as u8,
            position: Vec3::ZERO,
            deposited_energy: Energy::from_ev(*e_exc),
            parent_history: 0,
        };
        let mut sum = 0.0;
        let mut n_relax = 0u64;
        for _ in 0..20_000 {
            let d = engine.decay_event(&ev, &mut rng).unwrap();
            let ch = &engine.channel_table().states()[d.state].channels[d.channel];
            if ch.products.is_empty() {
                n_relax += 1;
                assert_eq!(d.decay_energy.ev(), *e_exc);
            } else {
                assert_eq!(d.decay_energy.ev(), 0.0);
            }
            sum += d.decay_energy.ev();
        }
        // a floating-point sum of up to 20 000 equal terms: relative 1e-12, not bitwise
        let exact = n_relax as f64 * e_exc;
        assert!(
            (sum - exact).abs() <= 1e-12 * exact.max(1.0),
            "level {level}: {sum} vs {exact}"
        );
    }
}

#[test]
fn unknown_indices_are_errors_not_panics() {
    let engine = PhysChemEngine::option3().unwrap();
    let mut rng = RngStreams::new(1).stream(0, Stage::PhysChem).unwrap();
    let ev = PhysChemEvent {
        kind: PhysChemKind::Excitation,
        level_or_shell: 5,
        position: Vec3::ZERO,
        deposited_energy: Energy::from_ev(1.0),
        parent_history: 0,
    };
    assert!(engine.decay_event(&ev, &mut rng).is_err());
    assert!(engine.convert_history(&[ev], &mut rng).is_err());
}
