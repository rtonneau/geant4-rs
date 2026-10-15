//! Errors of the physicochemical stage.

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ChemistryError {
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
