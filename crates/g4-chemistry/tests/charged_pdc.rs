//! Charged option3 PDC source fidelity, not a general Coulomb first-passage law.
//! Independent mpmath 1.3.0 / 80-digit anchors use the literal Geant4 11.4.1
//! v/alpha/sinh/coth formulas (G4DNAIRT.cc:285-405). The fixture records inputs,
//! formulas, units and tolerances. Tests use no production special functions
//! to build expectations. Statistical protocol was fixed before the first run:
//! 50,000 histories per ID, seed 2026102602, four CDF points and no-reaction
//! mass, each within an absolute four-sigma binomial bound (55 comparisons).

mod common;

use common::{fixture, isolated_pair, option3_parameters, reaction_time_bits};
use g4_chemistry::ChemistryError;
use g4_chemistry::irt::{ReactionTime, sample_independent_reaction_time};
use g4_chemistry::molecule::MoleculePair;
use g4_chemistry::reaction_parameters::TwoBodyReactionParameters as Parameters;
use g4_chemistry::reactions::{ObservedRate, Reaction, ReactionParticipant, ReactionTable};
use g4_chemistry::species::{SpeciesProps, SpeciesTable};
use g4_core::rng::{RngStreams, Stage, u64_to_open01};
use g4_core::testing::ScriptedRng;
use g4_core::units::{Diffusivity, Length};
use serde_json::Value;

const N: usize = 50_000;
const SEED: u64 = 2_026_102_602;

fn value(row: &Value, key: &str) -> f64 {
    number(&row[key])
}

fn number(number: &Value) -> f64 {
    number.as_str().unwrap().parse().unwrap()
}

fn close(actual: f64, expected: f64, relative: f64) {
    assert!(
        (actual - expected).abs() <= relative * expected.abs(),
        "{actual:.17e} != {expected:.17e} (relative tolerance {relative})"
    );
}

fn sample(
    p: Parameters,
    distance: f64,
    rng: &mut ScriptedRng,
) -> Result<ReactionTime, ChemistryError> {
    let [a, b] = isolated_pair(p, distance, 0);
    sample_independent_reaction_time(MoleculePair::new(&a, &b).unwrap(), &p, rng)
}

fn elapsed(result: Result<ReactionTime, ChemistryError>) -> f64 {
    let ReactionTime::After(time) = result.unwrap() else {
        panic!("expected a positive elapsed time");
    };
    assert!(time.ps().is_finite() && time.ps() > 0.0);
    time.ps()
}

fn inputs(id: u16) -> (Reaction, [SpeciesProps; 2]) {
    let reaction = ReactionTable::option3().unwrap().get(id).unwrap().clone();
    let species = SpeciesTable::option3().unwrap();
    let props = reaction.reactants.map(|reactant| match reactant {
        ReactionParticipant::Mobile(s) => *species.get(s),
        _ => panic!("requires mobile species"),
    });
    (reaction, props)
}

fn assert_anchors(p: Parameters, row: &Value) {
    close(
        p.effective_radius().nm(),
        value(row, "effective_radius_nm"),
        2e-13,
    );
    close(
        p.contact_probability(),
        value(row, "contact_probability"),
        2e-13,
    );
    close(p.onsager_radius().nm(), value(row, "onsager_nm"), 2e-13);
    let distance = value(row, "distance_nm");
    let winf = value(row, "winf");
    let samples = row["samples"].as_array().unwrap();
    let fallback = samples.last().unwrap();
    for expected in samples {
        let proposal = value(expected, "proposal");
        let acceptance = value(expected, "acceptance");
        let offset = 1e-7_f64.min(acceptance * 0.01);
        if acceptance > 4.0 * u64_to_open01(0) {
            let mut rng = ScriptedRng::new(&[winf / 2.0, proposal, acceptance - offset]);
            close(
                elapsed(sample(p, distance, &mut rng)),
                value(expected, "elapsed_ps"),
                2e-11,
            );
            assert_eq!(rng.consumed(), 3);
        }
        // Includes source ID 45's lower proposal, whose acceptance is below
        // the smallest open uniform. It must reject, then use a fresh proposal.
        let mut rng = ScriptedRng::new(&[
            winf / 2.0,
            proposal,
            (acceptance + offset).max(u64_to_open01(0)),
            value(fallback, "proposal"),
            value(fallback, "acceptance") / 2.0,
        ]);
        close(
            elapsed(sample(p, distance, &mut rng)),
            value(fallback, "elapsed_ps"),
            2e-11,
        );
        assert_eq!(rng.consumed(), 5);
    }
    // Equality follows the continuum b=0 mixture, not the contact probability.
    let sigma = p.reaction_radius().nm();
    let gate = p.observed_rate().nm3_per_ps() / p.pdc().unwrap().diffusion_rate().nm3_per_ps();
    let mut rng = ScriptedRng::new(&[(gate + p.contact_probability()) / 2.0, 0.25, 0.5]);
    close(
        elapsed(sample(p, sigma, &mut rng)),
        value(row, "equality_time_ps"),
        2e-11,
    );
    assert_eq!(rng.consumed(), 3);
}

