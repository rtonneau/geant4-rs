//! Conditional parent and source-ordered product placement for option3.
//!
//! Normative source: Geant4 11.4.1
//! `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc:393-495`,
//! `MakeReaction`; the unary fake partner is created at lines 277-280.
//! The caller supplies parent positions at a common sampling time and an elapsed
//! delay. This module does not diffuse a history, assign IDs, or schedule products.
//!
//! **Source limitation:** line 433 passes `s12 + s22*s22/s12`, where
//! `s12 = 2 D1 dt` and `s22 = 2 D2 dt`, as a Gaussian *standard deviation*.
//! It has dimensions of length squared, although it is added to a position.
//! We retain the approved literal expression, including its native-unit scale:
//! Geant4 uses mm/ns, so with s12/s22 in nm^2 the numerical standard deviation
//! in nm is `(s12 + s22*s22/s12) * 1e-6`. This is not a Brownian width and
//! cannot be made unit invariant by taking a square root. See
//! `source/global/HEPRandom/include/Randomize.hh:47`,
//! `source/externals/clhep/include/CLHEP/Random/RandGaussQ.icc:55-57`, and
//! `source/externals/clhep/include/CLHEP/Units/SystemOfUnits.h:63,82,138-142`.
//!
//! Coincident parents retain the source's +0.001 nm z offset of parent B.
//! The source sets absolute polar angles, rather than rotating about the initial
//! separation; that choice and its asymmetric S2 expression are preserved.
//! `CLHEP/Vector/ThreeVector.icc:128-130,149-161` makes the azimuth zero
//! when the initial separation lies exactly on z, including coincident inputs:
//! `setPhi` cannot change a zero transverse component before `setTheta`.
//! Unrepresentable arithmetic and the source's early-return ambiguity at zero
//! alpha or zero total diffusion are typed errors, never invented displacements.
//! Normals use the project's Box-Muller helper, not CLHEP's random sequence.

use crate::molecule::{Molecule, MoleculePair};
use crate::reaction_network::CompiledReaction;
use crate::reactions::ReactionParticipant;
use crate::{ChemistryError, Species};
use g4_core::sampling::{gaussian_pair, uniform};
use g4_core::units::{Diffusivity, Length, Time};
use g4_core::vec3::Vec3;
use rand::Rng;

/// A source product and its position, before the caller assigns identity and time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacedProduct {
    pub species: Species,
    /// Position in nm.
    pub position: Vec3,
}

/// Conditional parent sites and products in the source table's product order.
#[derive(Debug, Clone, PartialEq)]
pub struct ReactionPlacement {
    /// Parent A then B; B is the fake bulk partner for a unary channel.
    pub parent_positions: [Vec3; 2],
    /// Empty for a zero-product reaction; implicit water is not a mobile product.
    pub products: Vec<PlacedProduct>,
}

/// Place a selected pair reaction without mutating either molecule.
///
/// Parents must match **source reactant order** (unlike the symmetric IRT clock).
/// The formula is asymmetric in the parents, so swapping them silently would
/// change its meaning. Molecule record times are not used: positions must already
/// refer to a common sampling time, and `elapsed` is measured from that time.
pub fn place_pair_reaction(
    pair: MoleculePair<'_>,
    reaction: &CompiledReaction,
    elapsed: Time,
    rng: &mut impl Rng,
) -> Result<ReactionPlacement, ChemistryError> {
    let id = reaction.source().id;
    let parameters = reaction
        .pair()
        .ok_or(ChemistryError::UnsupportedReactionOrder { reaction_id: id })?;
    let parents = [pair.first().instance(), pair.second().instance()];
    if parents.map(|p| p.species) != parameters.reactants() {
        return Err(ChemistryError::ReactionSpeciesMismatch { reaction_id: id });
    }
    let positions = conditional_parent_positions(
        id,
        parents.map(|p| p.position),
        reaction.reactant_diffusivities(),
        parameters.effective_radius(),
        elapsed,
        rng,
    )?;
    place_products(reaction, positions)
}

/// Place a selected unary reaction with Geant4's immobile fake bulk partner.
///
/// The fake partner starts at the mobile parent's position; the source's
/// coincident-site offset and zero-D2 branch then apply, even when no products
/// are emitted. No Brownian draw is made. Disabled channels are rejected.
pub fn place_unary_reaction(
    parent: &Molecule,
    reaction: &CompiledReaction,
    elapsed: Time,
    rng: &mut impl Rng,
) -> Result<ReactionPlacement, ChemistryError> {
    let id = reaction.source().id;
    let parameters = reaction
        .unary()
        .ok_or(ChemistryError::UnsupportedReactionOrder { reaction_id: id })?;
    if parent.instance().species != parameters.mobile() {
        return Err(ChemistryError::ReactionSpeciesMismatch { reaction_id: id });
    }
    if parameters.is_disabled() {
        return Err(ChemistryError::DisabledReactionPlacement { reaction_id: id });
    }
    // Effective radius is never used for D2=0; the unary observed rate has units
    // s^-1, so deriving a bimolecular radius from it would be meaningless.
    let positions = conditional_parent_positions(
        id,
        [parent.instance().position; 2],
        reaction.reactant_diffusivities(),
        Length::from_nm(0.0),
        elapsed,
        rng,
    )?;
    place_products(reaction, positions)
}

