//! Geant4 11.4.1 `G4DNAIRT.cc:286-311` TDC branch checks.
//! Numeric tolerance: relative 2e-12. Statistical checks use 20,000 fixed-seed
//! pairs and five binomial standard errors, set before running the tests.
//! Infinity is counted explicitly; finite-time counts use the entire ensemble.

use dna_chemistry::ChemistryError;
use dna_chemistry::irt::{ReactionTime, sample_independent_reaction_time};
use dna_chemistry::molecule::{Molecule, MoleculeId, MoleculePair};
use dna_chemistry::reaction_parameters::TwoBodyReactionParameters as Parameters;
use dna_chemistry::reactions::{ObservedRate, ReactionParticipant, ReactionTable};
use dna_chemistry::species::SpeciesTable;
use dna_chemistry::{Species, SpeciesInstance};
use dna_core::rng::{RngStreams, Stage, u64_to_open01};
use dna_core::testing::ScriptedRng;
use dna_core::units::{Diffusivity, Time};
use dna_core::vec3::Vec3;
use rand::Rng;

fn parameters(id: u16) -> Parameters {
    Parameters::derive(
        ReactionTable::option3().unwrap().get(id).unwrap(),
        &SpeciesTable::option3().unwrap(),
    )
    .unwrap()
}

fn molecules(p: Parameters, distance: f64, time: f64) -> [Molecule; 2] {
    std::array::from_fn(|i| {
        Molecule::new(
            MoleculeId(i as u64),
            SpeciesInstance {
                species: p.reactants()[i],
                position: Vec3::new(i as f64 * distance, 0.0, 0.0),
                time: Time::from_ps(time),
            },
        )
    })
}

fn sample(p: Parameters, distance: f64, rng: &mut impl Rng) -> ReactionTime {
    let [a, b] = molecules(p, distance, 0.0);
    sample_independent_reaction_time(MoleculePair::new(&a, &b).unwrap(), &p, rng).unwrap()
}

fn elapsed(outcome: ReactionTime) -> f64 {
    let ReactionTime::After(time) = outcome else {
        panic!("expected a finite positive elapsed time, got {outcome:?}");
    };
    assert!(time.ps().is_finite() && time.ps() > 0.0);
    time.ps()
}

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 2e-12 * expected.abs(),
        "{actual:.17e} != {expected:.17e}"
    );
}

#[test]
fn strict_contact_returns_immediate_without_a_draw_before_the_charge_transform() {
    for id in [1, 2, 8, 9] {
        let p = parameters(id);
        let radius = p.effective_radius().nm();
        let mut rng = ScriptedRng::new(&[]);
        for distance in [0.0, 0.5 * radius, f64::from_bits(radius.to_bits() - 1)] {
            assert_eq!(sample(p, distance, &mut rng), ReactionTime::Immediate);
        }
        assert_eq!(rng.consumed(), 0);
    }
}

#[test]
fn neutral_equality_uses_the_continuum_draw_and_has_zero_elapsed_time() {
    let p = parameters(1);
    for uniform in [u64_to_open01(0), 0.25, u64_to_open01(u64::MAX)] {
        let mut rng = ScriptedRng::new(&[uniform]);
        assert_eq!(
            sample(p, p.effective_radius().nm(), &mut rng),
            ReactionTime::Immediate
        );
        assert_eq!(rng.consumed(), 1);
    }
}

#[test]
fn aa_inverse_first_passage_uses_relative_diffusion_and_one_draw() {
    let p = parameters(1);
    let mut rng = ScriptedRng::new(&[0.25]);
    let time = elapsed(sample(p, 2.0 * p.effective_radius().nm(), &mut rng));
    // Source H+H: sigma=0.0949531066936965 nm, D=0.014 nm^2/ps.
    // erfc^-1(1/2)=0.4769362762044699, so t=sigma^2/(4 D z^2).
    close(time, 0.707_798_466_255_987);
    assert_eq!(rng.consumed(), 1);
}

