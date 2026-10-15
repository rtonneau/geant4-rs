//! Transport of an electron and its whole secondary cascade in infinite liquid water.
//!
//! Algorithm (docs/04 D.1):
//! 1. A history owns a LIFO stack of tracks. It starts with the primary. A track is followed until
//!    it dies (solvation, attachment) or a limit is reached; secondary electrons produced on the
//!    way are pushed on the stack in order of production and tracked afterwards, **last produced
//!    first** (Geant4's urgent stack). All tracks share the history's single physics stream, so
//!    the draw order is: the primary's whole life, then the last secondary's whole life, and so on
//!    (docs/reproducibility.md).
//! 2. Per step, energies at or below the solvation limit (7.4 eV, closed) are solvated at once:
//!    the process has a cross section of `DBL_MAX` in Geant4, an interaction length of ~0, and
//!    therefore wins against every finite cross section (`models/solvation.rs`). Otherwise
//!    `Sigma_i(E) = n sigma_i(E)` for the five processes at the *pre-step* energy (the DNA
//!    processes are discrete, have no lambda tables (`SetBuildTableFlag(false)`,
//!    `G4DNAElastic.cc:70`) and no integral approach (`fXSType = fEmNoIntegral`,
//!    `G4VEmProcess.hh:392`)).
//! 3. Free path `s = -ln(u)/Sigma_tot`, process `i` with probability `Sigma_i / Sigma_tot` in the
//!    order of [`ProcessKind::ALL`]. Geant4 instead draws one number of interaction lengths per
//!    process and takes the smallest distance; both are the same Poisson process (memoryless),
//!    so the distributions agree but the random streams differ.
//! 4. Final state from the model; ionisation secondaries are transported (default) or only
//!    recorded ([`TransportConfig::transport_secondaries`]`= false`, used for the Geant4 M2
//!    comparison where secondaries are killed).
//!
//! # Termination
//! * `Completed`: every track ended by solvation or attachment.
//! * `TrackingCut`: the *primary* fell below the optional `tracking_cut` (the cascade it already
//!   produced is still followed when secondaries are transported).
//! * `InteractionLimit`: the optional history-wide `max_interactions` was reached (first
//!   interaction mode of the Geant4 `mfp` example); all unfinished tracks are abandoned.
//! * Exceeding the safety limit `max_steps` is an **error** (`StepLimitExceeded`), never a
//!   silent stop. So is an energy at which no process exists (`NoProcessAvailable`).
//!
//! # Energy ledger (docs/01 A.4)
//! `E_in = deposited_excitation + deposited_vibrational + deposited_binding + deposited_attachment
//! + deposited_solvation + escaped + untracked_secondaries + remaining_untracked`, accumulated with
//! compensated (Neumaier) sums and checked at the end of every history against
//! `ledger_limit_relative` (default 1e-9, V7). `escaped` is zero in an infinite medium.

use crate::error::PhysicsError;
use crate::models::{FinalState, ProcessKind, Quirks, SANCHE_LEVELS, Secondary};
use crate::physchem::{PhysChemEvent, PhysChemKind};
use crate::sampling::uniform;
use crate::table::ProcessTable;
use dna_core::units::Energy;
use dna_core::vec3::Vec3;
use rand::Rng;
use serde::Serialize;

/// Why a history ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Termination {
    /// Every track ended by solvation or attachment.
    Completed,
    /// `max_interactions` reached (the Geant4 `mfp` example kills the event after the first
    /// interaction).
    InteractionLimit,
    /// The primary fell below `tracking_cut`.
    TrackingCut,
}

