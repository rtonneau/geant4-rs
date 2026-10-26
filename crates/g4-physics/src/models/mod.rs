//! Cross-section and final-state models of the option2 electron physics.
//!
//! Each model is a plain struct owning its immutable tables (hence `Send + Sync`) and exposing
//!
//! * [`CrossSectionModel`]: name, closed energy window, cross section per molecule (zero outside
//!   the window, as every `CrossSectionPerVolume` of the DNA models);
//! * an inherent `sample` method drawing the final state from a `&mut impl Rng` (the history's
//!   physics stream). Outside the window it returns `UnsupportedEnergy` instead of Geant4's
//!   silent "do nothing".
//!
//! The closed set of models of one particle is the enum [`ElectronModel`]; the transport loop
//! dispatches on it without virtual calls (docs/03, rule 2). Interchangeable implementations
//! needed at configuration time use [`CrossSectionModel`] as a trait.

mod born_angle;
mod born_excitation;
mod born_ionisation;
mod champion;
mod melton;
mod sanche;
mod select;
mod solvation;

pub use born_angle::{born_electron_cos_theta, sample_born_electron_direction};
pub use born_excitation::{BORN_EXCITATION_WINDOW_EV, BornExcitation};
pub use born_ionisation::{
    BORN_IONISATION_WINDOW_EV, BornIonisation, K_SHELL_INDEX, SHELL2_REJECTION_BELOW_EV,
};
pub use champion::{CHAMPION_WINDOW_EV, ChampionElastic, ElasticRotation, scatter};
pub use melton::{MELTON_WINDOW_EV, MeltonAttachment};
pub use sanche::{
    SANCHE_LEVELS, SANCHE_LIQUID_PHASE_FACTOR, SANCHE_WINDOW_EV, SancheVibrational,
    vibration_energy,
};
pub use select::random_select;
pub use solvation::{
    MEESUNGNOEN_COEFFICIENTS, SOLVATION_HIGH_EV, SOLVATION_ZERO_RMEAN_DISPLACEMENT_NM, Solvation,
    SolvationOutcome, meesungnoen_r_mean_nm,
};

use crate::error::PhysicsError;
use g4_core::units::{Area, Energy, EnergyRange};
use g4_core::vec3::Vec3;
use rand::Rng;
use serde::Serialize;

/// The discrete processes of the option2 electron physics that are driven by a cross section
/// (M3: five). Solvation (cross section `DBL_MAX` below 7.4 eV, i.e. an immediate interaction) is
/// not a member: it is handled by [`Solvation`] ahead of the free-path draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessKind {
    Elastic,
    /// Electronic excitation (Born).
    Excitation,
    Ionisation,
    /// Sanche vibrational excitation (2-100 eV).
    VibrationalExcitation,
    /// Melton dissociative attachment (4-13 eV).
    Attachment,
}

impl ProcessKind {
    /// All processes, in the fixed order used for selection and reporting.
    pub const ALL: [ProcessKind; 5] = [
        ProcessKind::Elastic,
        ProcessKind::Excitation,
        ProcessKind::Ionisation,
        ProcessKind::VibrationalExcitation,
        ProcessKind::Attachment,
    ];

    /// Number of processes (length of [`ProcessKind::ALL`]).
    pub const COUNT: usize = 5;

    /// Position in [`ProcessKind::ALL`].
    pub const fn index(self) -> usize {
        match self {
            ProcessKind::Elastic => 0,
            ProcessKind::Excitation => 1,
            ProcessKind::Ionisation => 2,
            ProcessKind::VibrationalExcitation => 3,
            ProcessKind::Attachment => 4,
        }
    }

    /// Geant4 process name for electrons (`G4EmDNABuilder::ConstructDNAElectronPhysics`).
    pub const fn geant4_name(self) -> &'static str {
        match self {
            ProcessKind::Elastic => "e-_G4DNAElastic",
            ProcessKind::Excitation => "e-_G4DNAExcitation",
            ProcessKind::Ionisation => "e-_G4DNAIonisation",
            ProcessKind::VibrationalExcitation => "e-_G4DNAVibExcitation",
            ProcessKind::Attachment => "e-_G4DNAAttachment",
        }
    }
}

/// A secondary electron produced by ionisation: kinetic energy and direction (position is that
/// of the interaction and is added by the transport loop).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Secondary {
    pub energy: Energy,
    pub direction: Vec3,
}

