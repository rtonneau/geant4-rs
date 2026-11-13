//! Isolated IRT-syn kernels, not a scheduler or a production network.
//!
//! Normative source: Geant4 11.4.1 `dna/models/src/`
//! `G4DiffusionControlledReactionModel.cc`, `G4DNAMakeReaction.cc`, and
//! `G4DNAIndependentReactionTimeStepper.cc`. Table parameters are shared with
//! option3 because `G4DNAMolecularReactionTable.cc` derives them identically;
//! neither the option3 charged-distance transform nor its PDC sampler is reused.
//! Free propagation reuses [`crate::diffusion::diffuse`]. Unary channels are
//! deliberately absent: the default source constructor does not register them.
//!
//! Unresolved overlap inheritance in the source stepper is an explicit error,
//! never silently converted to an eventual no-event. Open uniforms and checked
//! arithmetic are numerical deviations; RNG sequences are not CLHEP sequences.

use crate::ChemistryError;
use crate::irt::ReactionTime;
use crate::molecule::MoleculePair;
use crate::placement::{PlacedProduct, ReactionPlacement};
use crate::reaction_network::CompiledReaction;
use crate::reaction_parameters::TwoBodyReactionParameters;
use crate::reactions::ReactionParticipant;
use crate::special::erfc_inv;
use g4_core::sampling::{gaussian_pair, uniform};
use g4_core::units::Time;
use g4_core::vec3::Vec3;
use rand::Rng;

/// Context needed to distinguish initial strict overlap from later overlap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairContext {
    /// The scheduler start-time overlap probability is applicable.
    Initial,
    /// Injection/product or later configuration: the source overlap is unresolved.
    PostStart,
}

/// Explicit semantic random primitives for mapping diagnostics only.
/// Normals are standard-normal *inputs*, not generated or certified by this type.
/// Uniforms must be finite in [0,1); repeated/unknown draw labels are invariant failures.
#[derive(Debug, Clone, Copy)]
pub struct PlacementPrimitives {
    pub normal: Vec3,
    pub azimuth: f64,
    pub polar: f64,
    pub product: f64,
}
trait PlacementDraws {
    fn normal3(&mut self) -> Vec3;
    fn uniform(&mut self, bit: u8) -> f64;
}
struct RandomDraws<'a, R>(&'a mut R);
impl<R: Rng> PlacementDraws for RandomDraws<'_, R> {
    fn normal3(&mut self) -> Vec3 {
        let (x, y) = gaussian_pair(self.0);
        let (z, _) = gaussian_pair(self.0);
        Vec3::new(x, y, z)
    }
    fn uniform(&mut self, _bit: u8) -> f64 {
        uniform(self.0)
    }
}
struct ExplicitDraws {
    input: PlacementPrimitives,
    mask: u8,
}
impl ExplicitDraws {
    fn validate(&self) -> Result<(), ChemistryError> {
        finite_position(0, self.input.normal)?;
        if [self.input.azimuth, self.input.polar, self.input.product]
            .iter()
            .any(|v| !v.is_finite() || !(0.0..1.0).contains(v))
        {
            return Err(domain(0, "invalid diagnostic uniform"));
        }
        Ok(())
    }
    fn consume(&mut self, bits: u8) {
        assert_eq!(self.mask & bits, 0, "repeated semantic placement draw");
        self.mask |= bits;
    }
}
impl PlacementDraws for ExplicitDraws {
    fn normal3(&mut self) -> Vec3 {
        self.consume(7);
        self.input.normal
    }
    fn uniform(&mut self, bit: u8) -> f64 {
        self.consume(bit);
        match bit {
            8 => self.input.azimuth,
            16 => self.input.polar,
            32 => self.input.product,
            _ => panic!("unknown semantic placement draw"),
        }
    }
}