/// Transport options.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransportConfig {
    /// Stop the **primary** when its energy falls below this value.
    pub tracking_cut: Option<Energy>,
    /// Stop the history after this many interactions of all its tracks (`Some(1)`:
    /// first-interaction mean-free-path mode).
    pub max_interactions: Option<u64>,
    /// Safety limit on the interactions of one history; exceeding it is an error.
    pub max_steps: u64,
    /// Hard limit of the relative ledger residual at the end of a history.
    pub ledger_limit_relative: f64,
    /// Follow the ionisation secondaries (default). `false` records them and books their energy
    /// as `untracked_secondaries` (the M2 behaviour, and what the Geant4 `track` reference does).
    pub transport_secondaries: bool,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            tracking_cut: None,
            max_interactions: None,
            max_steps: 100_000_000,
            ledger_limit_relative: 1e-9,
            transport_secondaries: true,
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
    /// Id of the history, copied into every [`PhysChemEvent`].
    pub history_id: u64,
}

/// Energy balance of one history, eV.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ledger {
    pub e_in: f64,
    /// Electronic excitation energies deposited locally (Born).
    pub deposited_excitation: f64,
    /// Vibrational excitation energies deposited locally (Sanche).
    pub deposited_vibrational: f64,
    /// Shell binding energies deposited locally (ionisation).
    pub deposited_binding: f64,
    /// Kinetic energy of electrons absorbed by attachment (Melton).
    pub deposited_attachment: f64,
    /// Kinetic energy of electrons solvated below 7.4 eV.
    pub deposited_solvation: f64,
    /// Energy leaving the medium: zero in the infinite medium of M3.
    pub escaped: f64,
    /// Kinetic energy of secondaries that were recorded but not transported.
    pub untracked_secondaries: f64,
    /// Kinetic energy of tracks abandoned by a cut or limit (primary below the tracking cut, or
    /// tracks left when `max_interactions` was reached).
    pub remaining_untracked: f64,
}

impl Ledger {
    /// `E_in - (all parts)`, eV; zero for a conserved history.
    pub fn residual(&self) -> f64 {
        self.e_in
            - self.deposited()
            - self.escaped
            - self.untracked_secondaries
            - self.remaining_untracked
    }

    /// Residual relative to `E_in`.
    pub fn residual_relative(&self) -> f64 {
        self.residual() / self.e_in
    }

    /// Everything deposited locally, the Geant4 `E_dep` of docs/01 A.4.
    pub fn deposited(&self) -> f64 {
        self.deposited_excitation
            + self.deposited_vibrational
            + self.deposited_binding
            + self.deposited_attachment
            + self.deposited_solvation
    }
}

/// Interaction counts per cross-section-driven process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProcessCounts {
    counts: [u64; ProcessKind::COUNT],
}

impl ProcessCounts {
    pub fn get(&self, p: ProcessKind) -> u64 {
        self.counts[p.index()]
    }
    pub fn total(&self) -> u64 {
        self.counts.iter().sum()
    }
    fn bump(&mut self, p: ProcessKind) {
        self.counts[p.index()] += 1;
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
    /// Sanche: `E - E_vib <= 0`, nothing happens although the process was selected.
    pub vibrational_noop: u64,
    /// Solvations with `r_mean == 0` (energy <= 0.1 eV): the 1e-3 nm random displacement branch.
    pub solvation_zero_rmean: u64,
}

impl QuirkCounts {
    fn add(&mut self, q: &Quirks) {
        self.shell2_redraws += u64::from(q.shell2_redraws);
        self.excitation_noop += u64::from(q.excitation_noop);
        self.ionisation_noop_below_binding += u64::from(q.ionisation_noop_below_binding);
        self.transfer_clamped_to_zero += u64::from(q.transfer_clamped_to_zero);
        self.k_shell_deexcitation_skipped += u64::from(q.k_shell_deexcitation_skipped);
        self.vibrational_noop += u64::from(q.vibrational_noop);
    }