/// Geant4 behaviours that look like bugs or corner cases and are reproduced on purpose
/// (docs/02 B.7.3). Every occurrence is reported so that none is silent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Quirks {
    /// Born1: number of times the shell was re-drawn because `k < 19 eV` selected shell 2
    /// (`G4DNABornIonisationModel1.cc:464-466`).
    pub shell2_redraws: u32,
    /// Born excitation: `E - E_exc <= 0`, the final state is left untouched (no loss, no
    /// deposit) although the excitation was selected (`G4DNABornExcitationModel1.cc:196-210`).
    pub excitation_noop: bool,
    /// Born1: `k < B` of the selected shell, returns without any change (`:472`).
    pub ionisation_noop_below_binding: bool,
    /// Born1: the table gave `W - B < 0`, clamped to a zero-energy ejected electron (`:1003`).
    pub transfer_clamped_to_zero: bool,
    /// Born1: oxygen K-shell ionisation; Geant4 would call the atomic de-excitation
    /// (`:528-535`), which is **not** simulated here (docs/09 Q12). The full binding energy is
    /// booked as local deposit.
    pub k_shell_deexcitation_skipped: bool,
    /// Sanche: `E - E_vib <= 0`, the final state is left untouched although the process was
    /// selected (`G4DNASancheExcitationModel.cc:253`, the kill-and-deposit branch above it is
    /// commented out in the 11.4.1 source). Unreachable inside the 2-100 eV window
    /// (largest level 0.835 eV).
    pub vibrational_noop: bool,
}

/// Result of sampling one interaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FinalState {
    /// Kinetic energy of the primary after the interaction.
    pub energy_after: Energy,
    /// Unit direction of the primary after the interaction.
    pub direction_after: Vec3,
    /// Energy booked as local deposit: excitation energy for excitation, shell binding energy
    /// for ionisation, zero for elastic scattering (docs/01 A.4).
    pub local_deposit: Energy,
    /// Ionisation: the ejected electron (only if its energy is > 0, as Geant4).
    pub secondary: Option<Secondary>,
    /// Excitation level / ionisation shell (0 for elastic).
    pub channel: usize,
    pub quirks: Quirks,
    /// The track is killed by the interaction (Melton attachment: the whole kinetic energy is
    /// the local deposit and `energy_after` is zero).
    pub killed: bool,
}

/// Common, non-generic interface of a cross-section model.
pub trait CrossSectionModel {
    /// Geant4 class name.
    fn name(&self) -> &'static str;
    fn process(&self) -> ProcessKind;
    /// Closed validity window `[low, high]` of the model itself.
    fn energy_range(&self) -> EnergyRange;
    /// Cross section per water molecule; **zero outside the window** (Geant4 semantics), the
    /// table value inside. Never extrapolates.
    fn cross_section(&self, energy: Energy) -> Result<Area, PhysicsError>;
}

/// One of the option2 electron models.
#[derive(Debug, Clone)]
pub enum ElectronModel {
    Elastic(ChampionElastic),
    Excitation(BornExcitation),
    Ionisation(BornIonisation),
    Vibrational(SancheVibrational),
    Attachment(MeltonAttachment),
}

impl ElectronModel {
    /// Sample the final state for a primary of kinetic `energy` travelling along the unit vector
    /// `direction`. Errors with `UnsupportedEnergy` outside the model window.
    pub fn sample<R: Rng + ?Sized>(
        &self,
        energy: Energy,
        direction: Vec3,
        rng: &mut R,
    ) -> Result<FinalState, PhysicsError> {
        match self {
            ElectronModel::Elastic(m) => m.sample(energy, direction, rng),
            ElectronModel::Excitation(m) => m.sample(energy, direction, rng),
            ElectronModel::Ionisation(m) => m.sample(energy, direction, rng),
            ElectronModel::Vibrational(m) => m.sample(energy, direction, rng),
            ElectronModel::Attachment(m) => m.sample(energy, direction, rng),
        }
    }

    fn inner(&self) -> &dyn CrossSectionModel {
        match self {
            ElectronModel::Elastic(m) => m,
            ElectronModel::Excitation(m) => m,
            ElectronModel::Ionisation(m) => m,
            ElectronModel::Vibrational(m) => m,
            ElectronModel::Attachment(m) => m,
        }
    }
}

impl CrossSectionModel for ElectronModel {
    fn name(&self) -> &'static str {
        self.inner().name()
    }
    fn process(&self) -> ProcessKind {
        self.inner().process()
    }
    fn energy_range(&self) -> EnergyRange {
        self.inner().energy_range()
    }
    fn cross_section(&self, energy: Energy) -> Result<Area, PhysicsError> {
        self.inner().cross_section(energy)
    }
}

/// `EnergyRange` of a window given in eV (constants above are known to satisfy `lo <= hi`).
pub(crate) fn window(low_ev: f64, high_ev: f64) -> EnergyRange {
    EnergyRange::new(Energy::from_ev(low_ev), Energy::from_ev(high_ev))
        .expect("model window constants satisfy low <= high")
}

/// Check that a table covers the model window; Geant4 silently clamps otherwise.
pub(crate) fn ensure_table_covers(
    table: &'static str,
    model: &'static str,
    table_range: EnergyRange,
    window: EnergyRange,
) -> Result<(), PhysicsError> {
    if table_range.min() <= window.min() && window.max() <= table_range.max() {
        Ok(())
    } else {
        Err(PhysicsError::TableDoesNotCoverWindow {
            table,
            model,
            table_min_ev: table_range.min().ev(),
            table_max_ev: table_range.max().ev(),
            min_ev: window.min().ev(),
            max_ev: window.max().ev(),
        })
    }
}