#[test]
fn all_eleven_ids_match_independent_source_formula_and_acceptance_anchors() {
    let data = fixture("charged_pdc_anchors.json");
    let rows = data["option3"].as_array().unwrap();
    assert_eq!(rows.len(), 11);
    for (id, row) in (42..=52).zip(rows) {
        assert_eq!(row["reaction_id"].as_u64(), Some(u64::from(id)));
        assert_anchors(option3_parameters(id), row);
    }
}

#[test]
fn contact_exact_radius_and_adjacent_radii_keep_the_draw_contract_for_both_signs() {
    for id in 42..=52 {
        let p = option3_parameters(id);
        let sigma = p.reaction_radius().nm();
        for distance in [0.0, sigma / 2.0, f64::from_bits(sigma.to_bits() - 1)] {
            for (draw, expected) in [
                (p.contact_probability() / 2.0, ReactionTime::Immediate),
                (
                    (1.0 + p.contact_probability()) / 2.0,
                    ReactionTime::NoReaction,
                ),
            ] {
                let mut rng = ScriptedRng::new(&[draw]);
                assert_eq!(sample(p, distance, &mut rng).unwrap(), expected, "ID {id}");
                assert_eq!(rng.consumed(), 1);
            }
        }
        let winf = p.observed_rate().nm3_per_ps() / p.pdc().unwrap().diffusion_rate().nm3_per_ps();
        for distance in [sigma, f64::from_bits(sigma.to_bits() + 1)] {
            let mut rng = ScriptedRng::new(&[(1.0 + winf) / 2.0]);
            assert_eq!(
                sample(p, distance, &mut rng).unwrap(),
                ReactionTime::NoReaction
            );
            assert_eq!(rng.consumed(), 1);
        }
        // Just outside contact retains a nonzero transformed gap, even when
        // independently rounded transformed lengths would coincide.
        let mut rng = ScriptedRng::new(&[winf / 2.0, u64_to_open01(u64::MAX), 0.001]);
        elapsed(sample(p, f64::from_bits(sigma.to_bits() + 1), &mut rng));
        assert_eq!(rng.consumed(), 3);
    }
}

#[test]
fn no_reaction_gate_uses_transformed_separation_and_preserves_the_far_field() {
    let data = fixture("charged_pdc_anchors.json");
    for row in data["option3"].as_array().unwrap() {
        let p = option3_parameters(row["reaction_id"].as_u64().unwrap() as u16);
        let winf = value(row, "winf");
        let mut rng = ScriptedRng::new(&[winf + 1e-7]);
        assert_eq!(
            sample(p, value(row, "distance_nm"), &mut rng).unwrap(),
            ReactionTime::NoReaction
        );
        assert_eq!(rng.consumed(), 1);
        let mut rng = ScriptedRng::new(&[0.5]);
        assert_eq!(
            sample(p, 1e200, &mut rng).unwrap(),
            ReactionTime::NoReaction
        );
        assert_eq!(rng.consumed(), 1);
    }
}

#[test]
fn neutral_limits_and_identical_reactant_factor_match_literal_source_anchors() {
    let data = fixture("charged_pdc_anchors.json");
    for row in data["synthetic_neutral_limits"].as_array().unwrap() {
        let id = row["reaction_id"].as_u64().unwrap() as u16;
        let (mut reaction, mut props) = inputs(id);
        reaction.rate = ObservedRate::Bimolecular {
            per_molar_s: value(row, "observed_per_molar_s"),
        };
        for (i, prop) in props.iter_mut().enumerate() {
            prop.radius = Length::from_nm(number(&row["radii_nm"][i]));
            prop.diffusivity =
                Diffusivity::from_nm2_per_ps(number(&row["diffusivities_nm2_ps"][i]));
            prop.charge = row["charges"][i].as_i64().unwrap() as i8;
        }
        assert_anchors(Parameters::from_species(&reaction, props).unwrap(), row);
    }
}

