//! Neutral option3 PDC branch and RNG contract from Geant4 11.4.1
//! `G4DNAIRT.cc:314-346,368-405`. Relative time tolerance: 2e-12.
//! Synthetic fixtures set sigma=1 nm, D=1 nm^2/ps, k_obs=k_diff/2,
//! hence a=2/nm and W_inf=1/(2 r0). Literal proposal times and acceptance
//! brackets below were evaluated independently from the source envelope.

use dna_chemistry::ChemistryError;
use dna_chemistry::SpeciesInstance;
use dna_chemistry::irt::{ReactionTime, sample_independent_reaction_time};
use dna_chemistry::molecule::{Molecule, MoleculeId, MoleculePair};
use dna_chemistry::reaction_parameters::TwoBodyReactionParameters as Parameters;
use dna_chemistry::reactions::{ObservedRate, ReactionParticipant, ReactionTable};
use dna_chemistry::species::SpeciesTable;
use dna_core::rng::u64_to_open01;
use dna_core::testing::ScriptedRng;
use dna_core::units::{Diffusivity, Length, Time};
use dna_core::vec3::Vec3;

fn parameters() -> Parameters {
    scaled_parameters(1.0, 1.0)
}

fn scaled_parameters(radius: f64, diffusion: f64) -> Parameters {
    let mut reaction = ReactionTable::option3().unwrap().get(11).unwrap().clone();
    // Inverse SI conversion of k_obs=2*pi*D*sigma, independent of production.
    reaction.rate = ObservedRate::Bimolecular {
        per_molar_s: 3_783_822_634_099.930_7 * diffusion * radius,
    };
    let species = SpeciesTable::option3().unwrap();
    let props = reaction.reactants.map(|reactant| {
        let ReactionParticipant::Mobile(s) = reactant else {
            panic!("fixture must contain mobile species");
        };
        let mut prop = *species.get(s);
        prop.radius = Length::from_nm(radius / 2.0);
        prop.diffusivity = Diffusivity::from_nm2_per_ps(diffusion / 2.0);
        prop
    });
    Parameters::from_species(&reaction, props).unwrap()
}

fn sample(
    p: Parameters,
    distance: f64,
    rng: &mut ScriptedRng,
) -> Result<ReactionTime, ChemistryError> {
    let molecules: [Molecule; 2] = std::array::from_fn(|i| {
        Molecule::new(
            MoleculeId(i as u64),
            SpeciesInstance {
                species: p.reactants()[i],
                position: Vec3::new(i as f64 * distance, 0.0, 0.0),
                time: Time::from_ps(10.0),
            },
        )
    });
    sample_independent_reaction_time(
        MoleculePair::new(&molecules[0], &molecules[1]).unwrap(),
        &p,
        rng,
    )
}