    /// Component-wise sum (used by the scorers).
    pub fn accumulate(&mut self, o: &QuirkCounts) {
        self.shell2_redraws += o.shell2_redraws;
        self.excitation_noop += o.excitation_noop;
        self.ionisation_noop_below_binding += o.ionisation_noop_below_binding;
        self.transfer_clamped_to_zero += o.transfer_clamped_to_zero;
        self.k_shell_deexcitation_skipped += o.k_shell_deexcitation_skipped;
        self.ionisation_without_secondary += o.ionisation_without_secondary;
        self.vibrational_noop += o.vibrational_noop;
        self.solvation_zero_rmean += o.solvation_zero_rmean;
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
    /// History-wide interaction counter (0-based).
    pub index: u64,
    pub process: ProcessKind,
    /// 0 for the primary, 1 for its secondaries, and so on.
    pub generation: u32,
    /// Free path travelled before the interaction, nm.
    pub free_path_nm: f64,
    /// Position of the interaction, nm.
    pub position: Vec3,
    pub energy_before: Energy,
    pub final_state: &'a FinalState,
}

/// One solvation, as seen by an observer.
#[derive(Debug, Clone, Copy)]
pub struct SolvationEvent {
    pub generation: u32,
    /// Where the electron thermalised (and deposited its kinetic energy), nm.
    pub origin: Vec3,
    /// Displacement of the solvated electron, nm.
    pub displacement_nm: Vec3,
    /// Kinetic energy deposited, eV.
    pub energy: Energy,
    pub r_mean_nm: f64,
}

/// Receiver of the physics events of a history (scorers, secondary recorders, tests).
pub trait HistoryObserver {
    fn on_interaction(&mut self, _event: &StepEvent<'_>) {}
    fn on_solvation(&mut self, _event: &SolvationEvent) {}
    /// Physicochemical events in order of occurrence (after the interaction that caused them).
    fn on_physchem(&mut self, _event: &PhysChemEvent) {}
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
    pub generation: u32,
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
                generation: event.generation + 1,
            });
        }
    }
}

/// Everything physics reports about one finished history.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistoryOutcome {
    pub ledger: Ledger,
    pub termination: Termination,
    /// Interactions of all tracks (solvations are not counted here, see `solvations`).
    pub steps: u64,
    pub counts: ProcessCounts,
    /// Solvation events (one per electron that reached 7.4 eV).
    pub solvations: u64,
    /// Per excitation level / ionisation shell / vibrational level.
    pub excitation_levels: [u64; 5],
    pub ionisation_shells: [u64; 5],
    pub vibrational_levels: [u64; SANCHE_LEVELS],
    /// Secondary electrons created.
    pub n_secondaries: u64,
    /// Tracks followed (1 + transported secondaries).
    pub n_tracks: u64,
    /// Total path length of the primary track, nm.
    pub track_length_nm: f64,
    /// Total path length of all followed tracks, nm.
    pub total_track_length_nm: f64,
    pub start_position: Vec3,
    pub start_direction: Vec3,
    /// Where the primary track ended.
    pub end_position: Vec3,
    pub first_step: Option<FirstStep>,
    pub quirks: QuirkCounts,
}

impl HistoryOutcome {
    /// Straight-line distance between start and end point of the primary, nm.
    pub fn range_nm(&self) -> f64 {
        (self.end_position - self.start_position).norm()
    }
    /// Displacement of the primary along the initial direction, nm.
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

/// A track waiting on the stack or being followed.
#[derive(Debug, Clone, Copy)]
struct Track {
    energy: Energy,
    direction: Vec3,
    position: Vec3,
    generation: u32,
}

/// Transport one electron and its cascade until the stack is empty. `rng` is the history's
/// physics stream.
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
    let mut dep_excitation = CompensatedSum::default();
    let mut dep_vibrational = CompensatedSum::default();
    let mut dep_binding = CompensatedSum::default();
    let mut dep_attachment = CompensatedSum::default();
    let mut dep_solvation = CompensatedSum::default();
    let mut untracked_secondaries = CompensatedSum::default();
    let mut remaining_untracked = CompensatedSum::default();
    let mut primary_length = CompensatedSum::default();
    let mut total_length = CompensatedSum::default();

