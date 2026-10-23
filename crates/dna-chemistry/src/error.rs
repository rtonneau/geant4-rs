//! Errors of the physicochemical stage.

use crate::molecule::MoleculeId;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ChemistryError {
    /// A runtime reaction input is outside its physical finite domain.
    #[error("reaction {reaction_id}: invalid {field} = {value}")]
    InvalidReactionParameter {
        reaction_id: u16,
        field: &'static str,
        value: f64,
    },
    /// An isolated pair cannot diffuse relative to itself.
    #[error("reaction {reaction_id}: zero total diffusivity")]
    ZeroTotalDiffusivity { reaction_id: u16 },
    /// Two-body IRT requires bimolecular TDC or PDC kinetics.
    #[error("reaction {reaction_id}: unsupported reaction order or kinetic intent")]
    UnsupportedReactionOrder { reaction_id: u16 },
    /// Bulk participants require the separate first-order scheduler.
    #[error("reaction {reaction_id}: bulk reactant is unsupported by two-body IRT")]
    BulkReactionParticipant { reaction_id: u16 },
    /// Supplied properties do not describe the source reaction's reactants.
    #[error("reaction {reaction_id}: species properties do not match reactants")]
    ReactionSpeciesMismatch { reaction_id: u16 },
    /// Finite positive activation rates require a sub-diffusion-limited observed rate.
    #[error(
        "reaction {reaction_id}: observed rate {observed} must be below diffusion rate {diffusion} nm^3/ps"
    )]
    InvalidPdcRate {
        reaction_id: u16,
        observed: f64,
        diffusion: f64,
    },
    /// Finite input arithmetic left the representable physical domain.
    #[error("reaction {reaction_id}: arithmetic domain failure in {operation}")]
    ReactionParameterDomain {
        reaction_id: u16,
        operation: &'static str,
    },
    /// A reaction pair names the same physical molecule twice.
    #[error("reaction pair contains molecule {id:?} twice")]
    DuplicateMoleculeId { id: MoleculeId },
    /// A free-diffusion input is negative or non-finite.
    #[error(
        "diffusion requires finite position and non-negative finite diffusivity and time; {field} = {value}"
    )]
    InvalidDiffusionInput { field: &'static str, value: f64 },
    /// The diffusion width cannot be represented as a finite number.
    #[error("non-finite diffusion scale {value}")]
    NonFiniteDiffusionScale { value: f64 },
    /// A sampled position cannot be represented as a finite number.
    #[error("non-finite diffusion output on {axis}: {value}")]
    NonFiniteDiffusionOutput { axis: &'static str, value: f64 },
    /// A numerical kernel received an argument outside its finite domain.
    #[error("{function} requires {domain}; received {value}")]
    SpecialFunctionDomain {
        function: &'static str,
        value: f64,
        domain: &'static str,
    },
    /// A data file cannot be parsed as TOML of the expected shape.
    #[error("cannot parse {file}: {message}")]
    Parse { file: &'static str, message: String },
    /// A data file parses but is inconsistent (probabilities, tags, balance, ...).
    #[error("invalid chemistry data in {file}: {message}")]
    Invalid { file: &'static str, message: String },
    /// A physicochemical event carries a level or shell index that the data do not describe.
    #[error("physicochemical event '{kind}' has unknown level/shell index {index}")]
    UnknownIndex { kind: &'static str, index: u8 },
}

impl ChemistryError {
    pub(crate) fn invalid(file: &'static str, message: impl Into<String>) -> Self {
        ChemistryError::Invalid {
            file,
            message: message.into(),
        }
    }
}
