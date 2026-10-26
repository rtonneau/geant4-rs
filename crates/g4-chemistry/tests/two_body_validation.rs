//! V9 pre-registration (2026-10-23; fixed before the first statistical run).
//!
//! One family of 18 curve comparisons: option3 H + H (neutral A+A TDC, id 1,
//! r0=0.2 nm), °OH + H (neutral A+B PDC, id 11, r0=0.8 nm), H3Op + OHm
//! (charged A+B TDC, id 9, r0=2 nm), each at tau=[0.25,0.5,1,2,4,16].
//! tau=4*D*t/(r_eff-sigma)^2, with D=D_A+D_B; neutral r_eff=r0.
//! N=100,000 independent pairs per regime, seed=20_261_023_006, chemistry
//! history_id=reaction_id*N+pair_index. Every point must satisfy the absolute
//! probability bound |F_emp-F| <= 3*sqrt(F*(1-F)/N). No relative tolerance,
//! fitted parameters, seed search, or post-run point/threshold adjustment.
//! These correlated curve points are one family; 3 sigma is a pointwise rule,
//! not a claim of 99.73% simultaneous coverage. Survival is the complement of
//! the same event, including NoReaction in N, never conditioning on reaction.
//! Three separately registered mass-at-infinity checks use these same samples
//! and the same absolute 3-sigma binomial rule for P(NoReaction).
//!
//! Independent analytic oracle: mpmath 1.3.0, 80 decimal digits, evaluated from
//! source literals (not Rust parameters or the sampler's special functions).
//! Rates k_obs=[0.503e10,1.55e10,1.13e11] M^-1 s^-1 convert with 1e12/N_A,
//! N_A=6.02214076e23. D=[.014,.0092,.01476] nm^2/ps. TDC sigma=k/(4*pi*D_rate),
//! D_rate=.007 for H+H and .01476 for H3Op+OHm. PDC sigma=.22+.19 nm,
//! k_diff=4*pi*D*sigma, k_act=k_obs*k_diff/(k_diff-k_obs), a=k_act/(k_obs*sigma).
//! TDC F(t)=W_inf*erfc(x), W_inf=sigma/r_eff; PDC W_inf=sigma/r0*k_obs/k_diff,
//! F(t)=W_inf*[erfc(x)-exp(-x*x)*erfcx(x+a*sqrt(D*t))], x=1/sqrt(tau).
//! erfcx(z)=exp(z*z)*erfc(z). Literals below are rounded once to binary64.
//! The signed source Onsager radius uses
//! rc=-1.602176634e-19*299792458^2*1e-7/8.617333e-5/(293.15*80.1)*1e9 nm;
//! r_eff=rc/expm1(rc/r0). This charged case tests Geant4 effective-distance
//! fidelity, not a general exact electrostatic first-passage solution.
//!
//! Reference: Geant4 11.4.1 G4DNAIRT.cc, GetIndependentReactionTime/SamplePDC;
//! G4DNAMolecularReactionTable.cc, ComputeEffectiveRadius/SetReactionType;
//! G4EmDNAChemistry_option3.cc and G4ChemDissociationChannels_option1.cc.
//! Contact and extreme policies below are separate deterministic checks.

mod common;

use common::{isolated_pair, option3_parameters, reaction_time_bits};
use g4_chemistry::ChemistryError;
use g4_chemistry::irt::{ReactionTime, sample_independent_reaction_time};
use g4_chemistry::molecule::{Molecule, MoleculePair};
use g4_chemistry::reaction_parameters::TwoBodyReactionParameters as Parameters;
use g4_chemistry::{Species, SpeciesInstance};
use g4_core::rng::{RngStreams, Stage, u64_to_open01};
use g4_core::testing::ScriptedRng;
use g4_core::units::Length;
use g4_core::vec3::Vec3;

const N: usize = 100_000;
const SEED: u64 = 20_261_023_006;
const TAU: [f64; 6] = [0.25, 0.5, 1.0, 2.0, 4.0, 16.0];

struct Regime {
    name: &'static str,
    reaction_id: u16,
    species: [Species; 2],
    distance_nm: f64,
    time_scale_ps: f64,
    cdf: [f64; 6],
    no_reaction: f64,
}