    let mut counts = ProcessCounts::default();
    let mut solvations = 0u64;
    let mut excitation_levels = [0u64; 5];
    let mut ionisation_shells = [0u64; 5];
    let mut vibrational_levels = [0u64; SANCHE_LEVELS];
    let mut n_secondaries = 0u64;
    let mut n_tracks = 0u64;
    let mut quirks = QuirkCounts::default();
    let mut first_step = None;
    let mut steps = 0u64;
    let mut end_position = primary.position;
    let mut termination = Termination::Completed;

    let mut stack: Vec<Track> = Vec::with_capacity(64);
    stack.push(Track {
        energy: primary.energy,
        direction: start_direction,
        position: primary.position,
        generation: 0,
    });

    'history: while let Some(mut track) = stack.pop() {
        n_tracks += 1;
        'track: loop {
            if config.max_interactions.is_some_and(|m| steps >= m) {
                remaining_untracked.add(track.energy.ev());
                for t in stack.drain(..) {
                    remaining_untracked.add(t.energy.ev());
                }
                termination = Termination::InteractionLimit;
                break 'history;
            }
            if track.generation == 0
                && let Some(cut) = config.tracking_cut
                && track.energy < cut
            {
                remaining_untracked.add(track.energy.ev());
                if termination == Termination::Completed {
                    termination = Termination::TrackingCut;
                }
                break 'track;
            }

            // Solvation: interaction length ~0, always wins (see module docs).
            if let Some(solvation) = table.solvation()
                && solvation.applies(track.energy)
            {
                let s = solvation.sample(track.energy, rng)?;
                dep_solvation.add(s.deposit.ev());
                solvations += 1;
                if s.zero_rmean_branch {
                    quirks.solvation_zero_rmean += 1;
                }
                observer.on_solvation(&SolvationEvent {
                    generation: track.generation,
                    origin: track.position,
                    displacement_nm: s.displacement_nm,
                    energy: s.deposit,
                    r_mean_nm: s.r_mean_nm,
                });
                observer.on_physchem(&PhysChemEvent {
                    kind: PhysChemKind::Solvation,
                    level_or_shell: 0,
                    position: track.position + s.displacement_nm,
                    deposited_energy: s.deposit,
                    parent_history: primary.history_id,
                });
                break 'track;
            }

            let xs = table.macroscopic(track.energy)?;
            let total = xs.total();
            if total <= 0.0 {
                return Err(PhysicsError::NoProcessAvailable {
                    energy_ev: track.energy.ev(),
                });
            }
            if steps >= config.max_steps {
                return Err(PhysicsError::StepLimitExceeded {
                    max_steps: config.max_steps,
                });
            }

            let free_path = -uniform(rng).ln() / total;
            track.position = track.position + track.direction * free_path;
            total_length.add(free_path);
            if track.generation == 0 {
                primary_length.add(free_path);
                end_position = track.position;
            }

            let process = select_process(&xs, uniform(rng) * total);
            let model = table
                .select(process, track.energy)
                .expect("cross section > 0 implies a serving model");
            let fs = model.sample(track.energy, track.direction, rng)?;

            counts.bump(process);
            quirks.add(&fs.quirks);
            match process {
                ProcessKind::Elastic => debug_assert_eq!(fs.local_deposit.ev(), 0.0),
                ProcessKind::Excitation => {
                    excitation_levels[fs.channel] += 1;
                    dep_excitation.add(fs.local_deposit.ev());
                }
                ProcessKind::Ionisation => {
                    ionisation_shells[fs.channel] += 1;
                    dep_binding.add(fs.local_deposit.ev());
                    if fs.secondary.is_none() && !fs.quirks.ionisation_noop_below_binding {
                        quirks.ionisation_without_secondary += 1;
                    }
                }
                ProcessKind::VibrationalExcitation => {
                    vibrational_levels[fs.channel] += 1;
                    dep_vibrational.add(fs.local_deposit.ev());
                }
                ProcessKind::Attachment => dep_attachment.add(fs.local_deposit.ev()),
            }