/// Evaluate the independent Rust mapping with explicit inputs and return its
/// semantic draw mask even on an error. Bits: normals x/y/z=1/2/4, phi=8,
/// theta=16, product=32. This is not a production random-number API.
pub fn place_pair_with_primitives(
    pair: MoleculePair<'_>,
    reaction: &CompiledReaction,
    dt: Time,
    input: PlacementPrimitives,
) -> (Result<ReactionPlacement, ChemistryError>, u8) {
    let mut draws = ExplicitDraws { input, mask: 0 };
    let result = draws
        .validate()
        .and_then(|()| place_with_draws(pair, reaction, dt, &mut draws));
    (result, draws.mask)
}

/// Explicit-input version of the parent-only diagnostic, including one immobile
/// partner and mobile coincidence. Returns the consumed mask on success/error.
pub fn parent_positions_with_primitives(
    id: u16,
    positions: [Vec3; 2],
    diffusivities: [g4_core::units::Diffusivity; 2],
    radius: g4_core::units::Length,
    dt: Time,
    input: PlacementPrimitives,
) -> (Result<[Vec3; 2], ChemistryError>, u8) {
    let mut draws = ExplicitDraws { input, mask: 0 };
    let result = draws
        .validate()
        .and_then(|()| parents_with_draws(id, positions, diffusivities, radius, dt, &mut draws));
    (result, draws.mask)
}

/// Sample an uncensored elapsed clock for two distinct synchronized molecules.
/// Exact contact is sent through the encounter law. Accepted initial overlap
/// returns zero; rejected initial overlap and every post-start overlap return
/// an error because the source can inherit another pair's sampled time.
/// NoReaction here is eventual nonreaction, never horizon censoring.
pub fn sample_pair(
    pair: MoleculePair<'_>,
    parameters: &TwoBodyReactionParameters,
    context: PairContext,
    rng: &mut impl Rng,
) -> Result<ReactionTime, ChemistryError> {
    let id = parameters.reaction_id();
    let positions = validate_pair(pair, parameters, false)?;
    let delta = positions[0] - positions[1];
    let r = delta.x.hypot(delta.y).hypot(delta.z);
    let a = parameters.effective_radius().nm();
    if !r.is_finite() {
        return Err(domain(id, "pair separation"));
    }
    if r < a {
        if context == PairContext::Initial && uniform(rng) < parameters.contact_probability() {
            return Ok(ReactionTime::Immediate);
        }
        return Err(domain(id, "unresolved source overlap time inheritance"));
    }
    let winf = a / r;
    let u = uniform(rng);
    if u >= winf {
        return Ok(ReactionTime::NoReaction);
    }
    let inverse = erfc_inv(u / winf)?;
    let d = parameters.relative_diffusivity().nm2_per_ps();
    let root = (r - a) / (2.0 * d.sqrt() * inverse);
    let mut time = root * root;
    if !time.is_finite() || (r > a && time <= 0.0) {
        return Err(domain(id, "encounter delay"));
    }
    if let Some(rates) = parameters.pdc() {
        let kd = rates.diffusion_rate().nm3_per_ps();
        let ka = rates.activation_rate().nm3_per_ps();
        let sum = kd + ka;
        if !sum.is_finite() || sum <= 0.0 {
            return Err(domain(id, "activation rate sum"));
        }
        if uniform(rng) > ka / sum {
            return Ok(ReactionTime::NoReaction);
        }
        let y = gaussian_pair(rng).0.abs() * std::f64::consts::SQRT_2;
        // Source explicitly retains X=0 if the Gaussian is exactly zero.
        let x = if y > 0.0 { -uniform(rng).ln() / y } else { 0.0 };
        let f = x * a * (kd / sum);
        let delay = f * f / d;
        if !delay.is_finite() || (x > 0.0 && delay <= 0.0) {
            return Err(domain(id, "activation delay"));
        }
        time += delay;
    }
    if !time.is_finite() {
        return Err(domain(id, "total delay"));
    }
    Ok(if time == 0.0 {
        ReactionTime::Immediate
    } else {
        ReactionTime::After(Time::from_ps(time))
    })
}

