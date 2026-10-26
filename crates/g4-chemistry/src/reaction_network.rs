//! Immutable execution descriptors for all 72 option3 source reactions.
//!
//! Source: Geant4 11.4.1 `source/physics_lists/constructors/electromagnetic/src/
//! G4EmDNAChemistry_option3.cc`, `ConstructReactionTable` (146-664).
//! Pair parameters follow [`crate::reaction_parameters`]. Unary rates retain s^-1,
//! following `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc:250-281`.
//! Reservoirs and the NoneM sentinel remain distinct from mobile species. This
//! module neither samples reactions nor implements a scheduler.

use crate::reaction_parameters::TwoBodyReactionParameters;
use crate::reactions::{
    BulkParticipant, KineticIntent, ObservedRate, Reaction, ReactionParticipant, ReactionTable,
};
use crate::species::SpeciesTable;
use crate::{ChemistryError, Species};
use g4_core::units::Diffusivity;

#[derive(Debug, Clone, Copy, PartialEq)]
enum UnaryRate {
    Disabled,
    Enabled { per_s: f64 },
}

/// Validated first-/pseudo-first-order channel, with its source bulk partner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnaryReactionParameters {
    reaction_id: u16,
    mobile: Species,
    bulk: BulkParticipant,
    rate: UnaryRate,
}

impl UnaryReactionParameters {
    /// Stable source-table ID.
    pub fn reaction_id(self) -> u16 {
        self.reaction_id
    }

    /// The single mobile reactant consumed by this channel.
    pub fn mobile(self) -> Species {
        self.mobile
    }

    /// Bulk reservoir (or NoneM sentinel), never a mobile molecule.
    pub fn bulk(self) -> BulkParticipant {
        self.bulk
    }

    /// Observed source rate in s^-1; zero denotes an explicitly disabled channel.
    pub fn rate_per_s(self) -> f64 {
        match self.rate {
            UnaryRate::Disabled => 0.0,
            UnaryRate::Enabled { per_s } => per_s,
        }
    }

    /// Whether a present custom channel is disabled. Such a channel must not draw RNG.
    pub fn is_disabled(self) -> bool {
        matches!(self.rate, UnaryRate::Disabled)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ReactionExecution {
    Pair(TwoBodyReactionParameters),
    Unary(UnaryReactionParameters),
}

/// One immutable source record and its validated execution parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledReaction {
    source: Reaction,
    execution: ReactionExecution,
    reactant_diffusivities: [Diffusivity; 2],
}

impl CompiledReaction {
    /// Audited source fields, including ordered products, implicit water and anomaly marker.
    pub fn source(&self) -> &Reaction {
        &self.source
    }

    /// Pair parameters, absent only for unary channels.
    pub fn pair(&self) -> Option<&TwoBodyReactionParameters> {
        match &self.execution {
            ReactionExecution::Pair(parameters) => Some(parameters),
            ReactionExecution::Unary(_) => None,
        }
    }

    /// Unary parameters, including disabled channels; absent only for pairs.
    pub fn unary(&self) -> Option<&UnaryReactionParameters> {
        match &self.execution {
            ReactionExecution::Pair(_) => None,
            ReactionExecution::Unary(parameters) => Some(parameters),
        }
    }

    /// Diffusivities in source reactant order, retained for conditional placement.
    /// A unary channel's second entry is its zero-diffusion fake bulk partner.
    pub fn reactant_diffusivities(&self) -> [Diffusivity; 2] {
        self.reactant_diffusivities
    }
}

/// Complete source-ordered network. Construction fails atomically if any entry is invalid.
#[derive(Debug, Clone)]
pub struct ReactionNetwork {
    reactions: Vec<CompiledReaction>,
}

impl ReactionNetwork {
    /// Compile the full Geant4 option3 table with validated species properties.
    /// Custom rates are accepted; identities, product order and source annotations
    /// must match the audited table. There is no fallback for unsupported channels.
    pub fn compile(table: &ReactionTable, species: &SpeciesTable) -> Result<Self, ChemistryError> {
        let reference = ReactionTable::option3()?;
        let mut reactions = Vec::with_capacity(72);
        for (reaction, expected) in table.iter().zip(reference.iter()) {
            validate_signature(reaction, expected)?;
            // Validate products as well as parents, even if a channel is disabled.
            for participant in reaction.reactants.iter().chain(&reaction.products) {
                if let ReactionParticipant::Mobile(mobile) = participant {
                    let prop = species.get(*mobile);
                    crate::reaction_parameters::input(
                        reaction.id,
                        "diffusivity",
                        prop.diffusivity.nm2_per_ps(),
                        true,
                    )?;
                    crate::reaction_parameters::input(
                        reaction.id,
                        "species radius",
                        prop.radius.nm(),
                        false,
                    )?;
                }
            }
            let execution = match (reaction.rate, reaction.kinetic_intent, reaction.reactants) {
                (
                    ObservedRate::Bimolecular { .. },
                    KineticIntent::TotallyDiffusionControlled
                    | KineticIntent::PartiallyDiffusionControlled,
                    [
                        ReactionParticipant::Mobile(_),
                        ReactionParticipant::Mobile(_),
                    ],
                ) => ReactionExecution::Pair(TwoBodyReactionParameters::derive(reaction, species)?),
                (
                    ObservedRate::FirstOrder { per_s },
                    KineticIntent::FirstOrder | KineticIntent::PseudoFirstOrder,
                    [
                        ReactionParticipant::Mobile(mobile),
                        ReactionParticipant::Bulk(bulk),
                    ],
                ) if !reaction.set_reaction_type => {
                    crate::reaction_parameters::input(reaction.id, "unary rate", per_s, true)?;
                    ReactionExecution::Unary(UnaryReactionParameters {
                        reaction_id: reaction.id,
                        mobile,
                        bulk,
                        rate: if per_s == 0.0 {
                            UnaryRate::Disabled
                        } else {
                            UnaryRate::Enabled { per_s }
                        },
                    })
                }
                _ => {
                    return Err(ChemistryError::UnsupportedReactionOrder {
                        reaction_id: reaction.id,
                    });
                }
            };
            reactions.push(CompiledReaction {
                source: reaction.clone(),
                execution,
                reactant_diffusivities: reaction.reactants.map(|participant| match participant {
                    ReactionParticipant::Mobile(mobile) => species.get(mobile).diffusivity,
                    ReactionParticipant::Bulk(_) => Diffusivity::from_nm2_per_ps(0.0),
                }),
            });
        }
        Ok(Self { reactions })
    }

    /// All descriptors in stable source order, including disabled channels.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &CompiledReaction> {
        self.reactions.iter()
    }

    /// Lookup by source ID; an absent ID differs from a disabled unary descriptor.
    pub fn get(&self, id: u16) -> Option<&CompiledReaction> {
        self.reactions.get(usize::from(id.checked_sub(1)?))
    }
}

fn validate_signature(reaction: &Reaction, expected: &Reaction) -> Result<(), ChemistryError> {
    if reaction.id != expected.id
        || reaction.source_line != expected.source_line
        || reaction.source_class != expected.source_class
        || reaction.reactants != expected.reactants
        || reaction.products != expected.products
        || reaction.kinetic_intent != expected.kinetic_intent
        || reaction.set_reaction_type != expected.set_reaction_type
        || reaction.implicit_water != expected.implicit_water
        || reaction.balance_exception != expected.balance_exception
        || std::mem::discriminant(&reaction.rate) != std::mem::discriminant(&expected.rate)
    {
        return Err(ChemistryError::ReactionSourceMismatch {
            reaction_id: reaction.id,
        });
    }
    Ok(())
}
