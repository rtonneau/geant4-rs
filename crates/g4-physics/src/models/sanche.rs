//! `G4DNASancheExcitationModel`: vibrational excitation of liquid water by 2-100 eV electrons
//! (Sanche, Michaud et al.).
//!
//! Geant4 reference: `dna/models/src/G4DNASancheExcitationModel.cc` (11.4.1, tree `27ea526b`),
//! read line by line for M3.
//!
//! * Window `[2 eV, 100 eV]`, closed (`:48-49`, `CrossSectionPerVolume` `:207`).
//! * Data `dna/sigma_excitationvib_e_sanche.dat`: energy (eV) and nine level columns in
//!   `1e-16 cm^2` (`:309`), **linear-linear** interpolation (own reader `:139-178`, own
//!   `LinInterpolate` `:407-418`, the `t (1 - 1e-12)` protection at the last node `:290-296`).
//! * Cross section per volume (`:182-237`): `TotalCrossSection(E)` (interpolation of the **sum of
//!   the nine columns at the nodes**) `x 2 x n_water`; the factor 2 is Geant4's liquid-phase
//!   scaling ("see papers for factor 2 description (liquid phase)"). A zero result is replaced by
//!   `1e-30` (:312, :339) **in Geant4 internal units**, i.e. 1e-30 mm^2 = 1e-18 nm^2.
//! * Final state (`:240-279`): level `i` chosen with probability proportional to the nine
//!   `PartialCrossSection(E, i)` (each interpolated on its own column, zero replaced by the same
//!   floor), searching from level 8 downwards ([`random_select`]); `E_vib = {0.010, 0.024, 0.061,
//!   0.092, 0.204, 0.417, 0.460, 0.500, 0.835} eV` (`:347`); `E' = E - E_vib`; direction
//!   unchanged; local deposit `E_vib`.
//!
//! # Quirks (counted, not fixed)
//! 1. The "kill and deposit everything" branch quoted in the M3 brief (`:249-262`) is **commented
//!    out** in the 11.4.1 source. The live code is `if (E <= High && E' > 0) { change }`, and
//!    nothing happens otherwise. With every level below 0.835 eV and `E >= 2 eV` the `else` is
//!    unreachable inside the window; it is reproduced and counted
//!    ([`Quirks::vibrational_noop`](super::Quirks)).
//! 2. The sum of partials and `TotalCrossSection` are interpolated separately; they are equal up
//!    to rounding (linear interpolation of a sum is the sum of linear interpolations).
//! 3. `statCode` (stationary mode) is always `false` in option2 and not modelled.

use super::{
    CrossSectionModel, FinalState, ProcessKind, Quirks, ensure_table_covers, random_select, window,
};
use crate::error::PhysicsError;
use crate::sampling::uniform;
use g4_core::units::{Area, Energy, EnergyRange};
use g4_core::vec3::Vec3;
use g4_data::catalog::{DnaFile, Table};
use g4_data::datadir::DataDir;
use g4_data::xs::CrossSectionTable;
use rand::Rng;

/// Model window in eV (`G4DNASancheExcitationModel.cc:48-49`).
pub const SANCHE_WINDOW_EV: (f64, f64) = (2.0, 100.0);
/// Number of vibrational levels (`nLevels = 9`, `:50`).
pub const SANCHE_LEVELS: usize = 9;
/// Liquid-phase scaling applied to the total cross section (`return sigma*2.*waterDensity`, `:222`).
pub const SANCHE_LIQUID_PHASE_FACTOR: f64 = 2.0;
/// Level energies, eV (`VibrationEnergy`, `:347`).
const VIBRATION_ENERGIES_EV: [f64; SANCHE_LEVELS] =
    [0.01, 0.024, 0.061, 0.092, 0.204, 0.417, 0.460, 0.500, 0.835];
/// Replacement of a zero interpolated cross section: `1e-30` in Geant4 internal units (mm^2).
const ZERO_FLOOR_NM2: f64 = 1e-30 * 1e12;