/// Ordinary source call-path placement. The reviewed FindReaction path never
/// sets G4DNAMakeReaction::fTimeStep, so it remains zero. This method cannot
/// accidentally activate the positive-duration diagnostic branch.
pub fn place_pair(
    pair: MoleculePair<'_>,
    reaction: &CompiledReaction,
    rng: &mut impl Rng,
) -> Result<ReactionPlacement, ChemistryError> {
    place_pair_diagnostic(pair, reaction, Time::from_ps(0.0), rng)
}

/// Direct-kernel diagnostic ONLY: emulate TestReactibility(dt) then MakeReaction.
/// Positive dt is not ordinary scheduler behavior. The literal Gaussian width
/// is evaluated in native mm/ns units, and its lab-axis angular construction
/// is preserved. Neither dimensional nor rotation invariance is claimed.
/// Products share one random site; parent removal is the caller's responsibility.
pub fn place_pair_diagnostic(
    pair: MoleculePair<'_>,
    reaction: &CompiledReaction,
    dt: Time,
    rng: &mut impl Rng,
) -> Result<ReactionPlacement, ChemistryError> {
    place_with_draws(pair, reaction, dt, &mut RandomDraws(rng))
}

fn place_with_draws(
    pair: MoleculePair<'_>,
    reaction: &CompiledReaction,
    dt: Time,
    draws: &mut impl PlacementDraws,
) -> Result<ReactionPlacement, ChemistryError> {
    let id = reaction.source().id;
    let parameters = reaction
        .pair()
        .ok_or(ChemistryError::UnsupportedReactionOrder { reaction_id: id })?;
    let positions = parents_with_draws(
        id,
        validate_pair(pair, parameters, true)?,
        reaction.reactant_diffusivities(),
        parameters.effective_radius(),
        dt,
        draws,
    )?;
    let [d1, d2] = reaction.reactant_diffusivities().map(|d| d.nm2_per_ps());
    let mut products = Vec::with_capacity(reaction.source().products.len());
    if !reaction.source().products.is_empty() {
        let norm = d1.sqrt() + d2.sqrt();
        let site = positions[0] * (d2.sqrt() / norm) + positions[1] * (d1.sqrt() / norm);
        let u = draws.uniform(32);
        let position = (site + positions[0] * (1.0 - u) + positions[1] * u) * 0.5;
        finite_position(id, position)?;
        for product in &reaction.source().products {
            let ReactionParticipant::Mobile(species) = product else {
                return Err(domain(id, "unsupported bulk product"));
            };
            products.push(PlacedProduct {
                species: *species,
                position,
            });
        }
    }
    Ok(ReactionPlacement {
        parent_positions: positions,
        products,
    })
}

/// Direct source placement diagnostic for explicit diffusion coefficients.
/// Allows one immobile parent to test source boundary behavior without altering
/// the immutable embedded species table. Both immobile parents are rejected.
/// Positive duration remains a dormant-branch diagnostic, not scheduler policy.
pub fn conditional_parent_positions(
    id: u16,
    positions: [Vec3; 2],
    diffusivities: [g4_core::units::Diffusivity; 2],
    radius: g4_core::units::Length,
    dt: Time,
    rng: &mut impl Rng,
) -> Result<[Vec3; 2], ChemistryError> {
    parents_with_draws(
        id,
        positions,
        diffusivities,
        radius,
        dt,
        &mut RandomDraws(rng),
    )
}

