//! Transport of one electron in infinite liquid water.
//!
//! Algorithm (docs/04 D.1, restricted to the M2 processes):
//! 1. `Sigma_i(E) = n sigma_i(E)` for the three processes at the *pre-step* energy; DNA processes
//!    are discrete, have no lambda tables (`SetBuildTableFlag(false)`,
//!    `G4DNAElastic.cc:70`) and no integral approach (`fXSType = fEmNoIntegral`,
//!    `G4VEmProcess.hh:392`), so `Sigma` is exact at the pre-step energy.
//! 2. Free path `s = -ln(u)/Sigma_tot`; the particle moves along its direction (no boundaries).
//! 3. Process `i` with probability `Sigma_i / Sigma_tot`, in the fixed order elastic,
//!    excitation, ionisation. Geant4 instead draws one number of interaction lengths per process
//!    and takes the smallest distance; both are the same Poisson process (memoryless), so the
//!    distributions agree but the random streams differ (docs/reproducibility.md).
//! 4. Final state from the model; secondaries are *recorded* (energy, direction, position) through
//!    the observer and **not transported**.
//!
//! # Termination (decision: M2 stops where M3 will take over)
//! * `InteractionLimit`: optional cap on the number of interactions (first-interaction mode).
//! * `TrackingCut`: energy below the optional `tracking_cut` (used to compare with Geant4 runs
//!   that stop the primary at a given energy).
//! * `BelowLowestWindow`: no model serves the energy (below 7.4 eV): solvation (M3).
//! * `NoInelasticChannel`: only elastic scattering is available (7.4-9 eV; excitation starts at
//!   9 eV, ionisation at 11 eV). Elastic scattering never changes the energy, so continuing would
//!   never terminate; in Geant4 Sanche vibrational excitation and Melton attachment
//!   (M3) remove the electron here. **This guard is an M2 deviation**, see docs/09 Q14.
//!
//! In all cases the primary's kinetic energy at termination is booked as
//! `remaining_untracked` (to be replaced by solvation in M3).
//!
//! # Energy ledger (docs/01 A.4)
//! `E_in = deposited_excitation + deposited_binding + secondaries + remaining_untracked`,
//! accumulated with compensated (Neumaier) sums, checked in debug builds at every step and, in all
//! builds, at the end of the history against `ledger_limit_relative` (default 1e-9, V7).

use crate::error::PhysicsError;
use crate::models::{FinalState, ProcessKind, Quirks, Secondary};
use crate::sampling::uniform;
use crate::table::ProcessTable;
use dna_core::units::Energy;
use dna_core::vec3::Vec3;
use rand::Rng;
use serde::Serialize;

/// Why a history ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Termination {
    /// `max_interactions` reached (the Geant4 `mfp` example kills the event after the first
    /// interaction).
    InteractionLimit,
    TrackingCut,
    BelowLowestWindow,
    NoInelasticChannel,
}

/// Transport options.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransportConfig {
    /// Stop the primary when its energy falls below this value.
    pub tracking_cut: Option<Energy>,
    /// Stop after this many interactions (`Some(1)`: first-interaction mean-free-path mode).
    pub max_interactions: Option<u64>,
    /// Guard against runaway histories.
    pub max_steps: u64,
    /// Hard limit of the relative ledger residual at the end of a history.
    pub ledger_limit_relative: f64,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            tracking_cut: None,
            max_interactions: None,
            max_steps: 100_000_000,
            ledger_limit_relative: 1e-9,
        }
    }
}

/// Initial state of the primary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Primary {
    pub energy: Energy,
    /// Start position, nm.
    pub position: Vec3,
    /// Direction; normalised internally.
    pub direction: Vec3,
}

/// Energy balance of one history, eV.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ledger {
    pub e_in: f64,
    /// Sum of excitation energies deposited locally.
    pub deposited_excitation: f64,
    /// Sum of shell binding energies deposited locally.
    pub deposited_binding: f64,
    /// Kinetic energy of the recorded (untransported) secondaries.
    pub secondaries: f64,
    /// Kinetic energy of the primary when it was stopped (solvation/attachment in M3).
    pub remaining_untracked: f64,
}

impl Ledger {
    /// `E_in - (all parts)`, eV; zero for a conserved history.
    pub fn residual(&self) -> f64 {
        self.e_in
            - self.deposited_excitation
            - self.deposited_binding
            - self.secondaries
            - self.remaining_untracked
    }

