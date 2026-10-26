//! `G4DNAMeltonAttachmentModel`: dissociative electron attachment to water (Melton 1972).
//!
//! Geant4 reference: `dna/models/src/G4DNAMeltonAttachmentModel.cc` (11.4.1, tree `27ea526b`).
//!
//! * Window `[4 eV, 13 eV]`, closed (`:49-50`, `CrossSectionPerVolume` `:199`).
//! * Data `dna/sigma_attachment_e_melton.dat`: one column in `1e-18 cm^2` (`:141`), log-log
//!   through `G4DNACrossSectionDataSet` (`:144-147`). The table covers 0 - 12.3 eV.
//! * Final state (`:225-250`): the electron is killed (`fStopAndKill`) and its **whole kinetic
//!   energy is the local deposit**; the chemistry manager creates a dissociative-attachment
//!   event (M6 interface, `PhysChemEvent`). No random number is drawn.
//!
//! # Decision: 12.3 - 13 eV (docs/02 B.7.1, docs/10)
//! `G4EMDataSet::FindValue` returns the last node's value for any energy above the table, and the
//! last node (12.3 eV) is 0, so Geant4 returns **zero** between 12.3 and 13 eV. The M1 table
//! layer deliberately errors outside its range; this model reproduces the Geant4 value
//! *explicitly*: above the last node the cross section is 0, and construction fails unless the
//! last node is exactly 0 (otherwise a constant-clamp would be a silent extrapolation of a
//! non-zero cross section). Between 4 and 4.3 eV Geant4's log-log rule with the `1e-300` floor of
//! zero nodes gives a numerically-zero cross section; reproduced by the table layer.

use super::{CrossSectionModel, FinalState, ProcessKind, Quirks, window};
use crate::error::PhysicsError;
use g4_core::units::{Area, Energy, EnergyRange};
use g4_core::vec3::Vec3;
use g4_data::catalog::{DnaFile, Table};
use g4_data::datadir::DataDir;
use g4_data::xs::CrossSectionTable;
use rand::Rng;

/// Model window in eV (`G4DNAMeltonAttachmentModel.cc:49-50`).
pub const MELTON_WINDOW_EV: (f64, f64) = (4.0, 13.0);

#[derive(Debug, Clone)]
pub struct MeltonAttachment {
    table: CrossSectionTable,
    window: EnergyRange,
}

impl MeltonAttachment {
    pub fn new(table: CrossSectionTable) -> Result<Self, PhysicsError> {
        let window = window(MELTON_WINDOW_EV.0, MELTON_WINDOW_EV.1);
        let range = table.energy_range();
        if range.min() > window.min() {
            return Err(PhysicsError::TableDoesNotCoverWindow {
                table: "sigma_attachment_e_melton",
                model: "G4DNAMeltonAttachmentModel",
                table_min_ev: range.min().ev(),
                table_max_ev: range.max().ev(),
                min_ev: window.min().ev(),
                max_ev: window.max().ev(),
            });
        }
        if range.max() < window.max() {
            let last = *table
                .node_values_nm2(0)?
                .last()
                .expect("a table has at least two rows");
            if last != 0.0 {
                return Err(PhysicsError::TableTailNotZero {
                    table: "sigma_attachment_e_melton",
                    last_energy_ev: range.max().ev(),
                    last_value_nm2: last,
                });
            }
        }
        Ok(Self { table, window })
    }

    pub fn from_data_dir(dir: &DataDir) -> Result<Self, PhysicsError> {
        match dir.load(DnaFile::MeltonAttachmentE)? {
            Table::CrossSection(t) => Self::new(t),
            _ => unreachable!("catalog: MeltonAttachmentE is a cross-section table"),
        }
    }

    /// Last tabulated energy, eV (12.3 for the committed file).
    pub fn table_last_energy_ev(&self) -> f64 {
        self.table.energy_range().max().ev()
    }

    /// The electron is absorbed: no draw, whole kinetic energy deposited.
    pub fn sample<R: Rng + ?Sized>(
        &self,
        energy: Energy,
        direction: Vec3,
        _rng: &mut R,
    ) -> Result<FinalState, PhysicsError> {
        self.window.check(energy)?;
        Ok(FinalState {
            energy_after: Energy::from_ev(0.0),
            direction_after: direction,
            local_deposit: energy,
            secondary: None,
            channel: 0,
            quirks: Quirks::default(),
            killed: true,
        })
    }
}

impl CrossSectionModel for MeltonAttachment {
    fn name(&self) -> &'static str {
        "G4DNAMeltonAttachmentModel"
    }
    fn process(&self) -> ProcessKind {
        ProcessKind::Attachment
    }
    fn energy_range(&self) -> EnergyRange {
        self.window
    }
    fn cross_section(&self, energy: Energy) -> Result<Area, PhysicsError> {
        if !self.window.contains(energy) {
            return Ok(Area::from_nm2(0.0));
        }
        if energy > self.table.energy_range().max() {
            // Geant4: constant clamp to the last node, which is 0 (checked in `new`).
            return Ok(Area::from_nm2(0.0));
        }
        Ok(self.table.total(energy)?)
    }
}