fn parents_with_draws(
    id: u16,
    mut positions: [Vec3; 2],
    diffusivities: [g4_core::units::Diffusivity; 2],
    radius: g4_core::units::Length,
    dt: Time,
    draws: &mut impl PlacementDraws,
) -> Result<[Vec3; 2], ChemistryError> {
    let t = dt.ps();
    if !t.is_finite() || t < 0.0 {
        return Err(domain(id, "placement duration"));
    }
    let [d1, d2] = diffusivities.map(|d| d.nm2_per_ps());
    for value in [d1, d2] {
        if !value.is_finite() || value < 0.0 {
            return Err(domain(id, "diagnostic diffusivity"));
        }
    }
    if d1 + d2 <= 0.0 || !(d1 + d2).is_finite() {
        return Err(domain(id, "diagnostic total diffusion"));
    }
    let a = radius.nm();
    if !a.is_finite() || a <= 0.0 {
        return Err(domain(id, "diagnostic radius"));
    }
    for position in positions {
        finite_position(id, position)?;
    }
    let total = d1 + d2;
    if d1 == 0.0 {
        positions[1] = positions[0];
    } else if d2 == 0.0 {
        positions[0] = positions[1];
    } else {
        let delta = positions[0] - positions[1];
        let r = delta.x.hypot(delta.y).hypot(delta.z);
        if !r.is_finite() || r == 0.0 {
            return Err(domain(
                id,
                "coincident or unrepresentable mobile separation",
            ));
        }
        if t > 0.0 {
            let s12 = 2.0 * d1 * t;
            let s22 = 2.0 * d2 * t;
            // Width in nm: mm-native length-squared numerical width times 1e6.
            let sigma = (s12 + s12 * (s12 / s22)) * 1e-6;
            let alpha = a * r / (2.0 * total * t);
            if !sigma.is_finite() || sigma <= 0.0 || !alpha.is_finite() || alpha <= 0.0 {
                return Err(domain(id, "direct placement scale"));
            }
            let Vec3 { x, y, z } = draws.normal3();
            let s2 = positions[0] + positions[1] * (d1 / d2) + Vec3::new(x, y, z) * sigma;
            let draw_phi = std::f64::consts::TAU * draws.uniform(8);
            // CLHEP setPhi leaves an axial vector axial; subsequent setTheta
            // uses phi()==0. Preserve this source diagnostic anomaly.
            let phi = if delta.x == 0.0 && delta.y == 0.0 {
                0.0
            } else {
                draw_phi
            };
            let cosine = 1.0 + (-draws.uniform(16) * -(-2.0 * alpha).exp_m1()).ln_1p() / alpha;
            if !(-1.0..=1.0).contains(&cosine) {
                return Err(domain(id, "direct placement angle"));
            }
            let sine = cosine.acos().sin();
            let s1 = Vec3::new(sine * phi.cos(), sine * phi.sin(), cosine) * a;
            positions = [
                s1 * (d1 / total) + s2 * (d2 / total),
                (s2 - s1) * (d2 / total),
            ];
        }
    }
    for position in positions {
        finite_position(id, position)?;
    }
    Ok(positions)
}

fn validate_pair(
    pair: MoleculePair<'_>,
    p: &TwoBodyReactionParameters,
    ordered: bool,
) -> Result<[Vec3; 2], ChemistryError> {
    let id = p.reaction_id();
    let first = pair.first().instance();
    let second = pair.second().instance();
    let species = [first.species, second.species];
    if species != p.reactants() && (ordered || [species[1], species[0]] != p.reactants()) {
        return Err(ChemistryError::ReactionSpeciesMismatch { reaction_id: id });
    }
    if !first.time.ps().is_finite() || first.time.ps() < 0.0 || first.time != second.time {
        return Err(domain(id, "unsynchronized pair times"));
    }
    for position in [first.position, second.position] {
        finite_position(id, position)?;
    }
    Ok([first.position, second.position])
}
fn finite_position(id: u16, p: Vec3) -> Result<(), ChemistryError> {
    if [p.x, p.y, p.z].into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err(domain(id, "nonfinite position"))
    }
}
fn domain(reaction_id: u16, operation: &'static str) -> ChemistryError {
    ChemistryError::IrtDomain {
        reaction_id,
        operation,
    }
}
