//! Chemical species of the physicochemical and chemical stages, and the plain-data record that
//! the physicochemical stage hands to the chemical stage, owned by `dna-chemistry`.
//!
//! The set is closed and is the list of molecular configurations created by
//! `G4ChemDissociationChannels_option1::ConstructMolecule`
//! (`physics_lists/constructors/electromagnetic/src/G4ChemDissociationChannels_option1.cc:54-190`),
//! which `G4EmDNAChemistry_option3` uses. The chemistry names are the keys of `G4MoleculeTable`. The three
//! "bulk" configurations (`H2O(B)`, `H3Op(B)`, `OHm(B)`) and `NoneM` are bookkeeping species of
//! the reaction network (M7) and are not listed. Diffusion coefficients, charges and radii live in
//! `data/chemistry/species_option3.toml` (loaded by `dna-chemistry`), not here.

use dna_core::units::Time;
use dna_core::vec3::Vec3;

/// A chemical species (molecular configuration) of the Geant4-DNA water radiolysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Species {
    /// Hydronium ion H3O+ (`H3Op`).
    H3Op,
    /// Hydroxyl radical (`°OH`).
    OH,
    /// Hydroxide ion OH- (`OHm`).
    OHm,
    /// Hydrated (solvated) electron (`e_aq`).
    // Preserve Geant4's configuration spelling at the Rust identifier boundary.
    #[allow(non_camel_case_types)]
    e_aq,
    /// Hydrogen atom (`H`).
    H,
    /// Molecular hydrogen (`H2`).
    H2,
    /// Hydrogen peroxide (`H2O2`).
    H2O2,
    /// Hydroperoxyl radical (`HO2°`).
    HO2,
    /// Hydroperoxide anion (`HO2m`).
    HO2m,
    /// Atomic oxygen O(3P) (`Oxy`).
    Oxy,
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
        Species::OH,
        Species::OHm,
        Species::e_aq,
        Species::H,
        Species::H2,
        Species::H2O2,
        Species::HO2,
        Species::HO2m,
        Species::Oxy,
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
            Species::OH => "°OH",
            Species::OHm => "OHm",
            Species::e_aq => "e_aq",
            Species::H => "H",
            Species::H2 => "H2",
            Species::H2O2 => "H2O2",
            Species::HO2 => "HO2°",
            Species::HO2m => "HO2m",
            Species::Oxy => "Oxy",
            Species::Om => "Om",
            Species::O2 => "O2",
            Species::O2m => "O2m",
            Species::O3 => "O3",
            Species::O3m => "O3m",
        }
    }

    /// Parse an exact Geant4 v11.4.1 configuration key.
    pub fn from_chemistry_name(name: &str) -> Option<Species> {
        Species::ALL
            .into_iter()
            .find(|s| s.chemistry_name() == name)
    }

    /// Parse the former ASCII data-file tags, including `OH` and `HO2`.
    /// Canonical names are also accepted for callers migrating existing input.
    pub fn from_legacy_ascii_tag(tag: &str) -> Option<Species> {
        match tag {
            "OH" => Some(Species::OH),
            "HO2" => Some(Species::HO2),
            _ => Species::from_chemistry_name(tag),
        }
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
            ("°OH", 0) => Some(Species::OH),
            ("OH", -1) => Some(Species::OHm),
            ("e_aq", -1) => Some(Species::e_aq),
            ("H", 0) => Some(Species::H),
            ("H_2", 0) => Some(Species::H2),
            ("H2O2", 0) => Some(Species::H2O2),
            ("HO_2°", 0) => Some(Species::HO2),
            ("HO_2", -1) => Some(Species::HO2m),
            ("°O", 0) => Some(Species::Oxy),
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
            self.species.chemistry_name(),
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
            ("°OH^0", Species::OH),
            ("OH^-1", Species::OHm),
            ("e_aq^-1", Species::e_aq),
            ("H^0", Species::H),
            ("H_2^0", Species::H2),
            ("H2O2^0", Species::H2O2),
            ("HO_2°^0", Species::HO2),
            ("HO_2^-1", Species::HO2m),
            ("°O^0", Species::Oxy),
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
    fn chemistry_names_are_unique_and_round_trip() {
        for (i, s) in Species::ALL.iter().enumerate() {
            assert_eq!(Species::from_chemistry_name(s.chemistry_name()), Some(*s));
            assert_eq!(s.index(), i);
            for t in &Species::ALL[i + 1..] {
                assert_ne!(s.chemistry_name(), t.chemistry_name());
            }
        }
        assert_eq!(Species::from_chemistry_name("nonsense"), None);
    }

    #[test]
    fn exact_chemistry_names_round_trip_in_geant4_order() {
        let names = [
            "H3Op", "°OH", "OHm", "e_aq", "H", "H2", "H2O2", "HO2°", "HO2m", "Oxy", "Om", "O2",
            "O2m", "O3", "O3m",
        ];
        for (species, name) in Species::ALL.into_iter().zip(names) {
            assert_eq!(species.chemistry_name().as_bytes(), name.as_bytes());
            assert_eq!(Species::from_chemistry_name(name), Some(species));
        }
    }

    #[test]
    fn canonical_parser_is_strict_and_legacy_parser_accepts_ascii_tags() {
        for species in Species::ALL {
            assert_eq!(
                Species::from_legacy_ascii_tag(species.chemistry_name()),
                Some(species)
            );
        }
        assert_eq!(Species::from_legacy_ascii_tag("OH"), Some(Species::OH));
        assert_eq!(Species::from_legacy_ascii_tag("HO2"), Some(Species::HO2));
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
            species: Species::e_aq,
            position: Vec3::new(1.0, -2.5, 0.25),
            time: Time::from_ps(1.0),
        };
        assert_eq!(r.csv_row(), "e_aq,1.0,-2.5,0.25,1.0");
        assert_eq!(
            r.csv_row().split(',').count(),
            SPECIES_CSV_HEADER.split(',').count()
        );
        for species in Species::ALL {
            let row = SpeciesInstance { species, ..r }.csv_row();
            assert_eq!(row.split(',').next(), Some(species.chemistry_name()));
        }
    }
}
