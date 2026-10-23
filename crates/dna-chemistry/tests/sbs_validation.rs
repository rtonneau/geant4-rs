//! IRT/SBS pre-registration (2026-10-23; fixed before the first statistical run).
//!
//! One family of six comparisons for option3 H + H (TDC reaction id 1) at
//! r0=0.2 nm and tau=[0.25,0.5,1,2,4,16]. As in V9,
//! tau=4*D*t/(r0-sigma)^2, D=D_H+D_H, and the time scale below is
//! (r0-sigma)^2/(4*D) in ps. The six CDF probabilities below are V9's
//! independent analytic values, fixed here before sampling. N=100,000
//! independent pairs per sampler. IRT uses seed 20_261_023_006 (the V9
//! stream); SBS uses independent seed 20_261_023_102. For both, history id
//! is reaction_id*N+pair_index and the stage is Chemistry.
//!
//! The originally registered agreement rule was |F_IRT-F_SBS| <=
//! 3*sqrt(2*F_V9*(1-F_V9)/N): the combined binomial standard error for two
//! independent samples with the pre-registered V9 probability. This is a
//! pointwise three-sigma rule across the fixed six-point family, not a
//! simultaneous-coverage claim. It failed only at tau=0.25. A subsequent
//! Geant4 11.4.1 executable run showed an even larger early SBS deficit, so
//! the project chose executable fidelity over forcing agreement with IRT.
//! This regression therefore requires the registered bound to be exceeded at
//! tau=0.25 and satisfied at the other five points. Seeds, points, analytic
//! probabilities, sample count and bounds are unchanged; the classification
//! was recorded after the failure and is not claimed as preregistered.
//! Survival is N minus reacted-by-time for
//! each sampler, including every IRT NoReaction and every SBS pair still
//! unreacted at the observation boundary. The survival residual and bound
//! equal the CDF residual and bound. There is no fitted parameter, seed
//! search, or tolerance adjustment. This test does not read the Geant4 CSV.
//! At separations indistinguishable from contact at binary64 coordinate
//! resolution, the SBS trajectory is absorbed at its current time; this
//! prevents an adaptive step below coordinate resolution from repeating.
//!
//! Source: Geant4 11.4.1 G4DNAIRT.cc and
//! G4DNAMoleculeEncounterStepper.cc/G4DNASmoluchowskiReactionModel.cc.

mod common;

use common::{isolated_pair, option3_parameters};
use dna_chemistry::irt::{ReactionTime, sample_independent_reaction_time};
use dna_chemistry::molecule::MoleculePair;
use dna_chemistry::sbs::step_tdc_pair;
use dna_chemistry::species::SpeciesTable;
use dna_core::rng::{RngStreams, Stage};
use dna_core::species::Species;
use dna_core::units::Time;
use dna_core::vec3::Vec3;

const N: usize = 100_000;
const REACTION_ID: u16 = 1;
const DISTANCE_NM: f64 = 0.2;
const IRT_SEED: u64 = 20_261_023_006;
const SBS_SEED: u64 = 20_261_023_102;
const TAU: [f64; 6] = [0.25, 0.5, 1.0, 2.0, 4.0, 16.0];
const TIME_SCALE_PS: f64 = 0.197_050_889_166_177_16;
const V9_CDF: [f64; 6] = [
    0.002_220_827_343_701_086,
    0.021_601_957_061_711_33,
    0.074_680_241_949_397_87,
    0.150_648_092_540_691_45,
    0.227_650_131_308_291_4,
    0.343_575_287_428_839_3,
];

