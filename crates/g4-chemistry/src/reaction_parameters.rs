//! Validated runtime parameters for isolated option3 two-body chemistry.
//!
//! Normative source: Geant4 11.4.1,
//! `source/processes/electromagnetic/dna/utils/src/G4DNAMolecularReactionTable.cc`,
//! `ComputeEffectiveRadius` (127-151) and `SetReactionType` (296-335).
//! The option3 constructor is
//! `source/physics_lists/constructors/electromagnetic/src/G4EmDNAChemistry_option3.cc`.
//! Relative diffusion and the electrostatic distance mapping follow
//! `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc:286-333`.
//!
//! Rates are converted from M^-1 s^-1 to nm^3/ps per molecule; source literals
//! remain untouched. TDC A+A radii use D_A, although motion uses 2 D_A.
//! PDC uses the physical radius sum, halving its diffusion rate for A+A.
//! Charged PDC construction is supported; this does not imply a charged PDC sampler.
//! Invalid arithmetic and k_obs >= k_diff are errors instead of C++ sentinel data.

use crate::Species;
use crate::error::ChemistryError;
use crate::reactions::{KineticIntent, ObservedRate, Reaction, ReactionParticipant};
use crate::species::{SpeciesProps, SpeciesTable};
use g4_core::units::{Diffusivity, Length, RateConstant};
use std::f64::consts::PI;

// G4DNAMolecularReactionTable.cc:148 fixes T=293.15 K and epsilon=80.1.
// CLHEP/Units/PhysicalConstants.h:56,90-91,116 and SystemOfUnits.h:164:
// e^2/(4*pi*epsilon0*k_B) = e_SI*c^2*1e-7/k_B[eV/K], in m K.
// This intentionally differs from M6's density-dependent dielectric constant.
const UNIT_ONSAGER_NM: f64 =
    1.602_176_634e-19 * 299_792_458.0 * 299_792_458.0 * 1e-7 / 8.617_333e-5 / (293.15 * 80.1) * 1e9;
const ENCOUNTER_STEP_NM: f64 = 0.29;

/// Derived PDC rates, in per-molecule volume/time units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PdcParameters {
    diffusion_rate: RateConstant,
    activation_rate: RateConstant,
}

impl PdcParameters {
    /// Diffusion-limited rate, including Geant4's A+A factor of one half.
    pub fn diffusion_rate(self) -> RateConstant {
        self.diffusion_rate
    }
    /// Activation rate, distinct from the observed source-table rate.
    pub fn activation_rate(self) -> RateConstant {
        self.activation_rate
    }
}

/// Immutable, validated parameters derived without modifying the source reaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwoBodyReactionParameters {
    reaction_id: u16,
    reactants: [Species; 2],
    relative_diffusivity: Diffusivity,
    observed_rate: RateConstant,
    reaction_radius: Length,
    effective_radius: Length,
    onsager_radius: Length,
    contact_probability: f64,
    pdc: Option<PdcParameters>,
}

impl TwoBodyReactionParameters {
    /// Derive parameters for an entry of a reaction table using a species table.
    pub fn derive(reaction: &Reaction, species: &SpeciesTable) -> Result<Self, ChemistryError> {
        let reactants = mobile_reactants(reaction)?;
        Self::from_species(reaction, reactants.map(|s| *species.get(s)))
    }

    /// Derive parameters from explicit species properties in source reactant order.
    /// Non-negative diffusivities are allowed individually; their sum must be positive.
    /// For A+A the two property records must agree.
    pub fn from_species(
        reaction: &Reaction,
        props: [SpeciesProps; 2],
    ) -> Result<Self, ChemistryError> {
        let id = reaction.id;
        let reactants = mobile_reactants(reaction)?;
        let is_pdc = match (reaction.kinetic_intent, reaction.set_reaction_type) {
            (KineticIntent::TotallyDiffusionControlled, false) => false,
            (KineticIntent::PartiallyDiffusionControlled, true) => true,
            _ => return Err(ChemistryError::UnsupportedReactionOrder { reaction_id: id }),
        };
        let ObservedRate::Bimolecular { per_molar_s } = reaction.rate else {
            return Err(ChemistryError::UnsupportedReactionOrder { reaction_id: id });
        };
        let identical = reactants[0] == reactants[1];
        if props.map(|p| p.species) != reactants || (identical && props[0] != props[1]) {
            return Err(ChemistryError::ReactionSpeciesMismatch { reaction_id: id });
        }
        input(id, "observed rate", per_molar_s, false)?;
        for prop in props {
            input(id, "diffusivity", prop.diffusivity.nm2_per_ps(), true)?;
            input(id, "species radius", prop.radius.nm(), false)?;
        }
        let relative = props[0].diffusivity.nm2_per_ps() + props[1].diffusivity.nm2_per_ps();
        if relative == 0.0 {
            return Err(ChemistryError::ZeroTotalDiffusivity { reaction_id: id });
        }
        positive(id, "relative diffusivity", relative)?;
        let observed_rate = RateConstant::from_per_molar_per_s(per_molar_s);
        let observed = positive(id, "observed-rate conversion", observed_rate.nm3_per_ps())?;
        let onsager = f64::from(props[0].charge) * f64::from(props[1].charge) * UNIT_ONSAGER_NM;
        let rate_diffusivity = if identical {
            props[0].diffusivity.nm2_per_ps()
        } else {
            relative
        };
        let rate_factor = positive(id, "4 pi D", 4.0 * PI * rate_diffusivity)?;
        let (radius, effective, pdc, probability) = if is_pdc {
            let radius = positive(
                id,
                "physical contact radius",
                props[0].radius.nm() + props[1].radius.nm(),
            )?;
            let effective = transform_distance(id, onsager, radius)?;
            let diffusion = positive(id, "diffusion rate", rate_factor * effective)?;
            if observed >= diffusion {
                return Err(ChemistryError::InvalidPdcRate {
                    reaction_id: id,
                    observed,
                    diffusion,
                });
            }
            // Algebraically k_diff*k_obs/(k_diff-k_obs), avoiding the product overflow.
            let activation = positive(
                id,
                "activation rate",
                observed * (diffusion / (diffusion - observed)),
            )?;
            let denominator = positive(
                id,
                "contact probability denominator",
                ENCOUNTER_STEP_NM + (diffusion / activation) * (effective + ENCOUNTER_STEP_NM),
            )?;
            let probability = positive(id, "contact probability", ENCOUNTER_STEP_NM / denominator)?;
            if probability > 1.0 {
                return Err(ChemistryError::ReactionParameterDomain {
                    reaction_id: id,
                    operation: "contact probability above one",
                });
            }
            (
                radius,
                effective,
                Some(PdcParameters {
                    diffusion_rate: RateConstant::from_nm3_per_ps(diffusion),
                    activation_rate: RateConstant::from_nm3_per_ps(activation),
                }),
                probability,
            )
        } else {
            let radius = positive(id, "TDC effective radius", observed / rate_factor)?;
            (radius, radius, None, 1.0)
        };
        Ok(Self {
            reaction_id: id,
            reactants,
            relative_diffusivity: Diffusivity::from_nm2_per_ps(relative),
            observed_rate,
            reaction_radius: Length::from_nm(radius),
            effective_radius: Length::from_nm(effective),
            onsager_radius: Length::from_nm(onsager),
            contact_probability: probability,
            pdc,
        })
    }