#[test]
fn above_ultimate_probability_is_no_reaction_with_one_draw() {
    let p = parameters(2);
    for uniform in [0.75, u64_to_open01(u64::MAX)] {
        let mut rng = ScriptedRng::new(&[uniform]);
        assert_eq!(
            sample(p, 2.0 * p.effective_radius().nm(), &mut rng),
            ReactionTime::NoReaction
        );
        assert_eq!(rng.consumed(), 1);
    }
}

#[test]
fn endpoint_adjacent_draws_produce_finite_times_on_both_sides_of_the_tail() {
    let p = parameters(1);
    let distance = 2.0 * p.effective_radius().nm();
    let mut lo = ScriptedRng::new(&[u64_to_open01(0)]);
    let mut hi = ScriptedRng::new(&[0.5 - f64::EPSILON / 2.0]);
    let earliest = elapsed(sample(p, distance, &mut lo));
    let latest = elapsed(sample(p, distance, &mut hi));
    assert!(earliest < 0.01 && latest > 1e28);
    assert_eq!((lo.consumed(), hi.consumed()), (1, 1));

    // One representable step beyond contact must retain its small positive time.
    let just_outside = f64::from_bits(p.effective_radius().nm().to_bits() + 1);
    let mut rng = ScriptedRng::new(&[0.25]);
    assert!(elapsed(sample(p, just_outside, &mut rng)) < 1e-28);
}

#[test]
fn charged_ab_uses_the_source_effective_distance_in_probability_and_time() {
    let p = parameters(9);
    // Independently evaluated source transform at r=2 nm, rc=-0.7116337048868 nm.
    let winf = 1.011_652_407_973_75 / 2.376_873_400_022_39;
    // erfc(1/2) from the independent special-function fixture.
    let mut rng = ScriptedRng::new(&[winf * 0.479_500_122_186_953_5]);
    close(elapsed(sample(p, 2.0, &mut rng)), 126.275_633_951_916);
    assert_eq!(rng.consumed(), 1);

    // This draw lies between transformed and untransformed ultimate probabilities.
    let mut rng = ScriptedRng::new(&[0.47]);
    assert_eq!(sample(p, 2.0, &mut rng), ReactionTime::NoReaction);
}

#[test]
fn charged_equality_is_not_the_strict_contact_branch() {
    let p = parameters(9);
    let distance = p.effective_radius().nm();
    let mut reacts = ScriptedRng::new(&[0.25]);
    elapsed(sample(p, distance, &mut reacts));
    let mut escapes = ScriptedRng::new(&[0.99]);
    assert_eq!(sample(p, distance, &mut escapes), ReactionTime::NoReaction);
    assert_eq!((reacts.consumed(), escapes.consumed()), (1, 1));
}

#[test]
fn repulsive_transform_does_not_add_a_second_contact_branch() {
    let p = parameters(8);
    // Raw r > sigma, but the repulsive transform gives r_eff < sigma.
    // The source still samples its squared inverse law, with Winf > 1.
    let distance = 1.1 * p.effective_radius().nm();
    for uniform in [0.25, u64_to_open01(u64::MAX)] {
        let mut rng = ScriptedRng::new(&[uniform]);
        elapsed(sample(p, distance, &mut rng));
        assert_eq!(rng.consumed(), 1);
    }
}

#[test]
fn exact_coincidence_preserves_the_sources_one_picometre_substitution() {
    let table = ReactionTable::option3().unwrap();
    let mut reaction = table.get(1).unwrap().clone();
    reaction.rate = ObservedRate::Bimolecular { per_molar_s: 1e6 };
    let p = Parameters::derive(&reaction, &SpeciesTable::option3().unwrap()).unwrap();
    assert!(p.effective_radius().nm() < 1e-3);
    let mut at_zero = ScriptedRng::new(&[0.75]);
    let mut at_substitute = at_zero.clone();
    assert_eq!(sample(p, 0.0, &mut at_zero), ReactionTime::NoReaction);
    assert_eq!(
        sample(p, 1e-3, &mut at_substitute),
        ReactionTime::NoReaction
    );
    assert_eq!((at_zero.consumed(), at_substitute.consumed()), (1, 1));
}