#[test]
fn h_h_irt_and_source_faithful_sbs_record_the_early_discrepancy() {
    let parameters = option3_parameters(REACTION_ID);
    assert_eq!(parameters.reactants(), [Species::H, Species::H]);
    assert!(parameters.pdc().is_none());
    let hydrogen_diffusion = SpeciesTable::option3().unwrap().get(Species::H).diffusivity;
    assert_eq!(
        parameters.relative_diffusivity().nm2_per_ps(),
        2.0 * hydrogen_diffusion.nm2_per_ps()
    );
    let derived_scale = (DISTANCE_NM - parameters.effective_radius().nm()).powi(2)
        / (4.0 * parameters.relative_diffusivity().nm2_per_ps());
    assert!((derived_scale - TIME_SCALE_PS).abs() < 1e-14);
    let thresholds = TAU.map(|tau| tau * TIME_SCALE_PS);

    let irt_streams = RngStreams::new(IRT_SEED);
    let sbs_streams = RngStreams::new(SBS_SEED);
    let mut irt_reacted = [0usize; 6];
    let mut irt_surviving = [0usize; 6];
    let mut sbs_reacted = [0usize; 6];
    let mut sbs_surviving = [0usize; 6];
    let mut irt_no_reaction = 0;

    for index in 0..N {
        let history_id = u64::from(REACTION_ID) * N as u64 + index as u64;
        let [a, b] = isolated_pair(parameters, DISTANCE_NM, history_id);
        let mut irt_rng = irt_streams.stream(history_id, Stage::Chemistry).unwrap();
        let irt_time = sample_independent_reaction_time(
            MoleculePair::new(&a, &b).unwrap(),
            &parameters,
            &mut irt_rng,
        )
        .unwrap_or_else(|error| panic!("IRT pair {index}: {error}"));
        match irt_time {
            ReactionTime::After(time) => {
                for point in 0..6 {
                    if time.ps() <= thresholds[point] {
                        irt_reacted[point] += 1;
                    } else {
                        irt_surviving[point] += 1;
                    }
                }
            }
            ReactionTime::NoReaction => {
                irt_no_reaction += 1;
                for count in &mut irt_surviving {
                    *count += 1;
                }
            }
            ReactionTime::Immediate => panic!("IRT pair {index}: initially separated"),
        }

        let mut sbs_rng = sbs_streams.stream(history_id, Stage::Chemistry).unwrap();
        let mut positions = [a.instance().position, b.instance().position];
        let mut elapsed_ps = 0.0;
        let mut elapsed_error_ps = 0.0;
        let mut reacted = false;
        for point in 0..6 {
            while !reacted && elapsed_ps < thresholds[point] {
                if at_coordinate_resolution_of_contact(
                    positions,
                    parameters.effective_radius().nm(),
                ) {
                    reacted = true;
                    break;
                }
                let remaining = Time::from_ps(thresholds[point] - elapsed_ps);
                let step = step_tdc_pair(
                    positions,
                    [hydrogen_diffusion; 2],
                    parameters.effective_radius(),
                    remaining,
                    &mut sbs_rng,
                )
                .unwrap_or_else(|error| panic!("SBS pair {index}, tau={}: {error}", TAU[point]));
                assert!(step.elapsed.ps() > 0.0, "SBS pair {index}: zero-time step");
                positions = step.positions;
                if step.capped {
                    elapsed_ps = thresholds[point];
                    elapsed_error_ps = 0.0;
                } else {
                    // Near contact, adaptive steps may be below one ulp of
                    // elapsed time. Compensate the sum rather than lose them.
                    let increment = step.elapsed.ps() - elapsed_error_ps;
                    let next_elapsed_ps = elapsed_ps + increment;
                    elapsed_error_ps = (next_elapsed_ps - elapsed_ps) - increment;
                    elapsed_ps = next_elapsed_ps;
                }
                reacted = step.reacted;
            }
            if reacted {
                sbs_reacted[point] += 1;
            } else {
                sbs_surviving[point] += 1;
            }
        }
    }

    let mut exceeds_registered_bound = [false; 6];
    for point in 0..6 {
        assert_eq!(irt_reacted[point] + irt_surviving[point], N);
        assert_eq!(sbs_reacted[point] + sbs_surviving[point], N);
        assert!(irt_surviving[point] >= irt_no_reaction);
        let irt_cdf = irt_reacted[point] as f64 / N as f64;
        let sbs_cdf = sbs_reacted[point] as f64 / N as f64;
        let residual = (irt_cdf - sbs_cdf).abs();
        let bound = 3.0 * (2.0 * V9_CDF[point] * (1.0 - V9_CDF[point]) / N as f64).sqrt();
        eprintln!(
            "H+H tau={}: IRT={irt_cdf:.6} SBS={sbs_cdf:.6} |CDF/S residual|={residual:.6} bound={bound:.6}",
            TAU[point]
        );
        if residual > bound {
            exceeds_registered_bound[point] = true;
        }
    }
    assert_eq!(
        exceeds_registered_bound,
        [true, false, false, false, false, false],
        "source-faithful SBS discrepancy signature changed"
    );
}

fn at_coordinate_resolution_of_contact(positions: [Vec3; 2], radius_nm: f64) -> bool {
    let delta = positions[0] - positions[1];
    let separation = delta.x.hypot(delta.y).hypot(delta.z);
    let coordinate_scale = positions
        .into_iter()
        .flat_map(|p| [p.x.abs(), p.y.abs(), p.z.abs()])
        .fold(radius_nm, f64::max);
    separation - radius_nm <= 8.0 * f64::EPSILON * coordinate_scale
}
