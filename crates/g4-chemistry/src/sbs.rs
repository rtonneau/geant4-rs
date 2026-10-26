//! Validation-only step-by-step (SBS) oracle for an isolated TDC pair.
//!
//! Normative sources: Geant4 11.4.1, under
//! `source/processes/electromagnetic/dna/models/src/`:
//! `G4DNAMoleculeEncounterStepper.cc` (`Utils`, `CalculateStep`) and
//! `G4DNASmoluchowskiReactionModel.cc` (`FindReaction`). The encounter constant
//! is the executable `8 * (D_A + D_B + 2 * sqrt(D_A * D_B))`, despite the
//! nearby comment saying 16. For H+H this is `32 * D_H`.
//!
//! This kernel has no scheduler, neighbour search, products, electrostatic
//! drift, PDC kinetics, or minimum timestep. Callers supply the TDC effective
//! radius and synchronized positions in infinite water, and repeatedly step
//! until reaction or the observation boundary. Both particles diffuse through
//! the existing Brownian kernel; random sequences are not bitwise C++ replicas.
//! Inclusive contact (`r <= R`) intentionally differs from production IRT.
//!
//! The bridge preserves a source quirk: `FindReaction` resumes its partial
//! squared-distance loop at the index already added before `break`, counting
//! that coordinate twice. This affects only the bridge post-step distance.
//! It is retained here for executable-source comparison, not endorsed as a
//! rotationally invariant Brownian-bridge law. Finite-domain checks and open
//! uniforms are explicit numerical differences from C++.

use crate::diffusion::diffuse;
use crate::error::ChemistryError;
use g4_core::sampling::uniform;
use g4_core::units::{Diffusivity, Length, Time};
use g4_core::vec3::Vec3;
use rand::Rng;
use thiserror::Error;