/// Evaluate Geant4's conditional parent-position branch in unbounded water.
///
/// Positions are in nm; diffusivities, radius and elapsed time use typed units.
/// Inputs must be finite, diffusion/time/radius nonnegative and total diffusion
/// positive. Radius is used only when both parents diffuse over positive time;
/// zero radius in that branch is the source's ambiguous alpha=0 early return.
/// A coincident B receives +0.001 nm in z even at zero elapsed time. A positive
/// delay with one immobile parent puts both at that parent's (possibly offset)
/// site. Other positive-delay calls consume six uniforms: four for three
/// independent normals (one unused normal), then azimuth and polar angle.
/// Errors may consume RNG but never mutate the input positions.
pub fn conditional_parent_positions(
    reaction_id: u16,
    mut positions: [Vec3; 2],
    diffusivities: [Diffusivity; 2],
    effective_radius: Length,
    elapsed: Time,
    rng: &mut impl Rng,
) -> Result<[Vec3; 2], ChemistryError> {
    let id = reaction_id;
    for position in positions {
        finite_position(id, "input parent position", position)?;
    }
    let [d1, d2] = diffusivities.map(Diffusivity::nm2_per_ps);
    let dt = elapsed.ps();
    let radius = effective_radius.nm();
    for (field, value) in [
        ("D1", d1),
        ("D2", d2),
        ("elapsed", dt),
        ("effective radius", radius),
    ] {
        crate::reaction_parameters::input(id, field, value, true)?;
    }
    let total = positive(id, "total diffusivity", d1 + d2)?;
    if positions[0] == positions[1] {
        positions[1].z += 1e-3;
        finite_position(id, "coincident-parent offset", positions[1])?;
        if positions[0] == positions[1] {
            return Err(domain(id, "unrepresentable coincident-parent offset"));
        }
    }
    let delta = positions[0] - positions[1];
    let r0 = positive(
        id,
        "parent separation",
        delta.x.hypot(delta.y).hypot(delta.z),
    )?;
    if dt == 0.0 {
        return Ok(positions);
    }
    if d1 == 0.0 {
        return Ok([positions[0]; 2]);
    }
    if d2 == 0.0 {
        return Ok([positions[1]; 2]);
    }

    let s12 = positive(id, "2 D1 dt", 2.0 * d1 * dt)?;
    let s22 = positive(id, "2 D2 dt", 2.0 * d2 * dt)?;
    let alpha = positive(id, "conditional alpha", radius * r0 / (2.0 * total * dt))?;
    let ratio = positive(id, "conditional diffusion ratio", s12 / s22)?;
    // Retain the source's variance-like *standard deviation*, converting its
    // mm-native numerical value back to nm. Do not replace this by sqrt(...).
    let sigma = positive(
        id,
        "literal Gaussian width",
        (s12 + s22 * (s22 / s12)) * 1e-6,
    )?;
    let mean = finite_position(
        id,
        "conditional S2 mean",
        positions[0] + positions[1] * ratio,
    )?;
    let (x, y) = gaussian_pair(rng);
    let (z, _) = gaussian_pair(rng);
    let s2 = finite_position(id, "conditional S2", mean + Vec3::new(x, y, z) * sigma)?;

    let sampled_phi = std::f64::consts::TAU * uniform(rng);
    let phi = if delta.x == 0.0 && delta.y == 0.0 {
        0.0
    } else {
        sampled_phi
    };
    let u = uniform(rng);
    // expm1/ln_1p preserve the literal inverse angular CDF at small alpha.
    // Divide before subtracting, avoiding the source's overflowing 1/alpha.
    let cos_theta = 1.0 + (-u * -(-2.0 * alpha).exp_m1()).ln_1p() / alpha;
    if !cos_theta.is_finite() || !(-1.0..=1.0).contains(&cos_theta) {
        return Err(domain(id, "conditional polar angle"));
    }
    let sin_theta = cos_theta.acos().sin();
    let (sin_phi, cos_phi) = phi.sin_cos();
    let s1 = Vec3::new(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta) * radius;
    let a = finite_position(
        id,
        "conditional parent A",
        s1 * (d1 / total) + s2 * (d2 / total),
    )?;
    let b = finite_position(id, "conditional parent B", (s2 - s1) * (d2 / total))?;
    Ok([a, b])
}

fn place_products(
    reaction: &CompiledReaction,
    parents: [Vec3; 2],
) -> Result<ReactionPlacement, ChemistryError> {
    let id = reaction.source().id;
    let source_products = &reaction.source().products;
    let mut products = Vec::with_capacity(source_products.len());
    if !source_products.is_empty() {
        let [sqrt_d1, sqrt_d2] = reaction
            .reactant_diffusivities()
            .map(|d| d.nm2_per_ps().sqrt());
        let denominator = positive(id, "reaction-site diffusion weights", sqrt_d1 + sqrt_d2)?;
        let site = finite_position(
            id,
            "reaction site",
            parents[0] * (sqrt_d2 / denominator) + parents[1] * (sqrt_d1 / denominator),
        )?;
        let positions: &[Vec3] = match source_products.len() {
            1 => &[site],
            2 => &parents,
            3 => &[site, parents[0], parents[1]],
            _ => return Err(domain(id, "unsupported product count")),
        };
        for (&product, &position) in source_products.iter().zip(positions) {
            let ReactionParticipant::Mobile(species) = product else {
                return Err(domain(id, "non-mobile product"));
            };
            products.push(PlacedProduct { species, position });
        }
    }
    Ok(ReactionPlacement {
        parent_positions: parents,
        products,
    })
}

fn domain(reaction_id: u16, operation: &'static str) -> ChemistryError {
    ChemistryError::PlacementDomain {
        reaction_id,
        operation,
    }
}

fn positive(id: u16, operation: &'static str, value: f64) -> Result<f64, ChemistryError> {
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err(domain(id, operation))
    }
}

fn finite_position(
    id: u16,
    operation: &'static str,
    position: Vec3,
) -> Result<Vec3, ChemistryError> {
    if [position.x, position.y, position.z]
        .into_iter()
        .all(f64::is_finite)
    {
        Ok(position)
    } else {
        Err(domain(id, operation))
    }
}
