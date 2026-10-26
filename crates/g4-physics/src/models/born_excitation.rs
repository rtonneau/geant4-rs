//! `G4DNABornExcitationModel1` (alias `G4DNABornExcitationModel`): electronic excitation of water
//! by electrons.
//!
//! Geant4 reference: `dna/models/src/G4DNABornExcitationModel1.cc`.
//!
//! * Window for electrons: `[9 eV, 1 MeV]`, closed (`Initialise` `:98-100, 109-110`; cross-section
//!   test `:157`). The option2 builder passes a lower limit of 0 and `emaxDNA = 1 MeV`
//!   (`G4EmDNABuilder.cc:382-383`) but `Initialise` overrides the limits with the model's own.
//! * Cross section: `dna/sigma_excitation_e_born.dat`, five levels, value scale
//!   `(1e-22/3.343) m^2`, log-log (`:112-114`); table range 8 eV - 1 MeV.
//! * Final state (`:177-210`): level chosen proportional to the partial cross sections
//!   ([`random_select`](super::random_select)); `E' = E - E_exc(level)` with the excitation
//!   structure `{8.22, 10.00, 11.24, 12.61, 13.77} eV`; direction unchanged; local deposit
//!   `E_exc`. **Quirk (reproduced and counted):** when `E' <= 0` Geant4 does not touch the
//!   particle change (no loss, no deposit) but still creates the excited molecule for the
//!   chemistry ([`Quirks::excitation_noop`](super::Quirks)).
//! * Not modelled here: the stationary mode (`statCode`, `G4EmParameters::DNAStationary`), which
//!   option2 leaves at its default `false`.

use super::{
    CrossSectionModel, FinalState, ProcessKind, Quirks, ensure_table_covers, random_select, window,
};
use crate::error::PhysicsError;
use crate::sampling::uniform;
use crate::water::excitation_energy;
use g4_core::units::{Area, Energy, EnergyRange};
use g4_core::vec3::Vec3;
use g4_data::catalog::{DnaFile, Table};
use g4_data::datadir::{DataDir, VerifiedDataSet};
use g4_data::xs::CrossSectionTable;
use rand::Rng;

/// Model window in eV for electrons (`G4DNABornExcitationModel1.cc:99-100`).
pub const BORN_EXCITATION_WINDOW_EV: (f64, f64) = (9.0, 1.0e6);

#[derive(Debug, Clone)]
pub struct BornExcitation {
    table: CrossSectionTable,
    window: EnergyRange,
}

impl BornExcitation {
    pub fn new(table: CrossSectionTable) -> Result<Self, PhysicsError> {
        // Five levels in G4DNAWaterExcitationStructure.cc (Geant4 11.4.1).
        table.require_components(
            DnaFile::BornExcitationE.relative_path(),
            DnaFile::BornExcitationE.n_components(),
        )?;
        let window = window(BORN_EXCITATION_WINDOW_EV.0, BORN_EXCITATION_WINDOW_EV.1);
        ensure_table_covers(
            "sigma_excitation_e_born",
            "G4DNABornExcitationModel1",
            table.energy_range(),
            window,
        )?;
        Ok(Self { table, window })
    }

    pub fn from_data_dir(dir: &DataDir) -> Result<Self, PhysicsError> {
        match dir.load(DnaFile::BornExcitationE)? {
            Table::CrossSection(t) => Self::new(t),
            _ => unreachable!("catalog: BornExcitationE is a cross-section table"),
        }
    }

    /// Build from the checked snapshot without reading the source directory again.
    pub fn from_verified_data(data: &VerifiedDataSet) -> Result<Self, PhysicsError> {
        match data.table(DnaFile::BornExcitationE)? {
            Table::CrossSection(t) => Self::new(t.clone()),
            _ => unreachable!("catalog: BornExcitationE is a cross-section table"),
        }
    }

    /// Partial cross sections of the five levels at `energy` (nm^2), inside the table range.
    pub fn partial_cross_sections(&self, energy: Energy) -> Result<[f64; 5], PhysicsError> {
        let mut v = [0.0; 5];
        for (level, slot) in v.iter_mut().enumerate() {
            *slot = self.table.component(level, energy)?.nm2();
        }
        Ok(v)
    }

    /// Final state with the uniform variate for the level choice drawn from `rng`.
    pub fn sample<R: Rng + ?Sized>(
        &self,
        energy: Energy,
        direction: Vec3,
        rng: &mut R,
    ) -> Result<FinalState, PhysicsError> {
        self.window.check(energy)?;
        let partials = self.partial_cross_sections(energy)?;
        let level = random_select(&partials, uniform(rng));
        let e_exc = excitation_energy(level);
        let new_energy = energy - e_exc;
        if new_energy.ev() > 0.0 {
            Ok(FinalState {
                energy_after: new_energy,
                direction_after: direction,
                local_deposit: e_exc,
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
                    excitation_noop: true,
                    ..Quirks::default()
                },
                killed: false,
            })
        }
    }
}

impl CrossSectionModel for BornExcitation {
    fn name(&self) -> &'static str {
        "G4DNABornExcitationModel1"
    }
    fn process(&self) -> ProcessKind {
        ProcessKind::Excitation
    }
    fn energy_range(&self) -> EnergyRange {
        self.window
    }
    fn cross_section(&self, energy: Energy) -> Result<Area, PhysicsError> {
        if !self.window.contains(energy) {
            return Ok(Area::from_nm2(0.0));
        }
        Ok(self.table.total(energy)?)
    }
}
