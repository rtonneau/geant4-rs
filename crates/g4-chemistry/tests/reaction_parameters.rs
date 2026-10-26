//! Geant4 11.4.1 source-backed construction checks. Relative tolerance 2e-13,
//! fixed before execution. Literal expectations independently evaluated from
//! G4DNAMolecularReactionTable.cc:127-151,296-335 and the CLHEP constants.
//! The three registered regimes use G4EmDNAChemistry_option3.cc:146,200,215.

use g4_chemistry::ChemistryError;
use g4_chemistry::Species;
use g4_chemistry::reaction_parameters::TwoBodyReactionParameters as Parameters;
use g4_chemistry::reactions::{
    BulkParticipant, KineticIntent, ObservedRate, Reaction, ReactionParticipant, ReactionTable,
};
use g4_chemistry::species::{SpeciesProps, SpeciesTable};
use g4_core::units::{Diffusivity, Length};

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 2e-13 * expected.abs(),
        "{actual:.17e} != {expected:.17e}"
    );
}

fn inputs(id: u16) -> (Reaction, [SpeciesProps; 2]) {
    let reaction = ReactionTable::option3().unwrap().get(id).unwrap().clone();
    let species = SpeciesTable::option3().unwrap();
    let props = reaction.reactants.map(|p| match p {
        ReactionParticipant::Mobile(s) => *species.get(s),
        _ => panic!("test needs mobile species"),
    });
    (reaction, props)
}

#[test]
fn registered_h_h_uses_single_species_radius_but_relative_diffusion() {
    let table = ReactionTable::option3().unwrap();
    let reaction = table.get(1).unwrap();
    let original = reaction.clone();
    let p = Parameters::derive(reaction, &SpeciesTable::option3().unwrap()).unwrap();
    assert_eq!(p.reaction_id(), 1);
    assert_eq!(p.reactants(), [Species::H, Species::H]);
    close(p.relative_diffusivity().nm2_per_ps(), 0.014);
    close(p.observed_rate().nm3_per_ps(), 0.00835251150788445);
    close(p.effective_radius().nm(), 0.0949531066936965);
    assert_eq!(p.reaction_radius(), p.effective_radius());
    assert_eq!(p.onsager_radius().nm(), 0.0);
    assert_eq!(p.contact_probability(), 1.0);
    assert!(p.pdc().is_none());
    assert_eq!(reaction, &original);
}

#[test]
fn registered_oh_h_keeps_observed_activation_and_diffusion_rates_distinct() {
    let (reaction, props) = inputs(11);
    let p = Parameters::from_species(&reaction, props).unwrap();
    close(p.relative_diffusivity().nm2_per_ps(), 0.0092);
    close(p.reaction_radius().nm(), 0.41);
    close(p.effective_radius().nm(), 0.41);
    close(p.observed_rate().nm3_per_ps(), 0.0257383555411946);
    close(
        p.pdc().unwrap().diffusion_rate().nm3_per_ps(),
        0.0474003499573628,
    );
    close(
        p.pdc().unwrap().activation_rate().nm3_per_ps(),
        0.0563201631641572,
    );
    close(p.contact_probability(), 0.3298692641268);
}

#[test]
fn registered_h3op_ohm_retains_signed_onsager_and_transforms_distance_only() {
    let (reaction, props) = inputs(9);
    let p = Parameters::from_species(&reaction, props).unwrap();
    close(p.relative_diffusivity().nm2_per_ps(), 0.01476);
    close(p.effective_radius().nm(), 1.01165240797375);
    close(p.onsager_radius().nm(), -0.7116337048868);
    close(
        p.effective_distance(Length::from_nm(2.0)).unwrap().nm(),
        2.37687340002239,
    );
    assert_eq!(p.reaction_radius(), p.effective_radius());
}

#[test]
fn neutral_ab_and_charged_aa_follow_constructor_and_charge_sign() {
    let (reaction, props) = inputs(2);
    let p = Parameters::from_species(&reaction, props).unwrap();
    close(p.relative_diffusivity().nm2_per_ps(), 0.0119);
    assert_eq!(
        p.effective_distance(Length::from_nm(2.0)).unwrap().nm(),
        2.0
    );
    let (reaction, props) = inputs(8);
    let p = Parameters::from_species(&reaction, props).unwrap();
    close(p.onsager_radius().nm(), 0.7116337048868);
    assert!(p.effective_distance(Length::from_nm(2.0)).unwrap().nm() < 2.0);
}

#[test]
fn pdc_aa_halves_diffusion_rate_and_is_invariant_under_ab_exchange() {
    let (mut reaction, props) = inputs(11);
    let p = Parameters::from_species(&reaction, props).unwrap();
    reaction.reactants.reverse();
    let q = Parameters::from_species(&reaction, [props[1], props[0]]).unwrap();
    assert_eq!(p.pdc(), q.pdc());
    assert_eq!(p.contact_probability(), q.contact_probability());
    // Synthetic OH+OH at 1e9 M^-1 s^-1 isolates the identical-species factor.
    reaction.reactants = [ReactionParticipant::Mobile(Species::OH); 2];
    reaction.rate = ObservedRate::Bimolecular { per_molar_s: 1e9 };
    let p = Parameters::from_species(&reaction, [props[0]; 2]).unwrap();
    close(
        p.pdc().unwrap().diffusion_rate().nm3_per_ps(),
        0.012164246754699678,
    );
}