            observer.on_interaction(&StepEvent {
                index: steps,
                process,
                generation: track.generation,
                free_path_nm: free_path,
                position: track.position,
                energy_before: track.energy,
                final_state: &fs,
            });
            match process {
                ProcessKind::Excitation => observer.on_physchem(&PhysChemEvent {
                    kind: PhysChemKind::Excitation,
                    level_or_shell: fs.channel as u8,
                    position: track.position,
                    deposited_energy: fs.local_deposit,
                    parent_history: primary.history_id,
                }),
                ProcessKind::Ionisation if !fs.quirks.ionisation_noop_below_binding => observer
                    .on_physchem(&PhysChemEvent {
                        kind: PhysChemKind::Ionisation,
                        level_or_shell: fs.channel as u8,
                        position: track.position,
                        deposited_energy: fs.local_deposit,
                        parent_history: primary.history_id,
                    }),
                ProcessKind::Attachment => observer.on_physchem(&PhysChemEvent {
                    kind: PhysChemKind::Attachment,
                    level_or_shell: 0,
                    position: track.position,
                    deposited_energy: fs.local_deposit,
                    parent_history: primary.history_id,
                }),
                _ => {}
            }
            if first_step.is_none() {
                first_step = Some(FirstStep {
                    length_nm: free_path,
                    process,
                });
            }
            steps += 1;

            if let Some(s) = fs.secondary {
                n_secondaries += 1;
                if config.transport_secondaries {
                    stack.push(Track {
                        energy: s.energy,
                        direction: s.direction,
                        position: track.position,
                        generation: track.generation + 1,
                    });
                } else {
                    untracked_secondaries.add(s.energy.ev());
                }
            }
            if fs.killed {
                break 'track;
            }
            track.energy = fs.energy_after;
            track.direction = fs.direction_after;
        }

        #[cfg(debug_assertions)]
        {
            let in_flight: f64 = stack.iter().map(|t| t.energy.ev()).sum();
            let residual = e_in
                - dep_excitation.value()
                - dep_vibrational.value()
                - dep_binding.value()
                - dep_attachment.value()
                - dep_solvation.value()
                - untracked_secondaries.value()
                - remaining_untracked.value()
                - in_flight;
            debug_assert!(
                residual.abs() <= 1e-9 * e_in,
                "ledger residual {residual} eV after a track (E_in {e_in} eV)"
            );
        }
    }

    let ledger = Ledger {
        e_in,
        deposited_excitation: dep_excitation.value(),
        deposited_vibrational: dep_vibrational.value(),
        deposited_binding: dep_binding.value(),
        deposited_attachment: dep_attachment.value(),
        deposited_solvation: dep_solvation.value(),
        escaped: 0.0,
        untracked_secondaries: untracked_secondaries.value(),
        remaining_untracked: remaining_untracked.value(),
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
        solvations,
        excitation_levels,
        ionisation_shells,
        vibrational_levels,
        n_secondaries,
        n_tracks,
        track_length_nm: primary_length.value(),
        total_track_length_nm: total_length.value(),
        start_position: primary.position,
        start_direction,
        end_position,
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
        MacroscopicCrossSections::new([a, b, c, 0.0, 0.0])
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
            deposited_vibrational: 0.5,
            deposited_binding: 10.79,
            deposited_attachment: 1.0,
            deposited_solvation: 2.0,
            escaped: 0.0,
            untracked_secondaries: 30.0,
            remaining_untracked: 47.49,
        };
        assert!(l.residual().abs() < 1e-12);
        assert!((l.deposited() - 22.51).abs() < 1e-12);
    }
}