#[test]
fn outcomes_are_elapsed_and_invariant_under_ab_or_aa_exchange() {
    for id in [1, 2, 8, 9] {
        let p = parameters(id);
        let distance = 4.0 * p.effective_radius().nm();
        let [a, b] = molecules(p, distance, 9876.0);
        let [earlier_a, earlier_b] = molecules(p, distance, 1.0);
        let mut rng = RngStreams::new(104).stream(17, Stage::Chemistry).unwrap();
        let mut reverse_rng = rng.clone();
        let mut earlier_rng = rng.clone();
        let draw = |first, second, rng: &mut _| {
            sample_independent_reaction_time(MoleculePair::new(first, second).unwrap(), &p, rng)
                .unwrap()
        };
        for _ in 0..200 {
            let result = draw(&a, &b, &mut rng);
            assert_eq!(result, draw(&b, &a, &mut reverse_rng));
            assert_eq!(result, draw(&earlier_a, &earlier_b, &mut earlier_rng));
        }
    }
}

#[test]
fn fixed_seed_nonoverlap_cdf_includes_the_probability_mass_at_infinity() {
    const N: usize = 20_000;
    for id in [1, 2, 9] {
        let p = parameters(id);
        let (distance, effective) = if id == 9 {
            (2.0, 2.376_873_400_022_39)
        } else {
            let r = 2.0 * p.effective_radius().nm();
            (r, r)
        };
        let sigma = p.effective_radius().nm();
        let winf = sigma / effective;
        let threshold = (effective - sigma).powi(2) / p.relative_diffusivity().nm2_per_ps();
        let mut rng = RngStreams::new(40_004)
            .stream(id as u64, Stage::Chemistry)
            .unwrap();
        let mut finite = 0;
        let mut early = 0;
        for _ in 0..N {
            match sample(p, distance, &mut rng) {
                ReactionTime::After(time) => {
                    finite += 1;
                    early += usize::from(time.ps() <= threshold);
                }
                ReactionTime::NoReaction => {}
                ReactionTime::Immediate => panic!("non-overlapping pair cannot be immediate"),
            }
        }
        // F(threshold)=Winf*erfc(1/2); both tests divide by N, not finite.
        for (count, probability) in [
            (N - finite, 1.0 - winf),
            (early, winf * 0.479_500_122_186_953_5),
        ] {
            let measured = count as f64 / N as f64;
            let tolerance = 5.0 * (probability * (1.0 - probability) / N as f64).sqrt();
            assert!((measured - probability).abs() < tolerance);
        }
    }
}

#[test]
fn invalid_species_or_coordinates_fail_before_drawing() {
    let p = parameters(1);
    let [a, b] = molecules(p, 2.0, 0.0);
    let mut rng = ScriptedRng::new(&[]);
    let wrong_species = Molecule::new(
        b.id(),
        SpeciesInstance {
            species: Species::OH,
            ..*b.instance()
        },
    );
    assert!(matches!(
        sample_independent_reaction_time(
            MoleculePair::new(&a, &wrong_species).unwrap(),
            &p,
            &mut rng
        ),
        Err(ChemistryError::ReactionSpeciesMismatch { .. })
    ));
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for position in [
            Vec3::new(value, 0.0, 0.0),
            Vec3::new(0.0, value, 0.0),
            Vec3::new(0.0, 0.0, value),
        ] {
            let invalid = Molecule::new(
                b.id(),
                SpeciesInstance {
                    position,
                    ..*b.instance()
                },
            );
            assert!(matches!(
                sample_independent_reaction_time(
                    MoleculePair::new(&a, &invalid).unwrap(),
                    &p,
                    &mut rng
                ),
                Err(ChemistryError::InvalidIrtInput { .. })
            ));
        }
    }
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn a_molecule_cannot_react_with_itself_even_if_its_record_differs() {
    let p = parameters(1);
    let [a, b] = molecules(p, 2.0, 0.0);
    let same_id = Molecule::new(a.id(), *b.instance());
    assert!(matches!(
        MoleculePair::new(&a, &same_id),
        Err(ChemistryError::DuplicateMoleculeId { .. })
    ));
}

