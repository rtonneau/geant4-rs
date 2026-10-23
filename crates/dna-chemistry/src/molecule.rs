//! Identity for individual chemical molecules, separate from their species records.

use crate::error::ChemistryError;
use dna_core::species::SpeciesInstance;

/// Stable identity assigned by the chemistry caller; equal species do not imply equal molecules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MoleculeId(pub u64);

/// A chemistry-stage molecule with its unchanged physicochemical-stage record.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Molecule {
    id: MoleculeId,
    instance: SpeciesInstance,
}

impl Molecule {
    pub const fn new(id: MoleculeId, instance: SpeciesInstance) -> Self {
        Self { id, instance }
    }

    pub const fn id(&self) -> MoleculeId {
        self.id
    }

    pub const fn instance(&self) -> &SpeciesInstance {
        &self.instance
    }
}

/// Two distinct molecules, including two instances of the same species.
#[derive(Debug, Clone, Copy)]
pub struct MoleculePair<'a> {
    first: &'a Molecule,
    second: &'a Molecule,
}

impl<'a> MoleculePair<'a> {
    /// Rejects a molecule paired with itself even when supplied through different records.
    pub fn new(first: &'a Molecule, second: &'a Molecule) -> Result<Self, ChemistryError> {
        if first.id == second.id {
            return Err(ChemistryError::DuplicateMoleculeId { id: first.id });
        }
        Ok(Self { first, second })
    }

    pub const fn first(&self) -> &'a Molecule {
        self.first
    }

    pub const fn second(&self) -> &'a Molecule {
        self.second
    }
}
