//! Errors of the physicochemical stage.

use crate::molecule::MoleculeId;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ChemistryError {
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