const REGIMES: [Regime; 3] = [
    Regime {
        name: "H + H (neutral A+A TDC)",
        reaction_id: 1,
        species: [Species::H, Species::H],
        distance_nm: 0.2,
        time_scale_ps: 0.197_050_889_166_177_16,
        cdf: [
            0.002_220_827_343_701_086,
            0.021_601_957_061_711_33,
            0.074_680_241_949_397_87,
            0.150_648_092_540_691_45,
            0.227_650_131_308_291_4,
            0.343_575_287_428_839_3,
        ],
        no_reaction: 0.525_234_466_531_517_7,
    },
    Regime {
        name: "°OH + H (neutral A+B PDC)",
        reaction_id: 11,
        species: [Species::OH, Species::H],
        distance_nm: 0.8,
        time_scale_ps: 4.133_152_173_913_043,
        cdf: [
            0.000_234_935_050_228_257_36,
            0.003_613_585_543_007_519_6,
            0.018_066_144_494_784_605,
            0.048_208_492_975_613_12,
            0.089_027_931_456_887_64,
            0.168_764_805_623_126_6,
        ],
        no_reaction: 0.721_712_872_864_28,
    },
    Regime {
        name: "H3Op + OHm (charged A+B TDC)",
        reaction_id: 9,
        species: [Species::H3Op, Species::OHm],
        distance_nm: 2.0,
        time_scale_ps: 31.568_908_487_978_43,
        cdf: [
            0.001_990_952_424_052_103_5,
            0.019_365_966_876_384_24,
            0.066_950_188_252_889_06,
            0.135_054_706_469_376_5,
            0.204_086_365_403_211_07,
            0.308_012_260_967_061_9,
        ],
        no_reaction: 0.574_376_822_945_543_8,
    },
];

fn within_three_sigma(label: &str, point: f64, count: usize, probability: f64) {
    let empirical = count as f64 / N as f64;
    let standard_error = (probability * (1.0 - probability) / N as f64).sqrt();
    let residual = (empirical - probability).abs();
    eprintln!(
        "{label}, tau={point}: count={count}/{N}, expected={probability:.10}, |z|={:.4}",
        residual / standard_error
    );
    assert!(
        residual <= 3.0 * standard_error,
        "{label}, tau={point}: empirical={empirical}, expected={probability}, \
         absolute error={residual} exceeds 3 sigma={}",
        3.0 * standard_error
    );
}

#[test]
fn v9_eighteen_cdf_survival_points_and_three_no_reaction_masses() {
    let streams = RngStreams::new(SEED);
    let mut comparisons = 0;
    for regime in REGIMES {
        let p = option3_parameters(regime.reaction_id);
        assert_eq!(p.reactants(), regime.species);
        let thresholds = TAU.map(|tau| tau * regime.time_scale_ps);
        let mut early = [0; 6];
        let mut surviving = [0; 6];
        let mut no_reaction = 0;
        for index in 0..N {
            let history_id = u64::from(regime.reaction_id) * N as u64 + index as u64;
            let [a, b] = isolated_pair(p, regime.distance_nm, history_id);
            let mut rng = streams.stream(history_id, Stage::Chemistry).unwrap();
            let outcome =
                sample_independent_reaction_time(MoleculePair::new(&a, &b).unwrap(), &p, &mut rng)
                    .unwrap_or_else(|error| panic!("{}, pair {index}: {error}", regime.name));
            match outcome {
                ReactionTime::After(time) => {
                    assert!(time.ps().is_finite() && time.ps() > 0.0);
                    for i in 0..6 {
                        if time.ps() <= thresholds[i] {
                            early[i] += 1;
                        } else {
                            surviving[i] += 1;
                        }
                    }
                }
                ReactionTime::NoReaction => {
                    no_reaction += 1;
                    for count in &mut surviving {
                        *count += 1;
                    }
                }
                ReactionTime::Immediate => panic!("{}: initially separated pair", regime.name),
            }
        }
        for i in 0..6 {
            assert_eq!(early[i] + surviving[i], N);
            assert!(surviving[i] >= no_reaction);
            // F and S=1-F have the same absolute binomial residual and variance.
            within_three_sigma(regime.name, TAU[i], early[i], regime.cdf[i]);
            comparisons += 1;
        }
        within_three_sigma(regime.name, f64::INFINITY, no_reaction, regime.no_reaction);
    }
    assert_eq!(comparisons, 18);
}

fn sample(p: Parameters, distance: f64, draws: &[f64]) -> ReactionTime {
    let [a, b] = isolated_pair(p, distance, 0);
    sample_independent_reaction_time(
        MoleculePair::new(&a, &b).unwrap(),
        &p,
        &mut ScriptedRng::new(draws),
    )
    .unwrap()
}

#[test]
fn aa_exchange_preserves_outcome_and_elapsed_time_bits() {
    let p = option3_parameters(1);
    let [a, b] = isolated_pair(p, 0.2, 0);
    let streams = RngStreams::new(SEED);
    let mut kinds = [false; 3];
    for id in 0..1024 {
        let mut forward = streams.stream(id, Stage::Chemistry).unwrap();
        let mut reverse = forward.clone();
        let outcome = |first, second, rng: &mut _| {
            reaction_time_bits(
                sample_independent_reaction_time(
                    MoleculePair::new(first, second).unwrap(),
                    &p,
                    rng,
                )
                .unwrap(),
            )
        };
        let expected = outcome(&a, &b, &mut forward);
        assert_eq!(expected, outcome(&b, &a, &mut reverse));
        kinds[usize::from(expected.0)] = true;
    }
    assert!(kinds[1] && kinds[2]);
}

