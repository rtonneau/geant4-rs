//! Species table: diffusion coefficient, charge and van der Waals radius of every species of the
//! option3 chemistry (`data/chemistry/species_option3.toml`, provenance and line numbers in the file).

use crate::error::ChemistryError;
use dna_core::species::Species;
use dna_core::units::{Diffusivity, Length};
use serde::Deserialize;

const FILE: &str = "species_option3.toml";

/// Embedded default data (the file is part of the repository, `data/chemistry/`).
pub const DEFAULT_SPECIES_TOML: &str = include_str!("../../../data/chemistry/species_option3.toml");

#[derive(Debug, Deserialize)]
struct RawFile {
    species: Vec<RawSpecies>,
}

#[derive(Debug, Deserialize)]
struct RawSpecies {
    tag: String,
    diffusion_m2_per_s: f64,
    charge: i8,
    van_der_waals_radius_nm: f64,
    reachable_from_physchem: bool,
}

/// Properties of one species. Geant4 keeps them in `G4MolecularConfiguration`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeciesProps {
    pub species: Species,
    /// Diffusion coefficient (Geant4: `SetDiffusionCoefficient`, m^2/s in the file, nm^2/ps here).
    pub diffusivity: Diffusivity,
    /// Charge in units of e.
    pub charge: i8,
    /// Van der Waals radius (`SetVanDerVaalsRadius`).
    pub radius: Length,
    /// True for the species that a channel of the physicochemical stage can produce.
    pub reachable_from_physchem: bool,
}

/// Elemental composition `(hydrogen atoms, oxygen atoms)` of a species. Charge is a table property.
pub const fn atoms(species: Species) -> (i32, i32) {
    match species {
        Species::H3Op => (3, 1),
        Species::Oh | Species::Ohm => (1, 1),
        Species::EAq => (0, 0),
        Species::H => (1, 0),
        Species::H2 => (2, 0),
        Species::H2o2 => (2, 2),
        Species::Ho2 | Species::Ho2m => (1, 2),
        Species::O | Species::Om => (0, 1),
        Species::O2 | Species::O2m => (0, 2),
        Species::O3 | Species::O3m => (0, 3),
    }
}

/// All species, indexed by [`Species::index`].
#[derive(Debug, Clone, PartialEq)]
pub struct SpeciesTable {
    props: Vec<SpeciesProps>,
}

impl SpeciesTable {
    /// The embedded option3 table.
    pub fn option3() -> Result<Self, ChemistryError> {
        Self::from_toml(DEFAULT_SPECIES_TOML)
    }

    /// Parse and validate a table: every species exactly once, positive `D` and radius.
    pub fn from_toml(text: &str) -> Result<Self, ChemistryError> {
        let raw: RawFile = toml::from_str(text).map_err(|e| ChemistryError::Parse {
            file: FILE,
            message: e.to_string(),
        })?;
        let mut slots: Vec<Option<SpeciesProps>> = vec![None; Species::ALL.len()];
        for r in raw.species {
            let species = Species::from_tag(&r.tag)
                .ok_or_else(|| ChemistryError::invalid(FILE, format!("unknown species tag '{}'", r.tag)))?;
            if r.diffusion_m2_per_s <= 0.0 || r.van_der_waals_radius_nm <= 0.0 {
                return Err(ChemistryError::invalid(
                    FILE,
                    format!("{}: diffusion coefficient and radius must be positive", r.tag),
                ));
            }
            if slots[species.index()].is_some() {
                return Err(ChemistryError::invalid(FILE, format!("species '{}' listed twice", r.tag)));
            }
            slots[species.index()] = Some(SpeciesProps {
                species,
                diffusivity: Diffusivity::from_m2_per_s(r.diffusion_m2_per_s),
                charge: r.charge,
                radius: Length::from_nm(r.van_der_waals_radius_nm),
                reachable_from_physchem: r.reachable_from_physchem,
            });
        }
        let mut props = Vec::with_capacity(slots.len());
        for (s, slot) in Species::ALL.iter().zip(slots) {
            props.push(slot.ok_or_else(|| {
                ChemistryError::invalid(FILE, format!("species '{}' missing", s.tag()))
            })?);
        }
        Ok(Self { props })
    }

    pub fn get(&self, species: Species) -> &SpeciesProps {
        &self.props[species.index()]
    }

    pub fn iter(&self) -> impl Iterator<Item = &SpeciesProps> {
        self.props.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_table_loads_and_converts_units() {
        let t = SpeciesTable::option3().unwrap();
        // 1 m^2/s = 1e6 nm^2/ps (docs/units.md): e_aq 4.9e-9 m^2/s = 4.9e-3 nm^2/ps.
        assert!((t.get(Species::EAq).diffusivity.nm2_per_ps() - 4.9e-3).abs() < 1e-15);
        assert_eq!(t.get(Species::EAq).charge, -1);
        assert_eq!(t.get(Species::H3Op).charge, 1);
        assert_eq!(t.get(Species::Oh).radius.nm(), 0.22);
    }

    #[test]
    fn missing_or_duplicate_species_are_rejected() {
        let one = "[[species]]\ntag='H'\ndiffusion_m2_per_s=7e-9\ncharge=0\nvan_der_waals_radius_nm=0.19\nreachable_from_physchem=true\n";
        assert!(matches!(
            SpeciesTable::from_toml(one),
            Err(ChemistryError::Invalid { .. })
        ));
        let dup = DEFAULT_SPECIES_TOML.replacen("tag = \"OH\"", "tag = \"H\"", 1);
        assert!(matches!(
            SpeciesTable::from_toml(&dup),
            Err(ChemistryError::Invalid { .. })
        ));
        assert!(matches!(
            SpeciesTable::from_toml("not toml ["),
            Err(ChemistryError::Parse { .. })
        ));
    }
}