    /// Residual relative to `E_in`.
    pub fn residual_relative(&self) -> f64 {
        self.residual() / self.e_in
    }

    /// Everything deposited locally (excitation + binding), the Geant4 `E_dep` of docs/01 A.4
    /// for the primary track alone.
    pub fn deposited(&self) -> f64 {
        self.deposited_excitation + self.deposited_binding
    }
}

/// Interaction counts per process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProcessCounts {
    pub elastic: u64,
    pub excitation: u64,
    pub ionisation: u64,
}

impl ProcessCounts {
    pub fn get(&self, p: ProcessKind) -> u64 {
        match p {
            ProcessKind::Elastic => self.elastic,
            ProcessKind::Excitation => self.excitation,
            ProcessKind::Ionisation => self.ionisation,
        }
    }
    pub fn total(&self) -> u64 {
        self.elastic + self.excitation + self.ionisation
    }
    fn bump(&mut self, p: ProcessKind) {
        match p {
            ProcessKind::Elastic => self.elastic += 1,
            ProcessKind::Excitation => self.excitation += 1,
            ProcessKind::Ionisation => self.ionisation += 1,
        }
    }
}

/// Occurrences of the reproduced Geant4 quirks (see [`Quirks`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct QuirkCounts {
    pub shell2_redraws: u64,
    pub excitation_noop: u64,
    pub ionisation_noop_below_binding: u64,
    pub transfer_clamped_to_zero: u64,
    pub k_shell_deexcitation_skipped: u64,
    /// Ionisations whose ejected electron has zero energy (no secondary is created).
    pub ionisation_without_secondary: u64,
}

impl QuirkCounts {
    fn add(&mut self, q: &Quirks) {
        self.shell2_redraws += u64::from(q.shell2_redraws);
        self.excitation_noop += u64::from(q.excitation_noop);
        self.ionisation_noop_below_binding += u64::from(q.ionisation_noop_below_binding);
        self.transfer_clamped_to_zero += u64::from(q.transfer_clamped_to_zero);
        self.k_shell_deexcitation_skipped += u64::from(q.k_shell_deexcitation_skipped);
    }

    /// Component-wise sum (used by the scorers).
    pub fn accumulate(&mut self, o: &QuirkCounts) {
        self.shell2_redraws += o.shell2_redraws;
        self.excitation_noop += o.excitation_noop;
        self.ionisation_noop_below_binding += o.ionisation_noop_below_binding;
        self.transfer_clamped_to_zero += o.transfer_clamped_to_zero;
        self.k_shell_deexcitation_skipped += o.k_shell_deexcitation_skipped;
        self.ionisation_without_secondary += o.ionisation_without_secondary;
    }
}

/// First step of the history, for mean-free-path scoring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FirstStep {
    pub length_nm: f64,
    pub process: ProcessKind,
}

/// One interaction, as seen by an observer.
#[derive(Debug, Clone, Copy)]
pub struct StepEvent<'a> {
    pub index: u64,
    pub process: ProcessKind,
    /// Free path travelled before the interaction, nm.
    pub free_path_nm: f64,
    /// Position of the interaction, nm.
    pub position: Vec3,
    pub energy_before: Energy,
    pub final_state: &'a FinalState,
}

/// Receiver of the physics events of a history (scorers, secondary recorders, tests).
pub trait HistoryObserver {
    fn on_interaction(&mut self, _event: &StepEvent<'_>) {}
}

/// Observer that ignores everything.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoObserver;
impl HistoryObserver for NoObserver {}

/// A recorded secondary: where and with what it was ejected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecordedSecondary {
    pub energy: Energy,
    pub direction: Vec3,
    pub position: Vec3,
}

/// Observer that stores every secondary (energy, direction, position) of the history.
#[derive(Debug, Default, Clone)]
pub struct SecondaryRecorder {
    pub secondaries: Vec<RecordedSecondary>,
}

impl HistoryObserver for SecondaryRecorder {
    fn on_interaction(&mut self, event: &StepEvent<'_>) {
        if let Some(Secondary { energy, direction }) = event.final_state.secondary {
            self.secondaries.push(RecordedSecondary {
                energy,
                direction,
                position: event.position,
            });
        }
    }
}

