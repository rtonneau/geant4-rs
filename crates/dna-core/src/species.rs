//! Chemical species of the physicochemical and chemical stages, and the plain-data record that
//! the physicochemical stage hands to the chemical stage (docs/03: exchanged `dna-core` types).
//!
//! The set is closed and is the list of molecular configurations created by
//! `G4ChemDissociationChannels_option1::ConstructMolecule`
//! (`physics_lists/constructors/electromagnetic/src/G4ChemDissociationChannels_option1.cc:54-190`),
//! which `G4EmDNAChemistry_option3` uses. The tags are the keys of `G4MoleculeTable`. The three
//! "bulk" configurations (`H2O(B)`, `H3Op(B)`, `OHm(B)`) and `NoneM` are bookkeeping species of
//! the reaction network (M7) and are not listed. Diffusion coefficients, charges and radii live in
//! `data/chemistry/species_option3.toml` (loaded by `dna-chemistry`), not here.

use crate::units::Time;
use crate::vec3::Vec3;

/// A chemical species (molecular configuration) of the Geant4-DNA water radiolysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Species {
    /// Hydronium ion H3O+ (`H3Op`).
    H3Op,
    /// Hydroxyl radical (`°OH`).
    Oh,
    /// Hydroxide ion OH- (`OHm`).
    Ohm,
    /// Hydrated (solvated) electron (`e_aq`).
    EAq,
    /// Hydrogen atom (`H`).
    H,
    /// Molecular hydrogen (`H2`).
    H2,
    /// Hydrogen peroxide (`H2O2`).
    H2o2,
    /// Hydroperoxyl radical (`HO2°`).
    Ho2,
    /// Hydroperoxide anion (`HO2m`).
    Ho2m,
    /// Atomic oxygen O(3P) (`Oxy`).
    O,
    /// Atomic oxygen anion (`Om`).
    Om,
    /// Molecular oxygen (`O2`).
    O2,
    /// Superoxide anion (`O2m`).
    O2m,
    /// Ozone (`O3`).
    O3,
    /// Ozonide anion (`O3m`).
    O3m,
}

impl Species {
    /// All species, in the order of `ConstructMolecule`.
    pub const ALL: [Species; 15] = [
        Species::H3Op,
        Species::Oh,
        Species::Ohm,
        Species::EAq,
        Species::H,
        Species::H2,
        Species::H2o2,
        Species::Ho2,
        Species::Ho2m,
        Species::O,
        Species::Om,
        Species::O2,
        Species::O2m,
        Species::O3,
        Species::O3m,
    ];

    /// Stable ASCII tag used in the data files and the CSV output: the `G4MoleculeTable` key, with
    /// the non-ASCII degree sign of `°OH` and `HO2°` removed.
    pub const fn tag(self) -> &'static str {
        match self {
            Species::H3Op => "H3Op",
            Species::Oh => "OH",
            Species::Ohm => "OHm",
            Species::EAq => "e_aq",
            Species::H => "H",
            Species::H2 => "H2",
            Species::H2o2 => "H2O2",
            Species::Ho2 => "HO2",
            Species::Ho2m => "HO2m",
            Species::O => "Oxy",
            Species::Om => "Om",
            Species::O2 => "O2",
            Species::O2m => "O2m",
            Species::O3 => "O3",
            Species::O3m => "O3m",
        }
    }

    /// Species from its tag.
    pub fn from_tag(tag: &str) -> Option<Species> {
        Species::ALL.into_iter().find(|s| s.tag() == tag)
    }

    /// Index in [`Species::ALL`] (dense, for arrays of counts).
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// One species created by the physicochemical stage: the input record of the chemical stage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeciesInstance {
    pub species: Species,
    /// Position, nm.
    pub position: Vec3,
    /// Creation time (global time at which the species is placed), ps.
    pub time: Time,
}

/// Header of the species CSV (`SpeciesInstance::csv_row` gives the rows).
pub const SPECIES_CSV_HEADER: &str = "species,x_nm,y_nm,z_nm,time_ps";

impl SpeciesInstance {
    /// One CSV row (no newline); floats with the shortest round-trip representation.
    pub fn csv_row(&self) -> String {
        format!(
            "{},{:?},{:?},{:?},{:?}",
            self.species.tag(),
            self.position.x,
            self.position.y,
            self.position.z,
            self.time.ps()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_are_unique_and_round_trip() {
        for (i, s) in Species::ALL.iter().enumerate() {
            assert_eq!(Species::from_tag(s.tag()), Some(*s));
            assert_eq!(s.index(), i);
            for t in &Species::ALL[i + 1..] {
                assert_ne!(s.tag(), t.tag());
            }
        }
        assert_eq!(Species::from_tag("nonsense"), None);
    }

    #[test]
    fn csv_row_matches_the_header() {
        let r = SpeciesInstance {
            species: Species::EAq,
            position: Vec3::new(1.0, -2.5, 0.25),
            time: Time::from_ps(1.0),
        };
        assert_eq!(r.csv_row(), "e_aq,1.0,-2.5,0.25,1.0");
        assert_eq!(
            r.csv_row().split(',').count(),
            SPECIES_CSV_HEADER.split(',').count()
        );
    }
}