/// Energy lost in vibrational level `level` (0-based).
pub fn vibration_energy(level: usize) -> Energy {
    Energy::from_ev(VIBRATION_ENERGIES_EV[level])
}

#[derive(Debug, Clone)]
pub struct SancheVibrational {
    table: CrossSectionTable,
    window: EnergyRange,
}

impl SancheVibrational {
    pub fn new(table: CrossSectionTable) -> Result<Self, PhysicsError> {
        let window = window(SANCHE_WINDOW_EV.0, SANCHE_WINDOW_EV.1);
        ensure_table_covers(
            "sigma_excitationvib_e_sanche",
            "G4DNASancheExcitationModel",
            table.energy_range(),
            window,
        )?;
        debug_assert_eq!(table.n_components(), SANCHE_LEVELS);
        Ok(Self { table, window })
    }

    pub fn from_data_dir(dir: &DataDir) -> Result<Self, PhysicsError> {
        match dir.load(DnaFile::SancheVibrationalE)? {
            Table::CrossSection(t) => Self::new(t),
            _ => unreachable!("catalog: SancheVibrationalE is a cross-section table"),
        }
    }

    fn floored(nm2: f64) -> f64 {
        if nm2 == 0.0 { ZERO_FLOOR_NM2 } else { nm2 }
    }

    /// `PartialCrossSection` of the nine levels (nm^2, per molecule, without the liquid-phase
    /// factor), each with the zero floor.
    pub fn partial_cross_sections(
        &self,
        energy: Energy,
    ) -> Result<[f64; SANCHE_LEVELS], PhysicsError> {
        let mut v = [0.0; SANCHE_LEVELS];
        for (level, slot) in v.iter_mut().enumerate() {
            *slot = Self::floored(self.table.component(level, energy)?.nm2());
        }
        Ok(v)
    }

    /// `TotalCrossSection` (nm^2, per molecule, without the liquid-phase factor), with the zero
    /// floor.
    pub fn total_cross_section(&self, energy: Energy) -> Result<f64, PhysicsError> {
        Ok(Self::floored(self.table.total(energy)?.nm2()))
    }

    /// Final state: draws one uniform (the level).
    pub fn sample<R: Rng + ?Sized>(
        &self,
        energy: Energy,
        direction: Vec3,
        rng: &mut R,
    ) -> Result<FinalState, PhysicsError> {
        self.window.check(energy)?;
        let partials = self.partial_cross_sections(energy)?;
        let level = random_select(&partials, uniform(rng));
        let e_vib = vibration_energy(level);
        let new_energy = energy - e_vib;
        if new_energy.ev() > 0.0 {
            Ok(FinalState {
                energy_after: new_energy,
                direction_after: direction,
                local_deposit: e_vib,
                secondary: None,
                channel: level,
                quirks: Quirks::default(),
                killed: false,
            })
        } else {
            Ok(FinalState {
                energy_after: energy,
                direction_after: direction,
                local_deposit: Energy::from_ev(0.0),
                secondary: None,
                channel: level,
                quirks: Quirks {
                    vibrational_noop: true,
                    ..Quirks::default()
                },
                killed: false,
            })
        }
    }
}

impl CrossSectionModel for SancheVibrational {
    fn name(&self) -> &'static str {
        "G4DNASancheExcitationModel"
    }
    fn process(&self) -> ProcessKind {
        ProcessKind::VibrationalExcitation
    }
    fn energy_range(&self) -> EnergyRange {
        self.window
    }
    fn cross_section(&self, energy: Energy) -> Result<Area, PhysicsError> {
        if !self.window.contains(energy) {
            return Ok(Area::from_nm2(0.0));
        }
        Ok(Area::from_nm2(
            self.total_cross_section(energy)? * SANCHE_LIQUID_PHASE_FACTOR,
        ))
    }
}
