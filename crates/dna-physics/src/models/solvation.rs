//! Electron solvation (thermalisation): `G4DNAElectronSolvation` with the default
//! `G4DNAOneStepThermalizationModel` = `G4TDNAOneStepThermalizationModel<Meesungnoen2002>`.
//!
//! Geant4 references (11.4.1, tree `27ea526b`, `D` = `source/processes/electromagnetic/dna`):
//! `D/processes/src/G4DNAElectronSolvation.cc`, `D/models/include/G4DNAOneStepThermalizationModel.hh`
//! and `.hpp` (`SampleSecondaries`, `CrossSectionPerVolume`), `D/models/src/
//! G4DNAOneStepThermalizationModel.cc` (penetration laws), builder
//! `physics_lists/constructors/electromagnetic/src/G4EmDNABuilder.cc:318-335` (high limit
//! `emaxT = 7.4 eV` for option2, `therm->SetHighEnergyLimit(emaxT)`), default sub-type
//! `utils/src/G4EmLowEParameters.cc:78` (`fMeesungnoen2002eSolvation`).
//!
//! * Window `[0, 7.4 eV]`, closed (`CrossSectionPerVolume` returns 0 only for `E > High`).
//! * Inside the window the "cross section" is `DBL_MAX` (water present): the interaction length
//!   is ~0, so the process always wins the competition against the finite cross sections of the
//!   other processes. Here it is therefore applied **before** the free-path draw: no free path,
//!   no random number for the choice.
//! * Final state: the track is killed and the **whole kinetic energy is the local deposit**
//!   (the deposit and the kill are independent of whether chemistry is active; with chemistry on
//!   Geant4 additionally proposes a 25 meV kinetic energy for the killed track, which is not
//!   deposited anywhere). The solvated electron is created at `position + displacement`,
//!   displacement = three independent Gaussians of `sigma_1D = r_mean * sqrt(pi) / 2^{3/2}
//!   = 0.62665706865775006 r_mean` (so that the mean of the resulting Maxwell-distributed 3-D
//!   radius equals `r_mean`), `r_mean(E) = nm * sum_{i=0}^{12} c[12 - i] E[eV]^i` for `E >
//!   0.1 eV` (0 otherwise; fit of Meesungnoen et al., Radiat. Res. 158 (2002) 657, data until
//!   0.2 eV). If `r_mean == 0` the electron is displaced by 1e-3 nm in a random direction
//!   (`GetGaussianPenetrationFromRmean3D`). The geometry correction in Geant4 (`safety * 0.8`
//!   when the segment leaves the volume) is irrelevant in an infinite medium.
//!
//! `G4DNATransformElectronModel` (0 - 0.025 eV; kill, deposit `k`, solvated electron at the
//! *same* position) was read as well: no physics constructor of the 11.4.1 tree registers it
//! (only examples with the UHDR/scavenger physics lists do), so it is not part of option2 and
//! not implemented.
//!
//! Other penetration laws in the same factory (`Meesungnoen2002_amorphous`, `Terrisol1990`,
//! `Ritchie1994`, `Kreipl2009`) are not needed by option2 and left to M6.
//!
//! # Random numbers
//! Four uniforms for the three Gaussians (two Box-Muller pairs, the fourth variate discarded) or
//! two uniforms for the random direction when `r_mean == 0`. Geant4's `G4RandGauss` algorithm is
//! not reproduced, only the distribution.

use crate::direction::direction_from_angles;
use crate::error::PhysicsError;
use crate::sampling::{gaussian_pair, uniform};
pub use dna_core::penetration::{
    MEESUNGNOEN_COEFFICIENTS, SIGMA_1D_PER_RMEAN, meesungnoen_r_mean_nm,
};
use dna_core::units::Energy;
use dna_core::vec3::Vec3;
use rand::Rng;
use std::f64::consts::PI;

/// Upper limit of the solvation window for option2, eV (`G4EmDNABuilder.cc:318-322`).
pub const SOLVATION_HIGH_EV: f64 = 7.4;
/// Displacement used when `r_mean == 0`, nm (`1e-3 * nanometer`).
pub const SOLVATION_ZERO_RMEAN_DISPLACEMENT_NM: f64 = 1.0e-3;
/// What solvation of one electron produces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolvationOutcome {
    /// Local energy deposit: the whole kinetic energy.
    pub deposit: Energy,
    /// Displacement of the solvated electron from the point of thermalisation, nm.
    pub displacement_nm: Vec3,
    /// `r_mean` used for the displacement, nm.
    pub r_mean_nm: f64,
    /// `r_mean == 0` branch (energy <= 0.1 eV): fixed-length displacement in a random direction.
    pub zero_rmean_branch: bool,
}

/// Option2 solvation: electrons with `E <= 7.4 eV`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Solvation {
    high: Energy,
}

impl Default for Solvation {
    fn default() -> Self {
        Self::new()
    }
}

impl Solvation {
    pub fn new() -> Self {
        Self {
            high: Energy::from_ev(SOLVATION_HIGH_EV),
        }
    }

    /// Upper limit of the window (inclusive).
    pub fn high_limit(&self) -> Energy {
        self.high
    }

    /// True if the electron is solvated rather than transported: `E <= 7.4 eV` (closed).
    pub fn applies(&self, energy: Energy) -> bool {
        energy <= self.high
    }

    /// Kill the electron, deposit its kinetic energy, and draw the displacement of the solvated
    /// electron. `UnsupportedEnergy` above the window.
    pub fn sample<R: Rng + ?Sized>(
        &self,
        energy: Energy,
        rng: &mut R,
    ) -> Result<SolvationOutcome, PhysicsError> {
        if !self.applies(energy) {
            return Err(dna_core::error::DnaError::UnsupportedEnergy {
                energy_ev: energy.ev(),
                min_ev: 0.0,
                max_ev: self.high.ev(),
            }
            .into());
        }
        let r_mean = meesungnoen_r_mean_nm(energy.ev());
        let (displacement, zero) = if r_mean == 0.0 {
            // G4RandomDirection(): cos(theta) = 2u - 1, phi = 2 pi u'.
            let cos_theta = 2.0 * uniform(rng) - 1.0;
            let phi = 2.0 * PI * uniform(rng);
            (
                direction_from_angles(cos_theta, phi, Vec3::new(0.0, 0.0, 1.0))
                    * SOLVATION_ZERO_RMEAN_DISPLACEMENT_NM,
                true,
            )
        } else {
            let sigma = r_mean * SIGMA_1D_PER_RMEAN;
            let (x, y) = gaussian_pair(rng);
            let (z, _unused) = gaussian_pair(rng);
            (Vec3::new(sigma * x, sigma * y, sigma * z), false)
        };
        Ok(SolvationOutcome {
            deposit: energy,
            displacement_nm: displacement,
            r_mean_nm: r_mean,
            zero_rmean_branch: zero,
        })
    }
}
