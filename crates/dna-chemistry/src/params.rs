//! Parameters of the physicochemical stage (`data/chemistry/physchem_option3.toml`) and the electron-hole
//! recombination of `G4DNAElectronHoleRecombination`.
//!
//! # Electron-hole recombination
//! Geant4 option3 registers, for the water molecule, a rest process of ordering 2 next to the dissociation
//! (ordering 1) (`G4EmDNAChemistry_option3.cc:733, 741`). For an ionised molecule (charge +1) the process
//! looks for solvated electrons within `10 r_Onsager`; the molecule recombines with the **nearest alive**
//! one with probability `1 - exp(-r_Onsager / d)` (1 if `d == 0`), becomes `H2Ovib` (the electron is
//! killed) and dissociates through the `H2Ovib` channels instead of `H3O+ + OH`. The recombination wins the
//! tie with the dissociation (both lifetimes are 0): the GPIL vector is the reverse of the DoIt vector
//! (`G4ProcessManager.cc:1135-1159`) and the step processor keeps the first strictly shortest lifetime
//! (`G4ITStepProcessor.cc:561-575`).
//!
//! `r_Onsager = e^2 / (4 pi eps0 k_B) / (T eps(rho, T))` with the dielectric-constant parameterisation of the
//! source file (`G4DNAElectronHoleRecombination.cc:57-103`), `T = 293.15 K` (the temperature of `G4_WATER`, not
//! the 298 K of the chemistry, docs/09 Q23) and `rho = 1 g/cm^3`. The CLHEP constants of the reference tree are
//! used (`k_Boltzmann = 8.617333e-11 MeV/K`, `mu0 = 4 pi 1e-7 henry/m`, hence `e^2/(4 pi eps0) = e_SI^2 c^2 1e-7`).

use crate::error::ChemistryError;
use dna_core::units::{Energy, Length, Time};
use serde::Deserialize;

const FILE: &str = "physchem_option3.toml";

/// Embedded default data.
pub const DEFAULT_PHYSCHEM_TOML: &str = include_str!("../../../data/chemistry/physchem_option3.toml");

/// Speed of light, m/s (exact; CLHEP `c_light = 299.792458 mm/ns`).
const C_LIGHT_M_PER_S: f64 = 299_792_458.0;

#[derive(Debug, Deserialize)]
struct RawFile {
    creation: RawCreation,
    autoionisation: RawAuto,
    recombination: RawRecombination,
}

#[derive(Debug, Deserialize)]
struct RawCreation {
    birth_time_ps: f64,
}

#[derive(Debug, Deserialize)]
struct RawAuto {
    electron_kinetic_energy_ev: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct RawRecombination {
    enabled: bool,
    search_radius_in_onsager_radii: f64,
    temperature_k: f64,
    density_g_cm3: f64,
    y0: f64,
    a: [f64; 4],
    b: [f64; 3],
    s: [f64; 3],
    k_boltzmann_mev_per_k: f64,
    e_si: f64,
}

/// Electron-hole recombination model.
#[derive(Debug, Clone, PartialEq)]
pub struct Recombination {
    pub enabled: bool,
    pub temperature_k: f64,
    pub density_g_cm3: f64,
    /// Dielectric constant of the water at `(temperature, density)`.
    pub dielectric_constant: f64,
    pub onsager_radius: Length,
    /// Search radius, `search_radius_in_onsager_radii * r_Onsager`.
    pub search_radius: Length,
}

impl Recombination {
    fn from_raw(r: &RawRecombination) -> Result<Self, ChemistryError> {
        if r.temperature_k <= 0.0 || r.density_g_cm3 <= 0.0 || r.search_radius_in_onsager_radii <= 0.0 {
            return Err(ChemistryError::invalid(FILE, "recombination: temperature, density and search radius must be positive"));
        }
        let t = r.temperature_k;
        let poly = |c: &[f64]| c.iter().enumerate().map(|(i, &x)| x / t.powi(i as i32)).sum::<f64>();
        let (a, b, s) = (poly(&r.a), poly(&r.b), poly(&r.s));
        let (c, d) = (a - b - 3.0, b + 3.0);
        let y = 1.0 / (1.0 + r.y0 / (r.density_g_cm3 * r.density_g_cm3));
        let lr = r.density_g_cm3.log10();
        let epsilon = 1.0 + 10f64.powf(y * (c + (s - 1.0) * lr) + d + lr);
        // e^2 / (4 pi eps0 k_B) = e_SI^2 c^2 1e-7 / k_B[J/K], k_B[J/K] = k_B[MeV/K] * 1e6 * e_SI
        let k_b_j_per_k = r.k_boltzmann_mev_per_k * 1.0e6 * r.e_si;
        let onsager_constant_m_k = r.e_si * r.e_si * C_LIGHT_M_PER_S * C_LIGHT_M_PER_S * 1.0e-7 / k_b_j_per_k;
        let onsager_nm = onsager_constant_m_k / (t * epsilon) * 1.0e9;
        Ok(Self {
            enabled: r.enabled,
            temperature_k: t,
            density_g_cm3: r.density_g_cm3,
            dielectric_constant: epsilon,
            onsager_radius: Length::from_nm(onsager_nm),
            search_radius: Length::from_nm(onsager_nm * r.search_radius_in_onsager_radii),
        })
    }