/// Everything physics reports about one finished history.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistoryOutcome {
    pub ledger: Ledger,
    pub termination: Termination,
    pub steps: u64,
    pub counts: ProcessCounts,
    /// Per excitation level / ionisation shell.
    pub excitation_levels: [u64; 5],
    pub ionisation_shells: [u64; 5],
    pub n_secondaries: u64,
    /// Total path length, nm.
    pub track_length_nm: f64,
    pub start_position: Vec3,
    pub start_direction: Vec3,
    pub end_position: Vec3,
    pub first_step: Option<FirstStep>,
    pub quirks: QuirkCounts,
}

impl HistoryOutcome {
    /// Straight-line distance between start and end point, nm.
    pub fn range_nm(&self) -> f64 {
        (self.end_position - self.start_position).norm()
    }
    /// Displacement along the initial direction, nm.
    pub fn projected_range_nm(&self) -> f64 {
        (self.end_position - self.start_position).dot(self.start_direction)
    }
}

/// Neumaier compensated sum.
#[derive(Debug, Clone, Copy, Default)]
struct CompensatedSum {
    sum: f64,
    c: f64,
}

impl CompensatedSum {
    fn add(&mut self, x: f64) {
        let t = self.sum + x;
        if self.sum.abs() >= x.abs() {
            self.c += (self.sum - t) + x;
        } else {
            self.c += (x - t) + self.sum;
        }
        self.sum = t;
    }
    fn value(&self) -> f64 {
        self.sum + self.c
    }
}

/// Transport one electron until termination. `rng` is the history's physics stream.
pub fn transport_electron<R: Rng + ?Sized, O: HistoryObserver + ?Sized>(
    table: &ProcessTable,
    config: &TransportConfig,
    primary: Primary,
    rng: &mut R,
    observer: &mut O,
) -> Result<HistoryOutcome, PhysicsError> {
    table.primary_energy_range().check(primary.energy)?;
    let start_direction = primary
        .direction
        .normalised()
        .ok_or(PhysicsError::InvalidDirection)?;

    let e_in = primary.energy.ev();
    let mut energy = primary.energy;
    let mut direction = start_direction;
    let mut position = primary.position;

    let mut deposited_excitation = CompensatedSum::default();
    let mut deposited_binding = CompensatedSum::default();
    let mut secondaries = CompensatedSum::default();
    let mut track_length = CompensatedSum::default();

    let mut counts = ProcessCounts::default();
    let mut excitation_levels = [0u64; 5];
    let mut ionisation_shells = [0u64; 5];
    let mut n_secondaries = 0u64;
    let mut quirks = QuirkCounts::default();
    let mut first_step = None;
    let mut steps = 0u64;

    let termination = loop {
        if config.max_interactions.is_some_and(|m| steps >= m) {
            break Termination::InteractionLimit;
        }
        if let Some(cut) = config.tracking_cut
            && energy < cut
        {
            break Termination::TrackingCut;
        }
        let xs = table.macroscopic(energy)?;
        let total = xs.total();
        if total <= 0.0 {
            break Termination::BelowLowestWindow;
        }
        if xs.inelastic() <= 0.0 {
            break Termination::NoInelasticChannel;
        }
        if steps >= config.max_steps {
            return Err(PhysicsError::StepLimitExceeded {
                max_steps: config.max_steps,
            });
        }

        let free_path = -uniform(rng).ln() / total;
        position = position + direction * free_path;
        track_length.add(free_path);

        let process = select_process(&xs, uniform(rng) * total);
        let model = table
            .select(process, energy)
            .expect("cross section > 0 implies a serving model");
        let fs = model.sample(energy, direction, rng)?;

        counts.bump(process);
        match process {
            ProcessKind::Excitation => excitation_levels[fs.channel] += 1,
            ProcessKind::Ionisation => ionisation_shells[fs.channel] += 1,
            ProcessKind::Elastic => {}
        }
        quirks.add(&fs.quirks);
        match process {
            ProcessKind::Excitation => deposited_excitation.add(fs.local_deposit.ev()),
            ProcessKind::Ionisation => {
                deposited_binding.add(fs.local_deposit.ev());
                if fs.secondary.is_none() && !fs.quirks.ionisation_noop_below_binding {
                    quirks.ionisation_without_secondary += 1;
                }
            }
            ProcessKind::Elastic => debug_assert_eq!(fs.local_deposit.ev(), 0.0),
        }
        if let Some(s) = fs.secondary {
            secondaries.add(s.energy.ev());
            n_secondaries += 1;
        }

        observer.on_interaction(&StepEvent {
            index: steps,
            process,
            free_path_nm: free_path,
            position,
            energy_before: energy,
            final_state: &fs,
        });
        if first_step.is_none() {
            first_step = Some(FirstStep {
                length_nm: free_path,
                process,
            });
        }

        energy = fs.energy_after;
        direction = fs.direction_after;
        steps += 1;

        #[cfg(debug_assertions)]
        {
            let residual = e_in
                - deposited_excitation.value()
                - deposited_binding.value()
                - secondaries.value()
                - energy.ev();
            debug_assert!(
                residual.abs() <= 1e-9 * e_in,
                "ledger residual {residual} eV at step {steps} (E_in {e_in} eV)"
            );
            debug_assert!(
                (direction.norm() - 1.0).abs() < 1e-9,
                "direction left the unit sphere: {direction:?}"
            );
        }
    };

    let ledger = Ledger {
        e_in,
        deposited_excitation: deposited_excitation.value(),
        deposited_binding: deposited_binding.value(),
        secondaries: secondaries.value(),
        remaining_untracked: energy.ev(),
    };
    if ledger.residual().abs() > config.ledger_limit_relative * e_in {
        return Err(PhysicsError::LedgerViolation {
            residual_ev: ledger.residual(),
            e_in_ev: e_in,
            limit_relative: config.ledger_limit_relative,
        });
    }

    Ok(HistoryOutcome {
        ledger,
        termination,
        steps,
        counts,
        excitation_levels,
        ionisation_shells,
        n_secondaries,
        track_length_nm: track_length.value(),
        start_position: primary.position,
        start_direction,
        end_position: position,
        first_step,
        quirks,
    })
}

