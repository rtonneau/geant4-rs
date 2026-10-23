//! Isolated option3 independent reaction times in unbounded three-dimensional water.
//!
//! Normative source: Geant4 11.4.1,
//! `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc:286-311`,
//! `GetIndependentReactionTime`, reaction type 0. This is not IRT-syn.
//! For TDC, the source first checks strict contact, then transforms charged
//! distances and samples `W_inf = sigma / r_eff` and
//! `t = (r_eff - sigma)^2 / (4 D erfc_inv(W / W_inf)^2)`.
//! The charged law reproduces Geant4's mapping, not a general electrostatic solution.
//!
//! Intentional numerical differences: open-(0,1) uniforms exclude theoretical
//! endpoint draws; finite arithmetic is checked; explicit outcomes replace the
//! C++ negative-time sentinel. The source's exact-zero distance substitution of
//! 0.001 nm is retained, even for synthetic reaction radii smaller than that.

use crate::error::ChemistryError;
use crate::molecule::MoleculePair;
use crate::reaction_parameters::TwoBodyReactionParameters;
use crate::special::erfc_inv;
use dna_core::sampling::uniform;
use dna_core::units::{Length, Time};
use rand::Rng;

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
/// Only TDC kinetics are currently supported. Arithmetic overflow/underflow of
/// a selected positive reaction time is an error, never [`ReactionTime::NoReaction`].
pub fn sample_independent_reaction_time(
    pair: MoleculePair<'_>,
    parameters: &TwoBodyReactionParameters,
    rng: &mut impl Rng,
) -> Result<ReactionTime, ChemistryError> {
    let reaction_id = parameters.reaction_id();
    if parameters.pdc().is_some() {
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