#[test]
fn unrepresentable_separation_is_an_error_not_physical_nonreaction() {
    let p = parameters(1);
    let [a, b] = molecules(p, 2.0, 0.0);
    let extreme = |m: Molecule, x| {
        Molecule::new(
            m.id(),
            SpeciesInstance {
                position: Vec3::new(x, 0.0, 0.0),
                ..*m.instance()
            },
        )
    };
    let a = extreme(a, -f64::MAX);
    let b = extreme(b, f64::MAX);
    let mut rng = ScriptedRng::new(&[]);
    assert!(matches!(
        sample_independent_reaction_time(MoleculePair::new(&a, &b).unwrap(), &p, &mut rng),
        Err(ChemistryError::IrtDomain { .. })
    ));
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn a_large_finite_separation_does_not_overflow_the_distance_norm() {
    let p = parameters(1);
    let mut rng = ScriptedRng::new(&[0.75]);
    assert_eq!(sample(p, 1e200, &mut rng), ReactionTime::NoReaction);
    assert_eq!(rng.consumed(), 1);
}

#[test]
fn first_order_or_bulk_reactions_cannot_be_supplied_to_the_pair_sampler() {
    let table = ReactionTable::option3().unwrap();
    let species = SpeciesTable::option3().unwrap();
    for id in [53, 54, 61] {
        assert!(matches!(
            Parameters::derive(table.get(id).unwrap(), &species),
            Err(ChemistryError::BulkReactionParticipant { .. })
        ));
    }
    let mut reaction = table.get(1).unwrap().clone();
    reaction.rate = ObservedRate::FirstOrder { per_s: 1.0 };
    reaction.reactants = [ReactionParticipant::Mobile(Species::H); 2];
    assert!(matches!(
        Parameters::derive(&reaction, &species),
        Err(ChemistryError::UnsupportedReactionOrder { .. })
    ));
}

#[test]
fn unsupported_charged_pdc_is_an_error_before_any_contact_or_rng_draw() {
    let p = parameters(42);
    let [a, b] = molecules(p, 0.0, 0.0);
    let mut rng = ScriptedRng::new(&[]);
    assert!(matches!(
        sample_independent_reaction_time(MoleculePair::new(&a, &b).unwrap(), &p, &mut rng),
        Err(ChemistryError::UnsupportedIrtKinetics { .. })
    ));
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn unrepresentable_finite_reaction_time_is_an_error_not_no_reaction() {
    let table = ReactionTable::option3().unwrap();
    let mut reaction = table.get(1).unwrap().clone();
    reaction.rate = ObservedRate::Bimolecular {
        per_molar_s: 1e-100,
    };
    let mut props = *SpeciesTable::option3().unwrap().get(Species::H);
    props.diffusivity = Diffusivity::from_nm2_per_ps(1e-200);
    let p = Parameters::from_species(&reaction, [props; 2]).unwrap();
    let [a, b] = molecules(p, 2.0 * p.effective_radius().nm(), 0.0);
    let mut rng = ScriptedRng::new(&[0.25]);
    assert!(matches!(
        sample_independent_reaction_time(MoleculePair::new(&a, &b).unwrap(), &p, &mut rng),
        Err(ChemistryError::IrtDomain { .. })
    ));
    assert_eq!(rng.consumed(), 1);
}