/// Input or numerical failure in the validation-only SBS oracle.
#[derive(Debug, Error)]
pub enum SbsError {
    #[error("invalid SBS {field}: {value}")]
    InvalidInput { field: &'static str, value: f64 },
    #[error("SBS arithmetic outside the finite domain: {operation}")]
    Domain { operation: &'static str },
    #[error(transparent)]
    Diffusion(#[from] ChemistryError),
}

/// One elapsed step, with both diffused positions even when a reaction occurred.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SbsStep {
    pub positions: [Vec3; 2],
    pub elapsed: Time,
    pub reacted: bool,
    /// True when the observation boundary is at or before the adaptive step.
    pub capped: bool,
}

/// Compute `(r - R)^2 / [8 (D_A + D_B + 2 sqrt(D_A D_B))]` outside contact.
///
/// Inclusive contact returns zero. Inputs must be finite, `r >= 0`, `R > 0`,
/// each diffusivity nonnegative, and their sum positive. A separated pair
/// whose positive timestep cannot be represented returns an error.
pub fn adaptive_time_step(
    separation: Length,
    radius: Length,
    diffusivities: [Diffusivity; 2],
) -> Result<Time, SbsError> {
    let r = separation.nm();
    validate("separation", r, true)?;
    let constant = encounter_constant(radius, diffusivities)?;
    if r <= radius.nm() {
        return Ok(Time::from_ps(0.0));
    }
    let gap = r - radius.nm();
    let time = gap * gap / constant;
    if !time.is_finite() || time <= 0.0 {
        return Err(SbsError::Domain {
            operation: "adaptive timestep",
        });
    }
    Ok(Time::from_ps(time))
}

/// Advance an isolated TDC pair by at most the remaining observation time.
///
/// Positions, diffusivities, radius and remaining time are checked before any
/// draws. Initial contact reacts immediately without draws; a separated pair
/// with zero remaining time stays unchanged. Each mobile particle consumes
/// four uniforms via [`diffuse`]. Endpoint contact needs no further draw.
/// A positive step with separated endpoints consumes a bridge uniform only
/// when it ends at the observation boundary, including exact equality with
/// the adaptive step. It accepts at `u <= exp(-(r_pre-R)(r_post-R)/(D dt))`.
/// Here `D = D_A + D_B` and `r_post` preserves the documented source quirk.
/// Reaction times are step endpoints; this function does not interpolate them.
pub fn step_tdc_pair(
    positions: [Vec3; 2],
    diffusivities: [Diffusivity; 2],
    radius: Length,
    remaining: Time,
    rng: &mut impl Rng,
) -> Result<SbsStep, SbsError> {
    for position in positions {
        for (field, value) in [
            ("position.x", position.x),
            ("position.y", position.y),
            ("position.z", position.z),
        ] {
            if !value.is_finite() {
                return Err(SbsError::InvalidInput { field, value });
            }
        }
    }
    validate("remaining time", remaining.ps(), true)?;
    let displacement = positions[0] - positions[1];
    let pre_distance = displacement.x.hypot(displacement.y).hypot(displacement.z);
    let adaptive = adaptive_time_step(Length::from_nm(pre_distance), radius, diffusivities)?;
    let mut result = SbsStep {
        positions,
        elapsed: Time::from_ps(adaptive.ps().min(remaining.ps())),
        reacted: adaptive.ps() == 0.0,
        capped: remaining.ps() <= adaptive.ps(),
    };
    if result.reacted || result.elapsed.ps() == 0.0 {
        return Ok(result);
    }
    for index in 0..2 {
        result.positions[index] =
            diffuse(positions[index], diffusivities[index], result.elapsed, rng)?;
    }
    let post_delta = result.positions[0] - result.positions[1];
    let post_distance = post_delta.x.hypot(post_delta.y).hypot(post_delta.z);
    if !post_distance.is_finite() {
        return Err(SbsError::Domain {
            operation: "post-step separation",
        });
    }
    if post_distance <= radius.nm() {
        result.reacted = true;
    } else if result.capped {
        let bridge_distance = source_bridge_distance(post_delta, radius.nm())?;
        let relative = diffusivities[0].nm2_per_ps() + diffusivities[1].nm2_per_ps();
        // Divide successively to avoid underflow of D * dt at small scales.
        let exponent = (pre_distance - radius.nm()) / relative
            * ((bridge_distance - radius.nm()) / result.elapsed.ps());
        if exponent.is_nan() || exponent < 0.0 {
            return Err(SbsError::Domain {
                operation: "bridge exponent",
            });
        }
        // Positive infinity legitimately gives a zero bridge probability.
        result.reacted = uniform(rng) <= (-exponent).exp();
    }
    Ok(result)
}

fn validate(field: &'static str, value: f64, allow_zero: bool) -> Result<(), SbsError> {
    if !value.is_finite() || value < 0.0 || (!allow_zero && value == 0.0) {
        return Err(SbsError::InvalidInput { field, value });
    }
    Ok(())
}

fn encounter_constant(radius: Length, diffusivities: [Diffusivity; 2]) -> Result<f64, SbsError> {
    validate("radius", radius.nm(), false)?;
    let [a, b] = diffusivities.map(Diffusivity::nm2_per_ps);
    validate("first diffusivity", a, true)?;
    validate("second diffusivity", b, true)?;
    validate("relative diffusivity", a + b, false)?;
    // Separate roots avoid overflow in D_A * D_B without changing the law.
    let constant = 8.0 * (a + b + 2.0 * a.sqrt() * b.sqrt());
    if !constant.is_finite() || constant <= 0.0 {
        return Err(SbsError::Domain {
            operation: "encounter constant",
        });
    }
    Ok(constant)
}

fn source_bridge_distance(delta: Vec3, radius: f64) -> Result<f64, SbsError> {
    let squares = [delta.x * delta.x, delta.y * delta.y, delta.z * delta.z];
    let mut sum = 0.0;
    for (index, square) in squares.iter().enumerate() {
        sum += square;
        if sum > radius * radius {
            // G4DNASmoluchowskiReactionModel::FindReaction resumes at k,
            // not k+1: deliberately add the breaking coordinate a second time.
            sum += squares[index..].iter().sum::<f64>();
            break;
        }
    }
    if !sum.is_finite() {
        return Err(SbsError::Domain {
            operation: "source bridge separation",
        });
    }
    Ok(sum.sqrt())
}