/// Process owning the cumulative position `r` in `[0, total)`; falls back to the last process
/// with a positive cross section if rounding pushes `r` to the end.
fn select_process(xs: &crate::table::MacroscopicCrossSections, r: f64) -> ProcessKind {
    let mut cumulative = 0.0;
    let mut last_positive = ProcessKind::Elastic;
    for p in ProcessKind::ALL {
        let s = xs.get(p);
        if s > 0.0 {
            cumulative += s;
            last_positive = p;
            if r < cumulative {
                return p;
            }
        }
    }
    last_positive
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::MacroscopicCrossSections;

    fn xs(a: f64, b: f64, c: f64) -> MacroscopicCrossSections {
        MacroscopicCrossSections::new([a, b, c])
    }

    #[test]
    fn process_selection_partitions_the_total() {
        let x = xs(1.0, 2.0, 3.0);
        assert_eq!(select_process(&x, 0.0), ProcessKind::Elastic);
        assert_eq!(select_process(&x, 0.999), ProcessKind::Elastic);
        assert_eq!(select_process(&x, 1.0), ProcessKind::Excitation);
        assert_eq!(select_process(&x, 2.999), ProcessKind::Excitation);
        assert_eq!(select_process(&x, 3.0), ProcessKind::Ionisation);
        assert_eq!(select_process(&x, 6.0), ProcessKind::Ionisation);
    }

    #[test]
    fn zero_cross_sections_are_skipped() {
        let x = xs(0.0, 0.0, 4.0);
        assert_eq!(select_process(&x, 0.0), ProcessKind::Ionisation);
        let x = xs(5.0, 0.0, 0.0);
        assert_eq!(select_process(&x, 4.0), ProcessKind::Elastic);
        assert_eq!(select_process(&x, 5.0), ProcessKind::Elastic);
    }

    #[test]
    fn compensated_sum_beats_naive_summation() {
        let mut s = CompensatedSum::default();
        let mut naive = 0.0f64;
        s.add(1.0e16);
        naive += 1.0e16;
        for _ in 0..1000 {
            s.add(1.0);
            naive += 1.0;
        }
        s.add(-1.0e16);
        naive += -1.0e16;
        assert_eq!(s.value(), 1000.0);
        assert_ne!(naive, 1000.0);
    }

    #[test]
    fn ledger_residual_and_deposit() {
        let l = Ledger {
            e_in: 100.0,
            deposited_excitation: 8.22,
            deposited_binding: 10.79,
            secondaries: 30.0,
            remaining_untracked: 50.99,
        };
        assert!(l.residual().abs() < 1e-12);
        assert!((l.deposited() - 19.01).abs() < 1e-12);
    }
}
