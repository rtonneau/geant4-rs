//! The Geant4-DNA physicochemical stage for liquid water (milestone M6, part A): from the events of the
//! physical stage to the chemical species that the chemical stage (M7) starts from.
//!
//! * [`species`]: species table (diffusion coefficient, charge, radius) from `data/chemistry/species_option3.toml`.
//! * [`channels`]: decay channels of the excited / ionised / attached water molecule
//!   (`G4ChemDissociationChannels_option1`), validated at load.
//! * [`displacer`]: `G4DNAWaterDissociationDisplacer`, the positions of the decay products.
//! * [`params`]: birth time, auto-ionisation electron, electron-hole recombination (Onsager radius).
//! * [`engine`]: [`PhysChemEngine::convert_history`], the stage for one history.
//!
//! No diffusion and no reactions here (M7). The crate depends on `dna-core` only: the event and species
//! records are `dna-core` types (docs/03).

pub mod channels;
pub mod displacer;
pub mod engine;
pub mod error;
pub mod params;
pub mod species;

pub use engine::{EventDecay, HistoryChemistry, PhysChemEngine, PhysChemStats};
pub use error::ChemistryError;

/// SHA-256 of the three embedded option3 chemistry input files, in the documented order
/// species, dissociation channels, physicochemical parameters.
pub fn option3_data_sha256() -> String {
    let mut hash = dna_core::sha256::Sha256::new();
    hash.update(species::DEFAULT_SPECIES_TOML.as_bytes());
    hash.update(channels::DEFAULT_DISSOCIATION_TOML.as_bytes());
    hash.update(params::DEFAULT_PHYSCHEM_TOML.as_bytes());
    dna_core::sha256::hex(&hash.finalize())
}
