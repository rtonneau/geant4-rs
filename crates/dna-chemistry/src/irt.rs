//! Isolated option3 independent reaction times in unbounded three-dimensional water.
//!
//! Normative source: Geant4 11.4.1,
//! `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc:286-405`,
//! `GetIndependentReactionTime` and `SamplePDC`. This is not IRT-syn.
//! For TDC, the source first checks strict contact, then transforms charged
//! distances and samples `W_inf = sigma / r_eff` and
//! `t = (r_eff - sigma)^2 / (4 D erfc_inv(W / W_inf)^2)`.
//! The charged law reproduces Geant4's mapping, not a general electrostatic solution.
//!
//! Intentional numerical differences: open-(0,1) uniforms exclude theoretical
//! endpoint draws; finite arithmetic is checked; explicit outcomes replace the
//! C++ negative-time sentinel. The source's exact-zero distance substitution of
//! 0.001 nm is retained, even for synthetic reaction radii smaller than that.
//! Neutral PDC preserves the positive-gap rejection envelope, evaluated with
//! scaled variables and logarithmic acceptance to retain its small positive tail.
//! At exact equality the source envelope is singular; the continuum limiting
//! law is sampled directly (see `sample_neutral_pdc`). Exhaustion means exactly
//! 10,000 rejected attempts, rather than the source's sentinel after 10,001.

use crate::error::ChemistryError;
use crate::molecule::MoleculePair;
use crate::reaction_parameters::TwoBodyReactionParameters;
use crate::special::{erfc, erfc_inv};
use dna_core::sampling::uniform;
use dna_core::units::{Length, Time};
use rand::Rng;
use std::f64::consts::{PI, SQRT_2};

/// An elapsed-time outcome for the current pair configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReactionTime {
    /// Reaction with zero elapsed time.
    Immediate,
    /// Reaction after a finite, strictly positive elapsed time.
    After(Time),
    /// The pair never reacts in the isolated infinite-volume model.
    NoReaction,
}

/// Sample one independent reaction-time outcome using caller-owned randomness.
///
/// The molecule identities must differ (enforced by [`MoleculePair`]), species
/// must match the supplied reaction in either order, and coordinates must be
/// finite. The caller supplies positions at a common current time. Record times
/// are not added to the result or used to advance either molecule: [`ReactionTime::After`]
/// is an elapsed delay, not a scheduler timestamp.
///
/// TDC contact is strictly `sigma > distance` and consumes no randomness. All
/// other valid TDC configurations consume exactly one open uniform, including
/// equality: neutral equality returns [`ReactionTime::Immediate`] after drawing.
/// Charged distances are transformed only after the contact test. In particular,
/// repulsive transforms can give `W_inf > 1`; the source's inverse law is kept
/// without clamping or a second contact test.
///
/// Neutral PDC overlap consumes one contact-probability draw. Equality and
/// separation consume an ultimate-probability draw; if successful, a positive
/// gap consumes proposal/acceptance draw pairs (at most 10,000), while equality
/// consumes exponential/half-normal draws for the limiting continuum law.
/// Charged PDC is unsupported. Arithmetic overflow/underflow of a selected
/// positive reaction time is an error, never [`ReactionTime::NoReaction`].
pub fn sample_independent_reaction_time(
    pair: MoleculePair<'_>,
    parameters: &TwoBodyReactionParameters,
    rng: &mut impl Rng,
) -> Result<ReactionTime, ChemistryError> {
    let reaction_id = parameters.reaction_id();
    if parameters.pdc().is_some() && parameters.onsager_radius().nm() != 0.0 {
        return Err(ChemistryError::UnsupportedIrtKinetics { reaction_id });
    }
    let first = pair.first().instance();
    let second = pair.second().instance();
    let species = [first.species, second.species];
    if species != parameters.reactants() && [species[1], species[0]] != parameters.reactants() {
        return Err(ChemistryError::ReactionSpeciesMismatch { reaction_id });
    }
    for molecule in [pair.first(), pair.second()] {
        let position = molecule.instance().position;
        for (field, value) in [
            ("position.x", position.x),
            ("position.y", position.y),
            ("position.z", position.z),
        ] {
            if !value.is_finite() {
                return Err(ChemistryError::InvalidIrtInput {
                    id: molecule.id(),
                    field,
                    value,
                });
            }
        }
    }
    let displacement = first.position - second.position;
    // `Vec3::norm` squares first; hypot also handles large finite separations.
    let distance = displacement.x.hypot(displacement.y).hypot(displacement.z);
    if !distance.is_finite() {
        return Err(ChemistryError::IrtDomain {
            reaction_id,
            operation: "pair separation",
        });
    }
    let distance = if distance == 0.0 { 1e-3 } else { distance };
    if parameters.pdc().is_some() {
        return sample_neutral_pdc(distance, parameters, rng);
    }
    let sigma = parameters.effective_radius().nm();
    if sigma > distance {
        return Ok(ReactionTime::Immediate);
    }

    let effective = parameters
        .effective_distance(Length::from_nm(distance))?
        .nm();
    let winf = sigma / effective;
    if !winf.is_finite() || winf <= 0.0 {
        return Err(ChemistryError::IrtDomain {
            reaction_id,
            operation: "ultimate reaction probability",
        });
    }
    let draw = uniform(rng);
    if draw >= winf {
        return Ok(ReactionTime::NoReaction);
    }
    let gap = effective - sigma;
    if gap == 0.0 {
        return Ok(ReactionTime::Immediate);
    }
    // Divide by W_inf instead of multiplying r_eff * W: equivalent, with no
    // large intermediate distance product. The inverse has a strict finite domain.
    let inverse = erfc_inv(draw / winf)?;
    // Scale before squaring to avoid overflow when gap^2 itself is too large.
    let root_time =
        0.5 * gap.abs() / parameters.relative_diffusivity().nm2_per_ps().sqrt() / inverse;
    let time = root_time * root_time;
    if !time.is_finite() || time <= 0.0 {
        return Err(ChemistryError::IrtDomain {
            reaction_id,
            operation: "TDC elapsed time",
        });
    }
    Ok(ReactionTime::After(Time::from_ps(time)))
}