#[test]
fn charged_pdc_uses_electrostatic_effective_contact_radius() {
    let (mut reaction, props) = inputs(9);
    reaction.kinetic_intent = KineticIntent::PartiallyDiffusionControlled;
    reaction.set_reaction_type = true;
    reaction.rate = ObservedRate::Bimolecular { per_molar_s: 1e9 };
    let p = Parameters::from_species(&reaction, props).unwrap();
    close(p.reaction_radius().nm(), 0.58);
    assert_eq!(
        p.effective_radius(),
        p.effective_distance(p.reaction_radius()).unwrap()
    );
    assert!(p.effective_radius() > p.reaction_radius());
    assert!(p.contact_probability() > 0.0 && p.contact_probability() < 1.0);
}

#[test]
fn invalid_numeric_inputs_and_zero_relative_diffusion_are_typed() {
    let (reaction, props) = inputs(11);
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
        let mut changed = props;
        changed[0].diffusivity = Diffusivity::from_nm2_per_ps(bad);
        assert!(matches!(
            Parameters::from_species(&reaction, changed),
            Err(ChemistryError::InvalidReactionParameter { .. })
        ));
    }
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let mut changed = props;
        changed[0].radius = Length::from_nm(bad);
        assert!(matches!(
            Parameters::from_species(&reaction, changed),
            Err(ChemistryError::InvalidReactionParameter { .. })
        ));
        let mut changed = reaction.clone();
        changed.rate = ObservedRate::Bimolecular { per_molar_s: bad };
        assert!(matches!(
            Parameters::from_species(&changed, props),
            Err(ChemistryError::InvalidReactionParameter { .. })
        ));
    }
    let mut changed = props;
    for prop in &mut changed {
        prop.diffusivity = Diffusivity::from_nm2_per_ps(0.0);
    }
    assert!(matches!(
        Parameters::from_species(&reaction, changed),
        Err(ChemistryError::ZeroTotalDiffusivity { .. })
    ));
}

#[test]
fn unsupported_order_bulk_and_inconsistent_species_are_typed() {
    let (mut reaction, props) = inputs(11);
    reaction.rate = ObservedRate::FirstOrder { per_s: 1.0 };
    assert!(matches!(
        Parameters::from_species(&reaction, props),
        Err(ChemistryError::UnsupportedReactionOrder { .. })
    ));
    let (mut reaction, props) = inputs(11);
    reaction.reactants[1] = ReactionParticipant::Bulk(BulkParticipant::H2OB);
    assert!(matches!(
        Parameters::from_species(&reaction, props),
        Err(ChemistryError::BulkReactionParticipant { .. })
    ));
    let (reaction, props) = inputs(11);
    assert!(matches!(
        Parameters::from_species(&reaction, [props[1], props[0]]),
        Err(ChemistryError::ReactionSpeciesMismatch { .. })
    ));
}

#[test]
fn pdc_at_or_above_diffusion_limit_is_rejected() {
    let (mut reaction, props) = inputs(11);
    // Obtain the boundary in source units; allow rounding to either neighbour
    // by testing the first representable observed rate at or above the limit.
    let limit = Parameters::from_species(&reaction, props)
        .unwrap()
        .pdc()
        .unwrap()
        .diffusion_rate()
        .nm3_per_ps();
    let mut rate = limit * 6.02214076e23 / 1e12;
    while g4_core::units::RateConstant::from_per_molar_per_s(rate).nm3_per_ps() < limit {
        rate = f64::from_bits(rate.to_bits() + 1);
    }
    for bad in [rate, rate * 2.0] {
        reaction.rate = ObservedRate::Bimolecular { per_molar_s: bad };
        assert!(matches!(
            Parameters::from_species(&reaction, props),
            Err(ChemistryError::InvalidPdcRate { .. })
        ));
    }
}

#[test]
fn arithmetic_overflow_underflow_and_effective_distance_domain_are_errors() {
    let (reaction, mut props) = inputs(11);
    props[0].diffusivity = Diffusivity::from_nm2_per_ps(f64::MAX);
    props[1].diffusivity = Diffusivity::from_nm2_per_ps(f64::MAX);
    assert!(matches!(
        Parameters::from_species(&reaction, props),
        Err(ChemistryError::ReactionParameterDomain { .. })
    ));
    let (mut reaction, props) = inputs(1);
    reaction.rate = ObservedRate::Bimolecular {
        per_molar_s: f64::from_bits(1),
    };
    assert!(matches!(
        Parameters::from_species(&reaction, props),
        Err(ChemistryError::ReactionParameterDomain { .. })
    ));
    let (reaction, props) = inputs(8);
    let p = Parameters::from_species(&reaction, props).unwrap();
    for distance in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(matches!(
            p.effective_distance(Length::from_nm(distance)),
            Err(ChemistryError::InvalidReactionParameter { .. })
        ));
    }
    assert!(matches!(
        p.effective_distance(Length::from_nm(1e-300)),
        Err(ChemistryError::ReactionParameterDomain { .. })
    ));
}
