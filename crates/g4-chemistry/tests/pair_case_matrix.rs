//! R04 preregistration. This file contains no sampled outcomes or fitted values.
//! Keep historical V9 and RR-003 criteria in their original tests and requests.

use g4_chemistry::Species;
use g4_chemistry::reaction_parameters::TwoBodyReactionParameters;
use g4_chemistry::reactions::ReactionTable;
use g4_chemistry::species::SpeciesTable;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Regime {
    NeutralTdc,
    ChargedTdc,
    NeutralPdc,
    ChargedPdc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PairKind {
    Ab,
    Aa,
}

#[derive(Clone, Copy, Debug)]
struct LiteralCase {
    id: u16,
    reactants: [Species; 2],
    regime: Regime,
    kind: PairKind,
}

// IDs and ordered reactants are from docs/validation/option3-reaction-coverage.md.
const LITERAL_CASES: [LiteralCase; 8] = [
    LiteralCase {
        id: 1,
        reactants: [Species::H, Species::H],
        regime: Regime::NeutralTdc,
        kind: PairKind::Aa,
    },
    LiteralCase {
        id: 2,
        reactants: [Species::e_aq, Species::H],
        regime: Regime::NeutralTdc,
        kind: PairKind::Ab,
    },
    LiteralCase {
        id: 8,
        reactants: [Species::e_aq, Species::e_aq],
        regime: Regime::ChargedTdc,
        kind: PairKind::Aa,
    },
    LiteralCase {
        id: 9,
        reactants: [Species::H3Op, Species::OHm],
        regime: Regime::ChargedTdc,
        kind: PairKind::Ab,
    },
    LiteralCase {
        id: 11,
        reactants: [Species::OH, Species::H],
        regime: Regime::NeutralPdc,
        kind: PairKind::Ab,
    },
    LiteralCase {
        id: 17,
        reactants: [Species::OH, Species::OH],
        regime: Regime::NeutralPdc,
        kind: PairKind::Aa,
    },
    LiteralCase {
        id: 42,
        reactants: [Species::e_aq, Species::H3Op],
        regime: Regime::ChargedPdc,
        kind: PairKind::Ab,
    },
    LiteralCase {
        id: 51,
        reactants: [Species::Om, Species::Om],
        regime: Regime::ChargedPdc,
        kind: PairKind::Aa,
    },
];

#[derive(Clone, Copy, Debug)]
struct SyntheticCase {
    name: &'static str,
    regime: Regime,
    kind: PairKind,
    /// q_A q_B times the dimensionless Onsager length; zero for neutral pairs.
    signed_onsager_over_radius: f64,
    /// k_obs/k_diff for PDC only. This is a test parameter, not a source rate.
    pdc_rate_fraction: Option<f64>,
}

// D_A + D_B = 1, a = 1; all values here are dimensionless test inputs.
const SYNTHETIC_CASES: [SyntheticCase; 4] = [
    SyntheticCase {
        name: "syn-neutral-tdc-ab",
        regime: Regime::NeutralTdc,
        kind: PairKind::Ab,
        signed_onsager_over_radius: 0.0,
        pdc_rate_fraction: None,
    },
    SyntheticCase {
        name: "syn-charged-tdc-aa",
        regime: Regime::ChargedTdc,
        kind: PairKind::Aa,
        signed_onsager_over_radius: 1.0,
        pdc_rate_fraction: None,
    },
    SyntheticCase {
        name: "syn-neutral-pdc-aa",
        regime: Regime::NeutralPdc,
        kind: PairKind::Aa,
        signed_onsager_over_radius: 0.0,
        pdc_rate_fraction: Some(0.5),
    },
    SyntheticCase {
        name: "syn-charged-pdc-ab",
        regime: Regime::ChargedPdc,
        kind: PairKind::Ab,
        signed_onsager_over_radius: -1.0,
        pdc_rate_fraction: Some(0.5),
    },
];

const DISTANCE_OVER_RADIUS: [f64; 3] = [1.0, 1.5, 3.0];
const OBSERVATION_TAU: [f64; 5] = [0.01, 0.1, 1.0, 10.0, 100.0];
const POINTS_PER_DISTANCE: usize = OBSERVATION_TAU.len() + 1; // eventual nonreaction
const FAMILY_POINTS: usize = (LITERAL_CASES.len() + SYNTHETIC_CASES.len())
    * DISTANCE_OVER_RADIUS.len()
    * POINTS_PER_DISTANCE;

const FAMILY_ALPHA: f64 = 0.001;
const BROWNIAN_PAIR_COUNTS: [usize; 8] = [
    10_000, 20_000, 40_000, 80_000, 160_000, 320_000, 640_000, 1_000_000,
];
const REFINEMENT_DENOMINATORS: [usize; 5] = [16, 32, 64, 128, 256];
const BROWNIAN_ABSOLUTE_CDF_LIMIT: f64 = 0.005;
const GEANT4_ABSOLUTE_EFFECT_LIMIT: f64 = 0.01;
const THREE_COMBINED_SE: f64 = 3.0;
const ANALYTIC_SEED: u64 = 20_261_027_401;
const BROWNIAN_SEED: u64 = 20_261_027_402;
const RUST_GEANT4_COMPARISON_SEED: u64 = 20_261_027_403;
const GEANT4_SEED: u64 = 20_261_027_404;

#[test]
fn literal_cases_match_compiled_option3_channels() {
    let reactions = ReactionTable::option3().unwrap();
    let species = SpeciesTable::option3().unwrap();
    for case in LITERAL_CASES {
        let p =
            TwoBodyReactionParameters::derive(reactions.get(case.id).unwrap(), &species).unwrap();
        assert_eq!(p.reactants(), case.reactants, "reaction {}", case.id);
        assert_eq!(
            case.kind == PairKind::Aa,
            case.reactants[0] == case.reactants[1]
        );
        assert_eq!(
            matches!(case.regime, Regime::NeutralPdc | Regime::ChargedPdc),
            p.pdc().is_some()
        );
        assert_eq!(
            matches!(case.regime, Regime::ChargedTdc | Regime::ChargedPdc),
            p.onsager_radius().nm() != 0.0
        );
        assert!(p.reaction_radius().nm() > 0.0);
        assert!(p.relative_diffusivity().nm2_per_ps() > 0.0);
    }
}

#[test]
fn preregistered_grid_and_statistical_contract_are_fixed() {
    assert_eq!(LITERAL_CASES.len(), 8);
    assert_eq!(SYNTHETIC_CASES.len(), 4);
    assert_eq!(FAMILY_POINTS, 216);
    assert_eq!(DISTANCE_OVER_RADIUS, [1.0, 1.5, 3.0]);
    assert_eq!(OBSERVATION_TAU, [0.01, 0.1, 1.0, 10.0, 100.0]);
    assert_eq!(FAMILY_ALPHA / FAMILY_POINTS as f64, 0.001 / 216.0);
    assert_eq!(BROWNIAN_PAIR_COUNTS[0], 10_000);
    assert_eq!(BROWNIAN_PAIR_COUNTS[7], 1_000_000);
    for levels in REFINEMENT_DENOMINATORS.windows(2) {
        assert_eq!(levels[1], 2 * levels[0]);
    }
    assert_eq!(BROWNIAN_ABSOLUTE_CDF_LIMIT, 0.005);
    assert_eq!(GEANT4_ABSOLUTE_EFFECT_LIMIT, 0.01);
    assert_eq!(THREE_COMBINED_SE, 3.0);
    let seeds = [
        ANALYTIC_SEED,
        BROWNIAN_SEED,
        RUST_GEANT4_COMPARISON_SEED,
        GEANT4_SEED,
    ];
    for (i, seed) in seeds.iter().enumerate() {
        assert!(*seed > 0 && !seeds[..i].contains(seed));
    }
    for case in SYNTHETIC_CASES {
        assert!(case.name.starts_with("syn-"));
        assert_eq!(
            case.signed_onsager_over_radius != 0.0,
            matches!(case.regime, Regime::ChargedTdc | Regime::ChargedPdc)
        );
        assert_eq!(
            case.pdc_rate_fraction.is_some(),
            matches!(case.regime, Regime::NeutralPdc | Regime::ChargedPdc)
        );
        if let Some(fraction) = case.pdc_rate_fraction {
            assert!(fraction > 0.0 && fraction < 1.0);
        }
        let _ = case.kind;
    }
}