#[test]
fn same_identity_is_rejected_even_when_the_species_or_position_differs() {
    let [a, b] = isolated_pair(option3_parameters(1), 0.2, 17);
    assert!(MoleculePair::new(&a, &b).is_ok());
    for instance in [
        *a.instance(),
        *b.instance(),
        SpeciesInstance {
            species: Species::OH,
            ..*b.instance()
        },
    ] {
        let duplicate = Molecule::new(a.id(), instance);
        assert!(matches!(
            MoleculePair::new(&a, &duplicate),
            Err(ChemistryError::DuplicateMoleculeId { id }) if id == a.id()
        ));
    }
}

#[test]
fn contact_uses_strict_overlap_and_keeps_pdc_equality_in_the_continuum() {
    for id in [1, 9] {
        let p = option3_parameters(id);
        let sigma = p.effective_radius().nm();
        for distance in [0.0, sigma * 0.5, f64::from_bits(sigma.to_bits() - 1)] {
            assert_eq!(sample(p, distance, &[]), ReactionTime::Immediate);
        }
    }
    let tdc = option3_parameters(1);
    assert_eq!(
        sample(tdc, tdc.effective_radius().nm(), &[0.5]),
        ReactionTime::Immediate
    );
    let pdc = option3_parameters(11);
    let sigma = pdc.reaction_radius().nm();
    for distance in [0.0, sigma * 0.5, f64::from_bits(sigma.to_bits() - 1)] {
        assert_eq!(sample(pdc, distance, &[0.3298]), ReactionTime::Immediate);
        assert_eq!(sample(pdc, distance, &[0.33]), ReactionTime::NoReaction);
    }
    assert!(matches!(
        sample(pdc, sigma, &[0.4, 0.25, 0.5]),
        ReactionTime::After(time) if time.ps().is_finite() && time.ps() > 0.0
    ));
    assert_eq!(sample(pdc, sigma, &[0.6]), ReactionTime::NoReaction);
}

#[test]
fn charged_law_uses_geant4_effective_distance_after_the_raw_contact_test() {
    let p = option3_parameters(9);
    let effective = p.effective_distance(Length::from_nm(2.0)).unwrap().nm();
    assert!((effective - 2.376_873_400_022_385_3).abs() < 2e-14);
    // .47 is between transformed Winf=.425623... and raw sigma/r=.505826...
    assert_eq!(sample(p, 2.0, &[0.47]), ReactionTime::NoReaction);
    // W=Winf*erfc(.5) gives a known positive time from the independent oracle.
    let outcome = sample(p, 2.0, &[0.204_086_365_403_211_07]);
    let ReactionTime::After(time) = outcome else {
        panic!("charged separated pair should have a finite delay");
    };
    assert!((time.ps() - 126.275_633_951_913_72).abs() < 3e-10);
    assert_eq!(
        sample(
            p,
            f64::from_bits(p.effective_radius().nm().to_bits() - 1),
            &[]
        ),
        ReactionTime::Immediate
    );
    assert!(matches!(
        sample(p, p.effective_radius().nm(), &[0.25]),
        ReactionTime::After(_)
    ));
    assert_eq!(
        sample(p, p.effective_radius().nm(), &[0.99]),
        ReactionTime::NoReaction
    );
}

#[test]
fn extreme_draws_and_separations_preserve_finite_tails_or_typed_errors() {
    let p = option3_parameters(1);
    let sigma = p.effective_radius().nm();
    for draw in [u64_to_open01(0), 0.5 - f64::EPSILON / 2.0] {
        assert!(matches!(
            sample(p, 2.0 * sigma, &[draw]),
            ReactionTime::After(time) if time.ps().is_finite() && time.ps() > 0.0
        ));
    }
    assert!(matches!(
        sample(p, f64::from_bits(sigma.to_bits() + 1), &[0.25]),
        ReactionTime::After(time) if time.ps() > 0.0 && time.ps() < 1e-28
    ));
    for id in [1, 11, 9] {
        let p = option3_parameters(id);
        assert_eq!(sample(p, 1e200, &[0.75]), ReactionTime::NoReaction);
        let [a, b] = isolated_pair(p, 0.2, 0);
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let invalid = Molecule::new(
                b.id(),
                SpeciesInstance {
                    position: Vec3::new(bad, 0.0, 0.0),
                    ..*b.instance()
                },
            );
            assert!(matches!(
                sample_independent_reaction_time(
                    MoleculePair::new(&a, &invalid).unwrap(),
                    &p,
                    &mut ScriptedRng::new(&[]),
                ),
                Err(ChemistryError::InvalidIrtInput { .. })
            ));
        }
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
        assert!(matches!(
            sample_independent_reaction_time(
                MoleculePair::new(&a, &b).unwrap(),
                &p,
                &mut ScriptedRng::new(&[]),
            ),
            Err(ChemistryError::IrtDomain { .. })
        ));
    }
}