#[test]
fn fifty_five_registered_cdf_and_infinite_time_mass_checks() {
    let data = fixture("charged_pdc_anchors.json");
    let streams = RngStreams::new(SEED);
    let mut comparisons = 0;
    for row in data["option3"].as_array().unwrap() {
        let id = row["reaction_id"].as_u64().unwrap() as u16;
        let p = option3_parameters(id);
        let times: Vec<_> = row["times_ps"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect();
        let mut counts = [0_usize; 5];
        let [a, b] = isolated_pair(p, value(row, "distance_nm"), 0);
        for index in 0..N {
            let mut rng = streams
                .stream(u64::from(id) * N as u64 + index as u64, Stage::Chemistry)
                .unwrap();
            match sample_independent_reaction_time(MoleculePair::new(&a, &b).unwrap(), &p, &mut rng)
                .unwrap()
            {
                ReactionTime::After(time) => {
                    assert!(time.ps().is_finite() && time.ps() > 0.0);
                    for (count, threshold) in counts.iter_mut().zip(&times) {
                        *count += usize::from(time.ps() <= *threshold);
                    }
                }
                ReactionTime::NoReaction => counts[4] += 1,
                ReactionTime::Immediate => panic!("separated pair, ID {id}"),
            }
        }
        let mut probabilities: Vec<_> = row["cdf"].as_array().unwrap().iter().map(number).collect();
        probabilities.push(value(row, "no_reaction"));
        for (i, (count, probability)) in counts.into_iter().zip(probabilities).enumerate() {
            let error = (count as f64 / N as f64 - probability).abs();
            let sigma = (probability * (1.0 - probability) / N as f64).sqrt();
            assert!(
                error <= 4.0 * sigma,
                "ID {id}, point {i}: {count}/{N}, expected {probability}, |z|={}",
                error / sigma
            );
            comparisons += 1;
        }
    }
    assert_eq!(comparisons, 55);
}

#[test]
fn both_charge_signs_and_aa_are_exchange_and_rng_stream_reproducible() {
    let streams = RngStreams::new(SEED);
    for id in [42, 43, 51] {
        let p = option3_parameters(id);
        let [a, b] = isolated_pair(p, 2.0 * p.reaction_radius().nm(), 0);
        for history in 0..64 {
            let mut forward = streams.stream(history, Stage::Chemistry).unwrap();
            let mut reverse = forward.clone();
            let x = sample_independent_reaction_time(
                MoleculePair::new(&a, &b).unwrap(),
                &p,
                &mut forward,
            )
            .unwrap();
            let y = sample_independent_reaction_time(
                MoleculePair::new(&b, &a).unwrap(),
                &p,
                &mut reverse,
            )
            .unwrap();
            assert_eq!(reaction_time_bits(x), reaction_time_bits(y));
        }
    }
}

#[test]
fn rejection_exhaustion_is_a_typed_failure_and_the_last_attempt_can_succeed() {
    let p = option3_parameters(42);
    let mut draws = vec![0.001];
    for _ in 0..9_999 {
        draws.extend([u64_to_open01(0), 0.5]);
    }
    draws.extend([0.6, 0.01]);
    let mut rng = ScriptedRng::new(&draws);
    elapsed(sample(p, 1.5, &mut rng));
    assert_eq!(rng.consumed(), 20_001);
    draws.truncate(19_999);
    draws.extend([u64_to_open01(0), 0.5]);
    let mut rng = ScriptedRng::new(&draws);
    assert_eq!(
        sample(p, 1.5, &mut rng),
        Err(ChemistryError::IrtConvergence {
            reaction_id: 42,
            attempts: 10_000
        })
    );
    assert_eq!(rng.consumed(), 20_001);
}

#[test]
fn singular_and_unrepresentable_selected_arithmetic_is_typed_never_nonreaction() {
    // A repulsive contact-radius transform too small to represent is rejected
    // during parameter construction, before it can produce invalid rates.
    let (reaction, mut props) = inputs(43);
    for prop in &mut props {
        prop.radius = Length::from_nm(1e-100);
    }
    assert!(matches!(
        Parameters::from_species(&reaction, props),
        Err(ChemistryError::ReactionParameterDomain { .. })
    ));
    // Attraction can make a positive transformed gap too small to represent.
    // The two nonzero physical radii must not be mistaken for exact equality.
    let (mut reaction, mut props) = inputs(42);
    for prop in &mut props {
        prop.radius = Length::from_nm(1e-5);
    }
    let p = Parameters::from_species(&reaction, props).unwrap();
    let mut rng = ScriptedRng::new(&[0.001]);
    assert_eq!(
        sample(p, 4e-5, &mut rng),
        Err(ChemistryError::IrtDomain {
            reaction_id: 42,
            operation: "charged PDC transformed gap"
        })
    );
    assert_eq!(rng.consumed(), 1);
    // A finite positive selected delay can overflow even with valid parameters.
    for prop in &mut props {
        prop.radius = Length::from_nm(5e99);
        prop.diffusivity = Diffusivity::from_nm2_per_ps(5e-201);
    }
    reaction.rate = ObservedRate::Bimolecular {
        per_molar_s: 3.783_822_634_099_930_7e-88,
    };
    let p = Parameters::from_species(&reaction, props).unwrap();
    let mut rng = ScriptedRng::new(&[0.1, 0.6, 0.01]);
    assert_eq!(
        sample(p, 3e100, &mut rng),
        Err(ChemistryError::IrtDomain {
            reaction_id: 42,
            operation: "PDC elapsed time"
        })
    );
    assert_eq!(rng.consumed(), 3);
    // Invalid activation rates remain construction errors for charged pairs.
    let (mut reaction, props) = inputs(43);
    reaction.rate = ObservedRate::Bimolecular { per_molar_s: 1e100 };
    assert!(matches!(
        Parameters::from_species(&reaction, props),
        Err(ChemistryError::InvalidPdcRate { .. })
    ));
}