    /// Stable source-table reaction ID.
    pub fn reaction_id(self) -> u16 {
        self.reaction_id
    }
    /// Species in source reactant order.
    pub fn reactants(self) -> [Species; 2] {
        self.reactants
    }
    /// Relative-motion diffusivity, D_A + D_B even for A+A.
    pub fn relative_diffusivity(self) -> Diffusivity {
        self.relative_diffusivity
    }
    /// Observed source rate converted to nm^3/ps per molecule.
    pub fn observed_rate(self) -> RateConstant {
        self.observed_rate
    }
    /// TDC effective radius or PDC physical contact radius.
    pub fn reaction_radius(self) -> Length {
        self.reaction_radius
    }
    /// Effective reaction radius used by option3 IRT.
    pub fn effective_radius(self) -> Length {
        self.effective_radius
    }
    /// Signed electrostatic length: positive for repulsion, negative for attraction.
    pub fn onsager_radius(self) -> Length {
        self.onsager_radius
    }
    /// Stored encounter probability; PDC uses the source jump length Rs = 0.29 nm.
    pub fn contact_probability(self) -> f64 {
        self.contact_probability
    }
    /// PDC rates, absent for TDC (whose activation rate is not finite).
    pub fn pdc(self) -> Option<PdcParameters> {
        self.pdc
    }
    /// Geant4's signed-Onsager distance transform for a strictly positive distance.
    /// Uses `expm1` to preserve the neutral limit at large distances. A caller
    /// must handle contact (including zero separation) before this transform,
    /// as in `G4DNAIRT::GetIndependentReactionTime`.
    pub fn effective_distance(self, distance: Length) -> Result<Length, ChemistryError> {
        input(self.reaction_id, "distance", distance.nm(), false)?;
        transform_distance(self.reaction_id, self.onsager_radius.nm(), distance.nm())
            .map(Length::from_nm)
    }
}

fn mobile_reactants(reaction: &Reaction) -> Result<[Species; 2], ChemistryError> {
    match reaction.reactants {
        [
            ReactionParticipant::Mobile(a),
            ReactionParticipant::Mobile(b),
        ] => Ok([a, b]),
        _ => Err(ChemistryError::BulkReactionParticipant {
            reaction_id: reaction.id,
        }),
    }
}

fn input(id: u16, field: &'static str, value: f64, allow_zero: bool) -> Result<(), ChemistryError> {
    if !value.is_finite() || value < 0.0 || (!allow_zero && value == 0.0) {
        Err(ChemistryError::InvalidReactionParameter {
            reaction_id: id,
            field,
            value,
        })
    } else {
        Ok(())
    }
}

fn positive(id: u16, operation: &'static str, value: f64) -> Result<f64, ChemistryError> {
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err(ChemistryError::ReactionParameterDomain {
            reaction_id: id,
            operation,
        })
    }
}

fn transform_distance(id: u16, onsager: f64, distance: f64) -> Result<f64, ChemistryError> {
    if onsager == 0.0 {
        return Ok(distance);
    }
    let ratio = onsager / distance;
    if !ratio.is_finite() {
        return Err(ChemistryError::ReactionParameterDomain {
            reaction_id: id,
            operation: "Onsager/distance",
        });
    }
    if ratio == 0.0 {
        return Ok(distance);
    }
    // rc/(exp(rc/r)-1) is the source's -rc/(1-exp(rc/r)).
    positive(id, "effective distance", onsager / ratio.exp_m1())
}
