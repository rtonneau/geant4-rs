//! Geant4-DNA chemistry primitives for liquid water. M6 converts physical
//! events to species at the 1 ps boundary; the implemented M7 slice adds free
//! unbounded diffusion and an isolated two-body option3 IRT sampler. These
//! kernels do not implement complete chemical evolution.
//!
//! * [`species`]: species table (diffusion coefficient, charge, radius) from `data/chemistry/species_option3.toml`.
//! * [`channels`]: decay channels of the excited / ionised / attached water molecule
//!   (`G4ChemDissociationChannels_option1`), validated at load.
//! * [`displacer`]: `G4DNAWaterDissociationDisplacer`, the positions of the decay products.
//! * [`params`]: birth time, auto-ionisation electron, electron-hole recombination (Onsager radius).
//! * [`engine`]: [`PhysChemEngine::convert_history`], the stage for one history.
//!
//! [`diffusion`] advances positions in infinite 3-D space. [`molecule`]
//! provides stable identity and rejects self-pairs. [`reaction_parameters`]
//! derives validated TDC/PDC parameters, and [`irt`] samples one pair's elapsed
//! reaction-time outcome. Callers own RNG streams; integrated histories use
//! `Stage::Chemistry`. [`Species`] and [`SpeciesInstance`] are owned here;
//! physical handoff records remain in `g4_core::physchem`.

pub mod channels;
pub mod diffusion;
pub mod displacer;
pub mod engine;
pub mod error;
pub mod irt;
pub mod molecule;
pub mod params;
pub mod reaction_parameters;
pub mod reactions;
/// Validation-only step-by-step oracle; not a production chemistry scheduler.
pub mod sbs;
pub(crate) mod special;
pub mod species;
mod species_types;

pub use engine::{EventDecay, HistoryChemistry, PhysChemEngine, PhysChemStats};
pub use error::ChemistryError;
pub use species_types::{SPECIES_CSV_HEADER, Species, SpeciesInstance};

/// SHA-256 of the four embedded option3 chemistry input files, in the documented order:
/// species, dissociation channels, physicochemical parameters, reactions.
pub fn option3_data_sha256() -> String {
    let mut hash = g4_core::sha256::Sha256::new();
    // `include_str!` retains checkout line endings. Hash the repository's canonical
    // LF representation so provenance is independent of Git's Windows conversion.
    for input in [
        species::DEFAULT_SPECIES_TOML,
        channels::DEFAULT_DISSOCIATION_TOML,
        params::DEFAULT_PHYSCHEM_TOML,
        reactions::DEFAULT_REACTIONS_TOML,
    ] {
        hash.update(input.replace("\r\n", "\n").as_bytes());
    }
    g4_core::sha256::hex(&hash.finalize())
}

#[cfg(test)]
mod data_hash_tests {
    #[test]
    fn option3_fingerprint_includes_reactions_after_the_existing_three_files() {
        assert_eq!(
            super::option3_data_sha256(),
            "46a49f0cde41062cd4c55fb93e7817f481c68918e3ad6c810686db9dda4cbbde"
        );
    }
}