    /// Recombination probability with an electron at `distance_nm` (cc:268-273): `1 - exp(-r_O / d)`, 1 at `d == 0`.
    pub fn probability(&self, distance_nm: f64) -> f64 {
        if distance_nm != 0.0 {
            1.0 - (-self.onsager_radius.nm() / distance_nm).exp()
        } else {
            1.0
        }
    }
}

/// All parameters of the stage.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysChemParams {
    /// Time at which every initial species is placed (1 ps).
    pub birth_time: Time,
    /// Kinetic energy used for the penetration of the auto-ionisation electron (1.7 eV).
    pub autoionisation_electron_energy: Energy,
    pub recombination: Recombination,
}

impl PhysChemParams {
    pub fn option3() -> Result<Self, ChemistryError> {
        Self::from_toml(DEFAULT_PHYSCHEM_TOML)
    }

    pub fn from_toml(text: &str) -> Result<Self, ChemistryError> {
        let raw: RawFile = toml::from_str(text).map_err(|e| ChemistryError::Parse {
            file: FILE,
            message: e.to_string(),
        })?;
        if raw.creation.birth_time_ps < 0.0 || raw.autoionisation.electron_kinetic_energy_ev <= 0.1 {
            return Err(ChemistryError::invalid(
                FILE,
                "birth time must be >= 0 and the auto-ionisation electron energy above 0.1 eV (r_mean > 0)",
            ));
        }
        Ok(Self {
            birth_time: Time::from_ps(raw.creation.birth_time_ps),
            autoionisation_electron_energy: Energy::from_ev(raw.autoionisation.electron_kinetic_energy_ev),
            recombination: Recombination::from_raw(&raw.recombination)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_parameters_load() {
        let p = PhysChemParams::option3().unwrap();
        assert_eq!(p.birth_time.ps(), 1.0);
        assert_eq!(p.autoionisation_electron_energy.ev(), 1.7);
        assert!(p.recombination.enabled);
        let r = &p.recombination;
        assert!((r.search_radius.nm() / r.onsager_radius.nm() - 10.0).abs() < 1e-12);
        assert_eq!(r.probability(0.0), 1.0);
        assert!(r.probability(1.0) < r.probability(0.5));
    }

    #[test]
    fn invalid_parameters_are_rejected() {
        let bad = DEFAULT_PHYSCHEM_TOML.replacen("temperature_k = 293.15", "temperature_k = -1.0", 1);
        assert!(PhysChemParams::from_toml(&bad).is_err());
        assert!(PhysChemParams::from_toml("[creation]").is_err());
    }
}