fn sample_neutral_pdc(
    distance: f64,
    parameters: &TwoBodyReactionParameters,
    rng: &mut impl Rng,
) -> Result<ReactionTime, ChemistryError> {
    let reaction_id = parameters.reaction_id();
    let sigma = parameters.reaction_radius().nm();
    if sigma > distance {
        return Ok(if uniform(rng) < parameters.contact_probability() {
            ReactionTime::Immediate
        } else {
            ReactionTime::NoReaction
        });
    }
    let rates = parameters.pdc().expect("PDC dispatch requires PDC rates");
    let observed = parameters.observed_rate().nm3_per_ps();
    let winf = (sigma / distance) * (observed / rates.diffusion_rate().nm3_per_ps());
    if !winf.is_finite() || winf <= 0.0 {
        return Err(ChemistryError::IrtDomain {
            reaction_id,
            operation: "PDC ultimate reaction probability",
        });
    }
    if uniform(rng) >= winf {
        return Ok(ReactionTime::NoReaction);
    }

    // G4DNAIRT: a=(k_act/k_obs)/sigma, b=(r0-sigma)/2, elapsed=X/D.
    let a = (rates.activation_rate().nm3_per_ps() / observed) / sigma;
    let b = (distance - sigma) / 2.0;
    pdc_positive(reaction_id, "PDC a", a)?;
    let root_diffusion = parameters.relative_diffusivity().nm2_per_ps().sqrt();
    if b == 0.0 {
        // The b=0 conditional survival is erfcx(y), y=a*sqrt(X).
        // erfcx(y)=2/sqrt(pi) integral_0^inf exp(-z^2-2*y*z) dz.
        // Thus y=E/(2 Z), E exponential(1), Z=erfc_inv(V) half-normal.
        // This exact continuum limit avoids the source's p=0, q=infinity;
        // it is not a discrete contact reaction or IRT-syn activation delay.
        let exponential = -uniform(rng).ln();
        let half_normal = erfc_inv(uniform(rng))?;
        let root_time = (0.5 * exponential / half_normal) / a / root_diffusion;
        return pdc_time(reaction_id, root_time);
    }

    // Source s=sqrt(2b/a), p=2s, q=2/s, M=max(1/a^2,3b/a).
    // e=s^2/M=min(2ab,2/3) stays bounded, and the proposal split is e/(1+e).
    // Work with sqrt(X), avoiding the overflow of M, p+qM, and X itself.
    let s = SQRT_2 * b.sqrt() / a.sqrt();
    let e = (2.0 * (a * b)).min(2.0 / 3.0);
    let split = e / (1.0 + e);
    pdc_positive(reaction_id, "PDC envelope scale", s)?;
    pdc_positive(reaction_id, "PDC envelope split", split)?;
    for _ in 0..10_000 {
        let proposal = uniform(rng);
        let root_x = if proposal < split {
            s * (proposal / split)
        } else {
            s / (1.0 + e) / (1.0 - proposal)
        };
        let acceptance = uniform(rng);
        pdc_positive(reaction_id, "PDC proposal", root_x)?;

        // lambda=exp(-u^2)*(1-sqrt(pi)*v*erfcx(u+v)),
        // u=b/sqrt(X), v=a*sqrt(X). Replace the cancellation by
        // u/z + (v/z)*R(z), R(z)=1-sqrt(pi)*z*erfcx(z).
        let log_u = b.ln() - root_x.ln();
        let log_v = a.ln() + root_x.ln();
        let log_z = log_add(log_u, log_v);
        let log_bracket = log_add(log_u, log_v + log_erfcx_remainder(log_z)?) - log_z;
        let u = b / root_x;
        let mut log_acceptance = -u * u + log_bracket;
        if root_x > s {
            // Upper envelope accepts U*M/X <= lambda, or U <= lambda*X/M.
            log_acceptance += 2.0 * (root_x.ln() - s.ln()) + e.ln();
        }
        if log_acceptance.is_nan() {
            return Err(ChemistryError::IrtDomain {
                reaction_id,
                operation: "PDC acceptance probability",
            });
        }
        if acceptance.ln() <= log_acceptance {
            return pdc_time(reaction_id, root_x / root_diffusion);
        }
    }
    Err(ChemistryError::IrtConvergence {
        reaction_id,
        attempts: 10_000,
    })
}