fn elapsed(result: Result<ReactionTime, ChemistryError>) -> f64 {
    let ReactionTime::After(time) = result.unwrap() else {
        panic!("expected a finite positive continuum delay");
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
fn overlap_consumes_only_the_source_contact_probability_draw() {
    let p = parameters();
    // p_contact = .29 / (.29 + 1*(1+.29)) = .1835443038...
    for distance in [0.0, 0.5, f64::from_bits(1.0_f64.to_bits() - 1)] {
        for (draw, expected) in [
            (0.18, ReactionTime::Immediate),
            (0.19, ReactionTime::NoReaction),
        ] {
            let mut rng = ScriptedRng::new(&[draw]);
            assert_eq!(sample(p, distance, &mut rng).unwrap(), expected);
            assert_eq!(rng.consumed(), 1);
        }
    }
}

#[test]
fn option3_oh_h_uses_the_stored_contact_probability() {
    let p = Parameters::derive(
        ReactionTable::option3().unwrap().get(11).unwrap(),
        &SpeciesTable::option3().unwrap(),
    )
    .unwrap();
    for (draw, expected) in [
        (0.3298, ReactionTime::Immediate),
        (0.33, ReactionTime::NoReaction),
    ] {
        let mut rng = ScriptedRng::new(&[draw]);
        assert_eq!(sample(p, 0.2, &mut rng).unwrap(), expected);
        assert_eq!(rng.consumed(), 1);
    }
}

#[test]
fn nonreaction_gate_includes_observed_over_diffusion_rate() {
    // W_inf=1/6, NOT sigma/r=1/3, and failure draws no proposal.
    let mut rng = ScriptedRng::new(&[0.2]);
    assert_eq!(
        sample(parameters(), 3.0, &mut rng).unwrap(),
        ReactionTime::NoReaction
    );
    assert_eq!(rng.consumed(), 1);
}

#[test]
fn lower_envelope_proposal_and_acceptance_are_distinct_draws() {
    // b=1, M=1.5, p/(p+qM)=.4. u=.2 gives X=.25 and lambda=.0125046127.
    let mut rng = ScriptedRng::new(&[0.1, 0.2, 0.012]);
    close(elapsed(sample(parameters(), 3.0, &mut rng)), 0.25);
    assert_eq!(rng.consumed(), 3);
}

#[test]
fn rejected_proposal_retries_without_repeating_the_ultimate_gate() {
    // Reject first lambda=.0125046, then accept upper envelope at .2013544.
    let mut rng = ScriptedRng::new(&[0.1, 0.2, 0.013, 0.6, 0.201]);
    close(elapsed(sample(parameters(), 3.0, &mut rng)), 2.25);
    assert_eq!(rng.consumed(), 5);
}

#[test]
fn upper_envelope_acceptance_scales_by_x_over_m() {
    let mut rng = ScriptedRng::new(&[0.1, 0.6, 0.202, 0.2, 0.012]);
    close(elapsed(sample(parameters(), 3.0, &mut rng)), 0.25);
    assert_eq!(rng.consumed(), 5);
}

#[test]
fn inverse_a_squared_envelope_applies_for_small_gaps() {
    // b=.05 => M=.25, split=1/6. The accepted lower proposal has lambda=.6426795.
    let mut rng = ScriptedRng::new(&[0.1, 0.1, 0.642]);
    close(elapsed(sample(parameters(), 1.1, &mut rng)), 0.018);
    assert_eq!(rng.consumed(), 3);
    let mut rng = ScriptedRng::new(&[0.1, 0.1, 0.643, 0.5, 0.211]);
    close(
        elapsed(sample(parameters(), 1.1, &mut rng)),
        0.138_888_888_888_888_9,
    );
    assert_eq!(rng.consumed(), 5);
}

#[test]
fn equality_enters_the_continuum_and_has_nonreaction_mass() {
    let mut rng = ScriptedRng::new(&[0.6]);
    assert_eq!(
        sample(parameters(), 1.0, &mut rng).unwrap(),
        ReactionTime::NoReaction
    );
    assert_eq!(rng.consumed(), 1);
    // At b=0 the conditional survival is erfcx(a*sqrt(D*t)). Its exponential /
    // half-normal mixture uses u=.25, v=.5: t=[-ln(.25)/(4 erfc_inv(.5))]^2.
    // Gate .3 is ABOVE p_contact, so a mistaken contact branch cannot pass.
    let mut rng = ScriptedRng::new(&[0.3, 0.25, 0.5]);
    close(
        elapsed(sample(parameters(), 1.0, &mut rng)),
        0.528_044_128_258_248_9,
    );
    assert_eq!(rng.consumed(), 3);
}

#[test]
fn zero_distance_retains_the_source_point_zero_zero_one_nm_substitution() {
    let mut rng = ScriptedRng::new(&[0.75]);
    assert_eq!(
        sample(scaled_parameters(1e-4, 1.0), 0.0, &mut rng).unwrap(),
        ReactionTime::NoReaction
    );
    assert_eq!(rng.consumed(), 1);
}

#[test]
fn the_ten_thousandth_proposal_may_still_succeed() {
    let mut draws = vec![0.1];
    for _ in 0..9_999 {
        draws.extend([0.2, 0.013]);
    }
    draws.extend([0.6, 0.201]);
    let mut rng = ScriptedRng::new(&draws);
    close(elapsed(sample(parameters(), 3.0, &mut rng)), 2.25);
    assert_eq!(rng.consumed(), 20_001);
}

#[test]
fn ten_thousand_rejections_are_an_error_never_physical_nonreaction() {
    let mut draws = vec![0.1];
    for _ in 0..10_000 {
        draws.extend([0.2, 0.013]);
    }
    // A forbidden 10,001st proposal would succeed, detecting the C++ off-by-one.
    draws.extend([0.6, 0.001]);
    let mut rng = ScriptedRng::new(&draws);
    assert_eq!(
        sample(parameters(), 3.0, &mut rng),
        Err(ChemistryError::IrtConvergence {
            reaction_id: 11,
            attempts: 10_000,
        })
    );
    assert_eq!(rng.consumed(), 20_001);
}

#[test]
fn extreme_proposals_preserve_tails_without_erfcx_overflow_or_cancellation() {
    let min = u64_to_open01(0);
    let max = u64_to_open01(u64::MAX);
    // A nearly zero X underflows exp(-b^2/X), legitimately rejecting it.
    let mut rng = ScriptedRng::new(&[0.1, min, min, 0.6, 0.201]);
    close(elapsed(sample(parameters(), 3.0, &mut rng)), 2.25);
    assert_eq!(rng.consumed(), 5);
    // At X~1e31, naive 1-sqrt(pi)*z*erfcx(z) loses the entire positive tail.
    let mut rng = ScriptedRng::new(&[0.1, max, 0.4]);
    let time = elapsed(sample(parameters(), 3.0, &mut rng));
    assert!(time > 1e30);
    assert_eq!(rng.consumed(), 3);
}

#[test]
fn large_separation_can_return_physical_nonreaction_before_forming_the_envelope() {
    let mut rng = ScriptedRng::new(&[0.75]);
    assert_eq!(
        sample(parameters(), 1e200, &mut rng).unwrap(),
        ReactionTime::NoReaction
    );
    assert_eq!(rng.consumed(), 1);
}

#[test]
fn unrepresentable_selected_time_is_a_domain_error() {
    let p = scaled_parameters(1e100, 1e-200);
    let mut rng = ScriptedRng::new(&[0.1, 0.6, 0.201]);
    assert!(matches!(
        sample(p, 3e100, &mut rng),
        Err(ChemistryError::IrtDomain { .. })
    ));
    assert_eq!(rng.consumed(), 3);
}
