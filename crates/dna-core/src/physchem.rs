//! Physicochemical event record: the interface between the physical stage (`dna-physics`) and the
//! physicochemical stage (`dna-chemistry`). Defined in `dna-core` (moved there at M6, docs/03) so
//! that the two stage crates never depend on each other. Data model only; nothing here
//! simulates chemistry.
//!
//! Every interaction that leaves a chemically relevant state behind produces one
//! [`PhysChemEvent`]. In Geant4 these are the calls of `G4DNAChemistryManager::CreateWaterMolecule`
//! (ionised/excited/attached water molecule) and `CreateSolvatedElectron`:
//!
//! | Kind | Geant4 trigger | `level_or_shell` | `deposited_energy` | `position` |
//! |---|---|---|---|---|
//! | `Excitation` | Born excitation (`G4DNABornExcitationModel1.cc:208-212`) | level 0..4 | excitation energy (0 for the reproduced "no-op" quirk, where the molecule is created anyway) | interaction point |
//! | `Ionisation` | Born1 ionisation | shell 0..4 | binding energy | interaction point |
//! | `Attachment` | Melton (`eDissociativeAttachment`) | 0 | whole kinetic energy of the absorbed electron | interaction point |
//! | `Solvation` | one-step thermalisation | 0 | whole kinetic energy of the electron | **displaced** position of the solvated electron (the deposit itself is booked at the point of thermalisation) |
//!
//! Vibrational excitation creates no chemical species and is not an event.
//! Energies are eV, positions nm (docs/units.md).

use crate::units::Energy;
use crate::vec3::Vec3;

/// Kind of physicochemical event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysChemKind {
    Excitation,
    Ionisation,
    Attachment,
    Solvation,
}

impl PhysChemKind {
    /// Stable lower-case name used in the CSV output.
    pub const fn name(self) -> &'static str {
        match self {
            PhysChemKind::Excitation => "excitation",
            PhysChemKind::Ionisation => "ionisation",
            PhysChemKind::Attachment => "attachment",
            PhysChemKind::Solvation => "solvation",
        }
    }
}

/// One physicochemical event (see the module table for the meaning of the fields).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysChemEvent {
    pub kind: PhysChemKind,
    pub level_or_shell: u8,
    pub position: Vec3,
    pub deposited_energy: Energy,
    /// Id of the primary history (the cascade) that produced the event.
    pub parent_history: u64,
}

/// Header line of the CSV written by `dna-sim` (`PhysChemEvent::csv_row` gives the rows).
pub const PHYSCHEM_CSV_HEADER: &str =
    "parent_history,kind,level_or_shell,x_nm,y_nm,z_nm,deposited_energy_eV";

impl PhysChemEvent {
    /// One CSV row (no newline); floats with the shortest round-trip representation.
    pub fn csv_row(&self) -> String {
        format!(
            "{},{},{},{:?},{:?},{:?},{:?}",
            self.parent_history,
            self.kind.name(),
            self.level_or_shell,
            self.position.x,
            self.position.y,
            self.position.z,
            self.deposited_energy.ev()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_row_matches_the_header_columns_and_round_trips() {
        let e = PhysChemEvent {
            kind: PhysChemKind::Ionisation,
            level_or_shell: 3,
            position: Vec3::new(1.5, -2.25, 0.1),
            deposited_energy: Energy::from_ev(32.3),
            parent_history: 7,
        };
        let row = e.csv_row();
        assert_eq!(row, "7,ionisation,3,1.5,-2.25,0.1,32.3");
        assert_eq!(
            row.split(',').count(),
            PHYSCHEM_CSV_HEADER.split(',').count()
        );
    }
}