fn pdc_positive(
    reaction_id: u16,
    operation: &'static str,
    value: f64,
) -> Result<(), ChemistryError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(ChemistryError::IrtDomain {
            reaction_id,
            operation,
        })
    }
}

fn pdc_time(reaction_id: u16, root_time: f64) -> Result<ReactionTime, ChemistryError> {
    let time = root_time * root_time;
    pdc_positive(reaction_id, "PDC elapsed time", time)?;
    Ok(ReactionTime::After(Time::from_ps(time)))
}

fn log_add(a: f64, b: f64) -> f64 {
    let larger = a.max(b);
    larger + (a.min(b) - larger).exp().ln_1p()
}

// R(z) is positive. For z<8, erfc is representable and direct scaling is safe.
// Above 8, repeated integration by parts of the erfc integral gives
// R(z) = 1/(2z^2) * [1 - 3/(2z^2) + 15/(4z^4) - ...].
// Terms alternate and bound the remainder; stop below binary64 precision (or
// before they grow). Logarithms retain R even when z or z^2 would overflow.
fn log_erfcx_remainder(log_z: f64) -> Result<f64, ChemistryError> {
    if log_z < 8.0_f64.ln() {
        let z = log_z.exp();
        let erfcx = (z * z).exp() * erfc(z)?;
        return Ok((-PI.sqrt() * z).mul_add(erfcx, 1.0).ln());
    }
    let inverse_twice_square = 0.5 * (-2.0 * log_z).exp();
    let mut term = 1.0;
    let mut sum = 1.0;
    for n in 1..128 {
        let next = -term * f64::from(2 * n + 1) * inverse_twice_square;
        if next.abs() > term.abs() {
            break;
        }
        sum += next;
        if next.abs() <= f64::EPSILON * sum {
            break;
        }
        term = next;
    }
    Ok(-2.0_f64.ln() - 2.0 * log_z + sum.ln())
}
