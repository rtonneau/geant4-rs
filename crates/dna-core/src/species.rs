//! Chemical species of the physicochemical and chemical stages, and the plain-data record that
//! the physicochemical stage hands to the chemical stage (docs/03: exchanged `dna-core` types).
//!
//! The set is closed and is the list of molecular configurations created by
//! `G4ChemDissociationChannels_option1::ConstructMolecule`
//! (`physics_lists/constructors/electromagnetic/src/G4ChemDissociationChannels_option1.cc:54-190`),
//! which `G4EmDNAChemistry_option3` uses. The chemistry names are the keys of `G4MoleculeTable`. The three
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

    /// Exact `G4MoleculeTable` configuration key used by Geant4 v11.4.1.
    pub const fn chemistry_name(self) -> &'static str {
        match self {
            Species::H3Op => "H3Op",
            Species::Oh => "°OH",
            Species::Ohm => "OHm",
            Species::EAq => "e_aq",
            Species::H => "H",
            Species::H2 => "H2",
            Species::H2o2 => "H2O2",
            Species::Ho2 => "HO2°",
            Species::Ho2m => "HO2m",
            Species::O => "Oxy",
            Species::Om => "Om",
            Species::O2 => "O2",
            Species::O2m => "O2m",
            Species::O3 => "O3",
            Species::O3m => "O3m",
        }
    }

    /// Parse an exact Geant4 v11.4.1 configuration key.
    pub fn from_chemistry_name(name: &str) -> Option<Species> {
        Species::ALL.into_iter().find(|s| s.chemistry_name() == name)
    }

    /// Parse the former ASCII data-file tags, including `OH` and `HO2`.
    /// Canonical names are also accepted for callers migrating existing input.
    pub fn from_legacy_ascii_tag(tag: &str) -> Option<Species> {
        match tag {
            "OH" => Some(Species::Oh),
            "HO2" => Some(Species::Ho2),
            _ => Species::from_chemistry_name(tag),
        }
    }

    /// Temporary ASCII tag for existing data files and CSV output.
    pub const fn tag(self) -> &'static str {
        match self {
            Species::Oh => "OH",
            Species::Ho2 => "HO2",
            _ => self.chemistry_name(),
        }
    }

    /// Temporary forwarding API for existing ASCII data files.
    pub fn from_tag(tag: &str) -> Option<Species> {
        Species::from_legacy_ascii_tag(tag)
    }

    /// Species from the name of a Geant4 molecule, `G4Molecule::GetName()` = `<definition name>^<charge>`
    /// (e.g. `H3O^1`, `°OH^0`, `e_aq^-1`, `HO_2°^0`, `HO_2^-1`, `O^-1`). The definition names are those
    /// of `G4H3O`, `G4OH`, `G4Electron_aq`, `G4Hydrogen`, `G4H2`, `G4H2O2`, `G4HO2`, `G4Oxygen`,
    /// `G4O2`, `G4O3` and of the anion definitions created in `G4ChemDissociationChannels_option1.cc`
    /// (`OH`, `HO_2`, `O`; `O_2` and `O_3` with charge -1). Returns `None` for anything else
    /// (including `H2O^n`, the water states, which are not species).
    pub fn from_geant4_name(name: &str) -> Option<Species> {
        let (definition, charge) = name.rsplit_once('^')?;
        let charge: i32 = charge.parse().ok()?;
        match (definition, charge) {
            ("H3O", 1) => Some(Species::H3Op),
            ("°OH", 0) => Some(Species::Oh),
            ("OH", -1) => Some(Species::Ohm),
            ("e_aq", -1) => Some(Species::EAq),
            ("H", 0) => Some(Species::H),
            ("H_2", 0) => Some(Species::H2),
            ("H2O2", 0) => Some(Species::H2o2),
            ("HO_2°", 0) => Some(Species::Ho2),
            ("HO_2", -1) => Some(Species::Ho2m),
            ("°O", 0) => Some(Species::O),
            ("O", -1) => Some(Species::Om),
            ("O_2", 0) => Some(Species::O2),
            ("O_2", -1) => Some(Species::O2m),
            ("O_3", 0) => Some(Species::O3),
            ("O_3", -1) => Some(Species::O3m),
            _ => None,
        }
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
    fn geant4_molecule_names_map_to_species() {
        let cases = [
            ("H3O^1", Species::H3Op),
            ("°OH^0", Species::Oh),
            ("OH^-1", Species::Ohm),
            ("e_aq^-1", Species::EAq),
            ("H^0", Species::H),
            ("H_2^0", Species::H2),
            ("H2O2^0", Species::H2o2),
            ("HO_2°^0", Species::Ho2),
            ("HO_2^-1", Species::Ho2m),
            ("°O^0", Species::O),
            ("O^-1", Species::Om),
            ("O_2^0", Species::O2),
            ("O_2^-1", Species::O2m),
            ("O_3^0", Species::O3),
            ("O_3^-1", Species::O3m),
        ];
        for (name, species) in cases {
            assert_eq!(Species::from_geant4_name(name), Some(species), "{name}");
        }
        // all 15 species are reachable, and water states are not species
        let mapped: std::collections::HashSet<_> = cases.iter().map(|(_, s)| *s).collect();
        assert_eq!(mapped.len(), Species::ALL.len());
        assert_eq!(Species::from_geant4_name("H2O^1"), None);
        assert_eq!(Species::from_geant4_name("H2O^0"), None);
        assert_eq!(Species::from_geant4_name("H3Op"), None);
    }

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
    fn exact_chemistry_names_round_trip_in_geant4_order() {
        let names = [
            "H3Op", "°OH", "OHm", "e_aq", "H", "H2", "H2O2", "HO2°", "HO2m", "Oxy", "Om",
            "O2", "O2m", "O3", "O3m",
        ];
        for (species, name) in Species::ALL.into_iter().zip(names) {
            assert_eq!(species.chemistry_name().as_bytes(), name.as_bytes());
            assert_eq!(Species::from_chemistry_name(name), Some(species));
        }
    }

    #[test]
    fn canonical_parser_is_strict_and_legacy_parser_accepts_ascii_tags() {
        for species in Species::ALL {
            assert_eq!(Species::from_legacy_ascii_tag(species.tag()), Some(species));
        }
        for alias in ["OH", "HO2"] {
            assert_eq!(Species::from_chemistry_name(alias), None);
        }
        for invalid in ["", "oh", "Ho2", "HO_2°", "°OH^0", "H3Op(B)"] {
            assert_eq!(Species::from_chemistry_name(invalid), None);
            assert_eq!(Species::from_legacy_ascii_tag(invalid), None);
        }
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
