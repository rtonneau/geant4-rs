//! Serial chemical histories with competing events and stable molecule identities.
//!
//! Source baseline: Geant4 **11.4.1**, `source/processes/electromagnetic/dna/`
//! `models/src/G4DNAIRT.cc`: `IRTSampling`/`Sampling` (161-244) initialize
//! pairs once and add global time to elapsed clocks; `MakeReaction` (393-508)
//! materializes conditional parent sites and source-ordered products;
//! `FindReaction` (513-549) processes events through the current global time.
//! `management/include/G4ITReaction.hh`: `CanAddThisReaction` (223-241)
//! deduplicates pairs; `SelectThisReaction` (269-274) removes all events involving
//! either parent. Here alive/generation checks lazily discard invalid events,
//! preserving unaffected clocks without resampling. Unlike the track-hash tie
//! comparator in `management/src/G4ITReaction.cc` (45-55), the approved Rust
//! design deliberately resolves exact ties by canonical insertion order.
//! The existing [`crate::irt`] and [`crate::placement`] kernels retain their
//! documented source/numerical limitations. The 11.4.3 G1-F evidence does not
//! migrate these scheduler semantics from their 11.4.1 baseline.
//!
//! Products receive monotonic identities in source order and immediately sample
//! clocks against older live molecules and their unary channels. Spatial search
//! and complete-network scientific scores are not implemented here. No claim of
//! R05/G2 acceptance is made.
//!
//! Molecules keep creation-order identities, generation and last materialized
//! time. Survivors are never diffused merely to advance the clock. The input
//! snapshot remains immutable. Events order by absolute time, then insertion
//! ordinal; candidates order by ascending input IDs, independent of descriptor
//! order. Source reactant order is retained separately for conditional placement.
//! Unlike upstream's strict `irt < timeMax - globalTime`, the final boundary is
//! inclusive, as required by the approved R05 design.
//!
//! `Sampling` (212-231) temporarily diffuses an older partner to the newly
//! created molecule's time **only for clock sampling**. Neither its stored site
//! nor its existing clocks change. `MakeReaction` uses the stored sites and the
//! delay since that sampling time, not the temporary Brownian site. We retain
//! source-table parent ordering for the asymmetric placement kernel; upstream
//! instead uses scheduling-track order. This deliberate deterministic ordering
//! can change product positions when the new product is source reactant B.
//! Unary channels (250-281) compete through their earliest independent clock.
//! Unlike upstream, an earlier pair clock does not suppress that unary clock:
//! it remains available if a competing reaction consumes only the pair partner.
//! Only consumption of the unary mobile parent invalidates it. Bulk partners
//! never receive mobile identities. Per-event accounting includes consumed
//! reservoirs, implicit water and only the exact reaction-53 source exception.
//!
//! [`run_history`] returns count observations only after reaching the configured
//! horizon. Pre-IRT counts and the right-continuous post-contact 1 ps counts are
//! separate outputs. Observations never diffuse survivors or draw randomness;
//! only the intervening reactions do. Duplicate observation times are allowed.
//! Traces are disabled by default and have an independent finite budget when
//! enabled. A failed run returns diagnostics, never partial successful counts.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::diffusion::diffuse;
use crate::irt::{ReactionTime, sample_independent_reaction_time};
use crate::molecule::{Molecule, MoleculeId, MoleculePair};
use crate::placement::{ReactionPlacement, place_pair_reaction, place_unary_reaction};
use crate::reaction_network::{CompiledReaction, ReactionNetwork};
use crate::reactions::{BulkParticipant, ReactionParticipant};
use crate::species::{SpeciesTable, atoms};
use crate::unary::sample_unary_reaction_time;
use crate::{ChemistryError, Species, SpeciesInstance};
use g4_core::units::Time;
use rand::Rng;

const START_PS: f64 = 1.0;
const MAX_END_PS: f64 = 1_000_000.0;

/// Validated, immutable physicochemical snapshot at the common 1 ps boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryInput {
    snapshot: Vec<SpeciesInstance>,
}

impl HistoryInput {
    /// Validate every time and position before copying records in creation order.
    /// No tolerance or synchronization is applied: each time must be exactly 1 ps.
    pub fn new(records: &[SpeciesInstance]) -> Result<Self, ChemistryError> {
        for (index, record) in records.iter().enumerate() {
            validate_time("input time", record.time.ps(), START_PS, START_PS)?;
            for (field, value) in [
                ("position.x", record.position.x),
                ("position.y", record.position.y),
                ("position.z", record.position.z),
            ] {
                if !value.is_finite() {
                    return Err(ChemistryError::InvalidIrtInput {
                        id: MoleculeId(index as u64),
                        field,
                        value,
                    });
                }
            }
        }
        Ok(Self {
            snapshot: records.to_vec(),
        })
    }

    /// Pre-IRT records; this is distinct from any post-contact state.
    pub fn snapshot(&self) -> &[SpeciesInstance] {
        &self.snapshot
    }
}

/// Finite horizon and storage budgets for pair and unary histories.
#[derive(Debug, Clone, Copy)]
pub struct PairHistoryOptions {
    /// Inclusive absolute final time, from 1 ps through 1 us.
    pub end_time: Time,
    /// Maximum arena records, including dead records retained for identity.
    pub max_molecules: usize,
    /// Maximum simultaneous queued events, including stale entries awaiting removal.
    pub max_pending_events: usize,
    /// Maximum executed reactions, independent of optional trace retention.
    pub max_reactions: usize,
}

impl Default for PairHistoryOptions {
    fn default() -> Self {
        Self {
            end_time: Time::from_ps(MAX_END_PS),
            max_molecules: 100_000,
            max_pending_events: 100_000,
            max_reactions: 100_000,
        }
    }
}

/// Alive mobile populations in [`Species::ALL`] order, indexed by [`Species::index`].
/// Bulk reservoirs and implicit solvent water are not mobile populations.
pub type SpeciesCounts = [usize; Species::ALL.len()];

/// A right-continuous count sample after every event at or before `time`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CountObservation {
    /// Absolute time in ps; survivors need not be materialized at this time.
    pub time: Time,
    /// Alive mobile populations; no position diagnostic or G-value normalization.
    pub counts: SpeciesCounts,
}

/// Finite execution and output budgets for [`run_history`].
#[derive(Debug, Clone, Copy)]
pub struct HistoryOptions {
    /// Inclusive horizon, retained arena size, pending-event and reaction limits.
    pub limits: PairHistoryOptions,
    /// Maximum requested count samples, including duplicate times.
    pub max_observations: usize,
    /// `None` disables lineage retention. `Some(n)` retains at most n reactions;
    /// exceeding this budget fails the entire run instead of truncating the trace.
    pub max_trace_records: Option<usize>,
}

impl Default for HistoryOptions {
    fn default() -> Self {
        Self {
            limits: PairHistoryOptions::default(),
            max_observations: 1_024,
            max_trace_records: None,
        }
    }
}

/// Aggregate execution diagnostics, also available on incomplete runs.
/// These describe work performed, not a successful scientific count result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistoryDiagnostics {
    /// Last reached scheduler time; may precede the requested horizon on failure.
    pub time: Time,
    /// Reactions whose parent consumption and product creation completed.
    /// A failure while scheduling those products can leave this nonzero.
    pub executed_reactions: usize,
    /// Stored identities, including consumed parents.
    pub arena_records: usize,
    /// Queued entries, including stale events awaiting removal.
    pub pending_events: usize,
    /// Maximum simultaneous queued entries successfully retained.
    pub peak_pending_events: usize,
    /// Retained detailed records; zero when tracing is disabled.
    pub trace_records: usize,
}

impl HistoryDiagnostics {
    fn unstarted() -> Self {
        Self {
            time: Time::from_ps(START_PS),
            executed_reactions: 0,
            arena_records: 0,
            pending_events: 0,
            peak_pending_events: 0,
            trace_records: 0,
        }
    }

    fn incomplete(self, cause: ChemistryError) -> ChemistryError {
        ChemistryError::IncompleteHistory {
            diagnostics: self,
            cause: Box::new(cause),
        }
    }
}

/// Counts and diagnostics from a history completed through its inclusive horizon.
/// Successful execution is not evidence of R05/G2 acceptance or physical accuracy.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryResult {
    /// Immutable input population at 1 ps, before any IRT contact reaction.
    pub pre_irt_counts: SpeciesCounts,
    /// Population after all 1 ps contact reactions and their product cascades.
    pub post_contact_counts: SpeciesCounts,
    /// Requested samples in caller order, including duplicates.
    pub observations: Vec<CountObservation>,
    /// Population at the configured horizon, even if it was not requested as a sample.
    pub final_counts: SpeciesCounts,
    /// Work performed through the completed horizon.
    pub diagnostics: HistoryDiagnostics,
    /// Optional complete lineage, bounded by `max_trace_records`.
    pub trace: Option<Vec<HistoryReaction>>,
}

/// Evolve an immutable compiled network and 1 ps input using the caller's RNG.
///
/// Observation times must be nondecreasing and inside `[1 ps, end_time]`.
/// The entire schedule is validated before any RNG draw or history allocation.
/// Empty schedules still evolve to the horizon. Observing a boundary processes
/// all reactions at that time, including newly created immediate reactions.
///
/// Input-time errors are returned directly. Execution and resource failures
/// return [`ChemistryError::IncompleteHistory`] with diagnostics and the original
/// typed cause, without partial counts. RNG state is not rolled back on failure.
/// Storage limits count records, not allocator bytes; the arena retains dead IDs.
/// This uses the existing all-pairs scheduler, not yet a validated spatial index.
pub fn run_history(
    input: &HistoryInput,
    network: &ReactionNetwork,
    observation_times: &[Time],
    options: HistoryOptions,
    rng: &mut impl Rng,
) -> Result<HistoryResult, ChemistryError> {
    validate_time(
        "end time",
        options.limits.end_time.ps(),
        START_PS,
        MAX_END_PS,
    )?;
    let mut previous = START_PS;
    for &time in observation_times {
        validate_time(
            "observation time",
            time.ps(),
            previous,
            options.limits.end_time.ps(),
        )?;
        previous = time.ps();
    }
    capacity(
        "observations",
        observation_times.len(),
        options.max_observations,
    )
    .map_err(|cause| HistoryDiagnostics::unstarted().incomplete(cause))?;
    let descriptors: Vec<_> = network.iter().collect();
    let mut history = PairHistory::prepare(
        input,
        &descriptors,
        options.limits,
        options.max_trace_records,
    )
    .map_err(|cause| HistoryDiagnostics::unstarted().incomplete(cause))?;
    history
        .initialize(rng)
        .map_err(|cause| history.diagnostics().incomplete(cause))?;
    let pre_irt_counts = count_species(input.snapshot().iter().map(|m| m.species));
    let post_contact_counts = history
        .observe_counts(Time::from_ps(START_PS), rng)
        .map_err(|cause| history.diagnostics().incomplete(cause))?
        .counts;
    let mut observations = Vec::with_capacity(observation_times.len());
    for &time in observation_times {
        let observation = history
            .observe_counts(time, rng)
            .map_err(|cause| history.diagnostics().incomplete(cause))?;
        observations.push(observation);
    }
    let final_counts = history
        .observe_counts(options.limits.end_time, rng)
        .map_err(|cause| history.diagnostics().incomplete(cause))?
        .counts;
    Ok(HistoryResult {
        pre_irt_counts,
        post_contact_counts,
        observations,
        final_counts,
        diagnostics: history.diagnostics(),
        trace: options.max_trace_records.map(|_| history.reactions),
    })
}

/// Arena record; identity is never reused and fields cannot be mutated by callers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistoryMolecule {
    molecule: Molecule,
    generation: u64,
    alive: bool,
}

impl HistoryMolecule {
    /// Monotonic identity, assigned in input creation order starting at zero.
    pub fn id(&self) -> MoleculeId {
        self.molecule.id()
    }

    /// Current species, position and last materialized absolute time.
    pub fn instance(&self) -> &SpeciesInstance {
        self.molecule.instance()
    }

    /// Version used by scheduled events; consumption increments it once.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Whether this input molecule or product has not been consumed.
    pub fn is_alive(&self) -> bool {
        self.alive
    }
}

/// Integer atom/charge accounting for one executed event.
///
/// Components are hydrogen atoms, oxygen atoms and charge in units of e.
/// `mobile_change + implicit_water * [2, 1, 0] - bulk_composition`
/// equals `source_exception`. Each event is checked before consuming parents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReactionAccounting {
    /// Actual products minus consumed mobile parents.
    pub mobile_change: [i64; 3],
    /// One consumed reservoir unit; NoneM is a zero-composition sentinel.
    pub bulk_consumed: Option<BulkParticipant>,
    /// Signed implicit water products; negative means consumed solvent water.
    pub implicit_water: i32,
    /// Zero except for exact reaction 53: O3- -> H3O+ + O2-, giving [3, 0, 1].
    pub source_exception: [i64; 3],
}

/// Executed pair or unary lineage, placement and explicit reservoir accounting.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryReaction {
    /// Absolute event time in ps.
    pub time: Time,
    /// Source-table reaction identity.
    pub reaction_id: u16,
    /// One unary mobile parent or two source-ordered pair parents; never a bulk ID.
    pub participants: Vec<MoleculeId>,
    /// Versions at scheduling, before consumption increments them.
    pub generations: Vec<u64>,
    /// Monotonic identities in source product order, including repeated species.
    pub products: Vec<MoleculeId>,
    /// Source conditional parent sites and ordered products.
    pub placement: ReactionPlacement,
    /// Checked mobile, reservoir, implicit-water and source-exception terms.
    pub accounting: ReactionAccounting,
}

/// Compatibility name for the former pair-only reaction record.
pub type PairReaction = HistoryReaction;

#[derive(Debug, Clone, Copy)]
struct ParentVersion {
    id: MoleculeId,
    generation: u64,
}

#[derive(Debug, Clone, Copy)]
enum EventParents {
    Pair([ParentVersion; 2]),
    Unary(ParentVersion),
}

impl EventParents {
    fn as_slice(&self) -> &[ParentVersion] {
        match self {
            Self::Pair(parents) => parents,
            Self::Unary(parent) => std::slice::from_ref(parent),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PairEvent {
    time: f64,
    ordinal: u64,
    reaction_id: u16,
    parents: EventParents,
    sampled_at: f64,
}

impl PairEvent {
    fn is_current(&self, molecules: &[HistoryMolecule]) -> bool {
        let parents = self.parents.as_slice();
        (parents.len() == 1 || parents[0].id != parents[1].id)
            && parents.iter().all(|version| {
                let parent = &molecules[version.id.0 as usize];
                parent.alive && parent.generation == version.generation
            })
    }
}

impl PartialEq for PairEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time.total_cmp(&other.time) == Ordering::Equal && self.ordinal == other.ordinal
    }
}

impl Eq for PairEvent {}

impl PartialOrd for PairEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PairEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max heap: reverse both keys to select earliest/first.
        other
            .time
            .total_cmp(&self.time)
            .then_with(|| other.ordinal.cmp(&self.ordinal))
    }
}

#[derive(Debug, Default)]
struct EventQueue {
    heap: BinaryHeap<PairEvent>,
    next_ordinal: u64,
    peak_len: usize,
}

impl EventQueue {
    fn push(&mut self, mut event: PairEvent, now: f64, limit: usize) -> Result<(), ChemistryError> {
        validate_time("event time", event.time, now, MAX_END_PS)?;
        capacity("pending events", self.heap.len().saturating_add(1), limit)?;
        let next_ordinal =
            self.next_ordinal
                .checked_add(1)
                .ok_or(ChemistryError::HistoryCapacity {
                    resource: "event ordinals",
                    limit: usize::MAX,
                    required: usize::MAX,
                })?;
        event.ordinal = self.next_ordinal;
        self.heap.push(event);
        self.peak_len = self.peak_len.max(self.heap.len());
        self.next_ordinal = next_ordinal;
        Ok(())
    }
}

/// Incremental pair/unary history for diagnostics and selected-channel experiments.
/// Its constructor retains bounded lineage for compatibility; [`run_history`]
/// provides the complete-network count interface with tracing disabled by default.
#[derive(Debug)]
pub struct PairHistory<'a> {
    input: HistoryInput,
    molecules: Vec<HistoryMolecule>,
    descriptors: Vec<&'a CompiledReaction>,
    queue: EventQueue,
    reactions: Vec<PairReaction>,
    executed_reactions: usize,
    trace_limit: Option<usize>,
    options: PairHistoryOptions,
    now: f64,
    failed: bool,
    compositions: [[i64; 3]; Species::ALL.len()],
}

impl<'a> PairHistory<'a> {
    /// Initialize each eligible unordered pair once, in canonical identity order.
    ///
    /// The supplied descriptors explicitly define this chemical experiment.
    /// Duplicate channels are rejected before any RNG draws.
    /// Each unordered pair is sampled once per matching descriptor, with distinct
    /// molecule identities even for A+A. Candidates sort by ascending identity
    /// pair, then reaction ID; placement retains source reactant order separately.
    /// All-pairs scanning is deliberate for this slice, not production spatial search.
    pub fn new(
        input: &HistoryInput,
        descriptors: &[&'a CompiledReaction],
        options: PairHistoryOptions,
        rng: &mut impl Rng,
    ) -> Result<Self, ChemistryError> {
        let mut history = Self::prepare(input, descriptors, options, Some(options.max_reactions))?;
        history.initialize(rng)?;
        Ok(history)
    }

    fn prepare(
        input: &HistoryInput,
        descriptors: &[&'a CompiledReaction],
        options: PairHistoryOptions,
        trace_limit: Option<usize>,
    ) -> Result<Self, ChemistryError> {
        validate_time("end time", options.end_time.ps(), START_PS, MAX_END_PS)?;
        capacity("molecules", input.snapshot.len(), options.max_molecules)?;
        for (index, reaction) in descriptors.iter().enumerate() {
            if descriptors[..index]
                .iter()
                .any(|r| r.source().id == reaction.source().id)
            {
                return Err(ChemistryError::DuplicateHistoryReaction {
                    reaction_id: reaction.source().id,
                });
            }
        }
        let molecules: Vec<_> = input
            .snapshot
            .iter()
            .enumerate()
            .map(|(index, &record)| HistoryMolecule {
                molecule: Molecule::new(MoleculeId(index as u64), record),
                generation: 0,
                alive: true,
            })
            .collect();
        let mut descriptors = descriptors.to_vec();
        descriptors.sort_unstable_by_key(|reaction| reaction.source().id);
        let species = SpeciesTable::option3()?;
        let compositions = Species::ALL.map(|mobile| {
            let (h, o) = atoms(mobile);
            [
                i64::from(h),
                i64::from(o),
                i64::from(species.get(mobile).charge),
            ]
        });
        Ok(Self {
            input: input.clone(),
            molecules,
            descriptors,
            queue: EventQueue::default(),
            reactions: Vec::new(),
            executed_reactions: 0,
            trace_limit,
            options,
            now: START_PS,
            failed: false,
            compositions,
        })
    }

    fn initialize(&mut self, rng: &mut impl Rng) -> Result<(), ChemistryError> {
        // Stream candidates into the bounded queue rather than retaining an
        // unbounded quadratic candidate list. Each identity pair occurs once.
        for first in 0..self.molecules.len() {
            for second in (first + 1)..self.molecules.len() {
                self.schedule_matching_pair(first, second, rng)?;
            }
        }
        for parent in 0..self.molecules.len() {
            self.schedule_unary(parent, rng)?;
        }
        Ok(())
    }

    fn schedule_matching_pair(
        &mut self,
        first: usize,
        second: usize,
        rng: &mut impl Rng,
    ) -> Result<(), ChemistryError> {
        let species = [
            self.molecules[first].instance().species,
            self.molecules[second].instance().species,
        ];
        for index in 0..self.descriptors.len() {
            let Some(parameters) = self.descriptors[index].pair() else {
                continue;
            };
            let participants = if species == parameters.reactants() {
                [first, second]
            } else if [species[1], species[0]] == parameters.reactants() {
                [second, first]
            } else {
                continue;
            };
            self.schedule_pair(index, participants, rng)?;
        }
        Ok(())
    }

    fn parent_version(&self, index: usize) -> ParentVersion {
        ParentVersion {
            id: self.molecules[index].id(),
            generation: self.molecules[index].generation,
        }
    }

    fn schedule_pair(
        &mut self,
        reaction_index: usize,
        participants: [usize; 2],
        rng: &mut impl Rng,
    ) -> Result<(), ChemistryError> {
        let reaction = self.descriptors[reaction_index];
        let mut sampling_parents = participants.map(|index| self.molecules[index].molecule);
        // These temporary sites are deliberately not retained in the arena or
        // event: upstream uses them for r0 only, not for conditional placement.
        for (parent, diffusivity) in sampling_parents
            .iter_mut()
            .zip(reaction.reactant_diffusivities())
        {
            let instance = *parent.instance();
            let position = diffuse(
                instance.position,
                diffusivity,
                Time::from_ps(self.now - instance.time.ps()),
                rng,
            )?;
            *parent = Molecule::new(
                parent.id(),
                SpeciesInstance {
                    position,
                    ..instance
                },
            );
        }
        let pair = MoleculePair::new(&sampling_parents[0], &sampling_parents[1])?;
        let delay = sample_independent_reaction_time(
            pair,
            reaction.pair().expect("matching pair descriptor"),
            rng,
        )?;
        if let Some(time) = self.absolute_time(delay, "absolute pair time")? {
            self.queue.push(
                PairEvent {
                    time,
                    ordinal: 0,
                    reaction_id: reaction.source().id,
                    parents: EventParents::Pair(participants.map(|i| self.parent_version(i))),
                    sampled_at: self.now,
                },
                self.now,
                self.options.max_pending_events,
            )?;
        }
        Ok(())
    }

    fn absolute_time(
        &self,
        delay: ReactionTime,
        field: &'static str,
    ) -> Result<Option<f64>, ChemistryError> {
        let time = match delay {
            ReactionTime::Immediate => self.now,
            ReactionTime::After(delay) => {
                let time = self.now + delay.ps();
                if !time.is_finite() || time <= self.now {
                    return Err(ChemistryError::InvalidHistoryTime {
                        field,
                        value: time,
                        minimum: self.now,
                        maximum: self.options.end_time.ps(),
                    });
                }
                time
            }
            ReactionTime::NoReaction => return Ok(None),
        };
        Ok((time <= self.options.end_time.ps()).then_some(time))
    }

    fn schedule_unary(&mut self, parent: usize, rng: &mut impl Rng) -> Result<(), ChemistryError> {
        let mut earliest: Option<(f64, u16)> = None;
        for reaction in &self.descriptors {
            let Some(parameters) = reaction.unary() else {
                continue;
            };
            if parameters.mobile() != self.molecules[parent].instance().species {
                continue;
            }
            let delay = sample_unary_reaction_time(parameters, rng)?;
            if let Some(time) = self.absolute_time(delay, "absolute unary time")?
                && earliest.is_none_or(|(best, _)| time < best)
            {
                earliest = Some((time, reaction.source().id));
            }
        }
        // Other unary channels can never win for this unchanged mobile parent.
        // Keep the earliest even if a currently scheduled pair precedes it.
        if let Some((time, reaction_id)) = earliest {
            self.queue.push(
                PairEvent {
                    time,
                    ordinal: 0,
                    reaction_id,
                    parents: EventParents::Unary(self.parent_version(parent)),
                    sampled_at: self.now,
                },
                self.now,
                self.options.max_pending_events,
            )?;
        }
        Ok(())
    }

    /// Preserve the immutable input, even after contacts consume input molecules.
    pub fn pre_irt_snapshot(&self) -> &[SpeciesInstance] {
        self.input.snapshot()
    }

    /// Arena in identity order, including dead parents and source-ordered products.
    pub fn molecules(&self) -> &[HistoryMolecule] {
        &self.molecules
    }

    /// Executed reaction records in deterministic queue order.
    pub fn reactions(&self) -> &[PairReaction] {
        &self.reactions
    }

    /// Number of queued entries, including stale events not yet removed at the head.
    pub fn pending_events(&self) -> usize {
        self.queue.heap.len()
    }

    /// Earliest scheduled absolute time, without draws or survivor movement.
    pub fn next_event_time(&self) -> Option<Time> {
        self.queue
            .heap
            .peek()
            .map(|event| Time::from_ps(event.time))
    }

    /// Current absolute scheduler time; molecule materialization times may lag it.
    pub fn current_time(&self) -> Time {
        Time::from_ps(self.now)
    }

    /// Resource usage without counting species, drawing RNG or moving survivors.
    pub fn diagnostics(&self) -> HistoryDiagnostics {
        HistoryDiagnostics {
            time: self.current_time(),
            executed_reactions: self.executed_reactions,
            arena_records: self.molecules.len(),
            pending_events: self.pending_events(),
            peak_pending_events: self.queue.peak_len,
            trace_records: self.reactions.len(),
        }
    }

    /// Advance through all events at `time`, then count live species without draws
    /// or survivor movement. Reactions during advancement may consume randomness.
    /// Invalid/backwards requests do not mutate state; failed histories cannot
    /// produce a successful observation. At 1 ps this includes contact closure.
    pub fn observe_counts(
        &mut self,
        time: Time,
        rng: &mut impl Rng,
    ) -> Result<CountObservation, ChemistryError> {
        self.advance_to(time, rng)?;
        Ok(CountObservation {
            time,
            counts: count_species(
                self.molecules
                    .iter()
                    .filter(|m| m.alive)
                    .map(|m| m.instance().species),
            ),
        })
    }

    /// Process all initial immediate events before inspecting post-contact state.
    /// This is idempotent at 1 ps, and invalid after advancing beyond that time.
    pub fn close_contacts(&mut self, rng: &mut impl Rng) -> Result<(), ChemistryError> {
        self.advance_to(Time::from_ps(START_PS), rng)
    }

    /// Process every scheduled event at or before the absolute target time.
    ///
    /// Placement and capacity failures mark this history unusable for further
    /// advancement: callers may inspect partial diagnostics but cannot mistake
    /// a retry for a completed history. Invalid requested times do not mutate it.
    pub fn advance_to(&mut self, target: Time, rng: &mut impl Rng) -> Result<(), ChemistryError> {
        if self.failed {
            return Err(ChemistryError::HistoryAlreadyFailed);
        }
        validate_time(
            "advance time",
            target.ps(),
            self.now,
            self.options.end_time.ps(),
        )?;
        if let Err(error) = self.execute_through(target.ps(), rng) {
            self.failed = true;
            return Err(error);
        }
        self.now = target.ps();
        Ok(())
    }

    fn execute_through(&mut self, target: f64, rng: &mut impl Rng) -> Result<(), ChemistryError> {
        while let Some(&event) = self.queue.heap.peek() {
            // Discard invalid entries even beyond the target, so the exposed
            // next event always has live, unchanged parents. No clocks, placement
            // draws, reaction budget or scheduler time are spent on stale events.
            if !event.is_current(&self.molecules) {
                self.queue.heap.pop();
                continue;
            }
            if event.time > target {
                break;
            }
            capacity(
                "reactions",
                self.executed_reactions.saturating_add(1),
                self.options.max_reactions,
            )?;
            if let Some(limit) = self.trace_limit {
                capacity(
                    "trace records",
                    self.reactions.len().saturating_add(1),
                    limit,
                )?;
            }
            let reaction = *self
                .descriptors
                .iter()
                .find(|reaction| reaction.source().id == event.reaction_id)
                .expect("queued reaction belongs to immutable descriptors");
            let first_product = self.molecules.len();
            capacity(
                "molecules",
                first_product.saturating_add(reaction.source().products.len()),
                self.options.max_molecules,
            )?;
            let elapsed = Time::from_ps(event.time - event.sampled_at);
            let placement = match event.parents {
                EventParents::Pair(parents) => place_pair_reaction(
                    MoleculePair::new(
                        &self.molecules[parents[0].id.0 as usize].molecule,
                        &self.molecules[parents[1].id.0 as usize].molecule,
                    )?,
                    reaction,
                    elapsed,
                    rng,
                )?,
                EventParents::Unary(parent) => place_unary_reaction(
                    &self.molecules[parent.id.0 as usize].molecule,
                    reaction,
                    elapsed,
                    rng,
                )?,
            };
            let accounting = self.check_accounting(reaction, &event, &placement)?;
            let record = HistoryReaction {
                time: Time::from_ps(event.time),
                reaction_id: event.reaction_id,
                participants: event.parents.as_slice().iter().map(|p| p.id).collect(),
                generations: event
                    .parents
                    .as_slice()
                    .iter()
                    .map(|p| p.generation)
                    .collect(),
                products: (first_product..first_product + placement.products.len())
                    .map(|id| MoleculeId(id as u64))
                    .collect(),
                placement,
                accounting,
            };
            self.queue.heap.pop();
            for (parent_version, position) in event
                .parents
                .as_slice()
                .iter()
                .zip(record.placement.parent_positions)
            {
                let parent = &mut self.molecules[parent_version.id.0 as usize];
                parent.molecule = Molecule::new(
                    parent.id(),
                    SpeciesInstance {
                        position,
                        time: record.time,
                        ..*parent.instance()
                    },
                );
                parent.alive = false;
                parent.generation += 1;
            }
            for (id, product) in record.products.iter().zip(&record.placement.products) {
                self.molecules.push(HistoryMolecule {
                    molecule: Molecule::new(
                        *id,
                        SpeciesInstance {
                            species: product.species,
                            position: product.position,
                            time: record.time,
                        },
                    ),
                    generation: 0,
                    alive: true,
                });
            }
            self.now = event.time;
            self.executed_reactions += 1;
            if self.trace_limit.is_some() {
                self.reactions.push(record);
            }
            // Later siblings are not candidates until their source-order turn.
            // This exactly-once creation scan avoids old-old resampling, including
            // pairs whose initial independent clock returned NoReaction.
            for new in first_product..self.molecules.len() {
                for older in 0..new {
                    if self.molecules[older].alive {
                        self.schedule_matching_pair(older, new, rng)?;
                    }
                }
                self.schedule_unary(new, rng)?;
            }
        }
        Ok(())
    }

    fn check_accounting(
        &self,
        reaction: &CompiledReaction,
        event: &PairEvent,
        placement: &ReactionPlacement,
    ) -> Result<ReactionAccounting, ChemistryError> {
        let source = reaction.source();
        let parents = event.parents.as_slice();
        let actual_reactants = parents.iter().map(|p| {
            ReactionParticipant::Mobile(self.molecules[p.id.0 as usize].instance().species)
        });
        let expected_mobile = source
            .reactants
            .iter()
            .copied()
            .filter(|p| matches!(p, ReactionParticipant::Mobile(_)));
        if !actual_reactants.eq(expected_mobile)
            || !placement
                .products
                .iter()
                .map(|p| ReactionParticipant::Mobile(p.species))
                .eq(source.products.iter().copied())
        {
            return Err(ChemistryError::ReactionSourceMismatch {
                reaction_id: source.id,
            });
        }
        let mut mobile_change = [0; 3];
        for parent in parents {
            let species = self.molecules[parent.id.0 as usize].instance().species;
            for (sum, value) in mobile_change
                .iter_mut()
                .zip(self.compositions[species.index()])
            {
                *sum -= value;
            }
        }
        for product in &placement.products {
            for (sum, value) in mobile_change
                .iter_mut()
                .zip(self.compositions[product.species.index()])
            {
                *sum += value;
            }
        }
        let bulk_consumed = reaction.unary().map(|p| p.bulk());
        let bulk = match bulk_consumed {
            Some(BulkParticipant::H2OB) => [2, 1, 0],
            Some(BulkParticipant::H3OpB) => [3, 1, 1],
            Some(BulkParticipant::OHmB) => [1, 1, -1],
            Some(BulkParticipant::NoneM) | None => [0, 0, 0],
        };
        let water = i64::from(source.implicit_water);
        let source_exception =
            std::array::from_fn(|i| mobile_change[i] + [2 * water, water, 0][i] - bulk[i]);
        let exact_53 = source.id == 53
            && source.balance_exception
            && source.reactants
                == [
                    ReactionParticipant::Mobile(Species::O3m),
                    ReactionParticipant::Bulk(BulkParticipant::NoneM),
                ]
            && source.products
                == [
                    ReactionParticipant::Mobile(Species::H3Op),
                    ReactionParticipant::Mobile(Species::O2m),
                ]
            && source.implicit_water == 0
            && source_exception == [3, 0, 1];
        if !(exact_53 || (!source.balance_exception && source_exception == [0; 3])) {
            return Err(ChemistryError::ReactionSourceMismatch {
                reaction_id: source.id,
            });
        }
        Ok(ReactionAccounting {
            mobile_change,
            bulk_consumed,
            implicit_water: source.implicit_water,
            source_exception,
        })
    }
}

fn count_species(species: impl Iterator<Item = Species>) -> SpeciesCounts {
    let mut counts = [0; Species::ALL.len()];
    for mobile in species {
        counts[mobile.index()] += 1;
    }
    counts
}

fn validate_time(
    field: &'static str,
    value: f64,
    minimum: f64,
    maximum: f64,
) -> Result<(), ChemistryError> {
    if !value.is_finite() || value < minimum || value > maximum {
        return Err(ChemistryError::InvalidHistoryTime {
            field,
            value,
            minimum,
            maximum,
        });
    }
    Ok(())
}

fn capacity(resource: &'static str, required: usize, limit: usize) -> Result<(), ChemistryError> {
    if required > limit {
        return Err(ChemistryError::HistoryCapacity {
            resource,
            limit,
            required,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use g4_core::vec3::Vec3;

    #[test]
    fn accounting_rejects_changed_products_and_noncanonical_reaction_53_residual() {
        let network = crate::reactions::ReactionTable::option3()
            .unwrap()
            .compile(&SpeciesTable::option3().unwrap())
            .unwrap();
        let reaction = network.get(53).unwrap();
        let input = HistoryInput::new(&[SpeciesInstance {
            species: Species::O3m,
            position: Vec3::new(0.0, 0.0, 0.0),
            time: Time::from_ps(START_PS),
        }])
        .unwrap();
        let mut rng = g4_core::testing::ScriptedRng::new(&[1e-12]);
        let mut history =
            PairHistory::new(&input, &[reaction], PairHistoryOptions::default(), &mut rng).unwrap();
        let event = *history.queue.heap.peek().unwrap();
        let mut placement = place_unary_reaction(
            &history.molecules[0].molecule,
            reaction,
            Time::from_ps(event.time - START_PS),
            &mut rng,
        )
        .unwrap();
        assert_eq!(
            history
                .check_accounting(reaction, &event, &placement)
                .unwrap()
                .source_exception,
            [3, 0, 1]
        );
        placement.products[0].species = Species::H;
        assert_eq!(
            history.check_accounting(reaction, &event, &placement),
            Err(ChemistryError::ReactionSourceMismatch { reaction_id: 53 })
        );
        placement.products[0].species = Species::H3Op;
        // No blanket exemption for a descriptor carrying balance_exception.
        history.compositions[Species::H3Op.index()][0] += 1;
        assert_eq!(
            history.check_accounting(reaction, &event, &placement),
            Err(ChemistryError::ReactionSourceMismatch { reaction_id: 53 })
        );
    }

    fn event(time: f64, reaction_id: u16) -> PairEvent {
        PairEvent {
            time,
            ordinal: 0,
            reaction_id,
            parents: EventParents::Pair([0, 1].map(|id| ParentVersion {
                id: MoleculeId(id),
                generation: 0,
            })),
            sampled_at: START_PS,
        }
    }

    #[test]
    fn events_require_distinct_live_parents_with_matching_generations() {
        let parents = [0, 1].map(|id| HistoryMolecule {
            molecule: Molecule::new(
                MoleculeId(id),
                SpeciesInstance {
                    species: crate::Species::H,
                    position: Vec3::new(0.0, 0.0, 0.0),
                    time: Time::from_ps(START_PS),
                },
            ),
            generation: 0,
            alive: true,
        });
        let event = event(START_PS, 1);
        assert!(event.is_current(&parents));
        for index in 0..2 {
            let mut changed = parents;
            changed[index].generation += 1;
            assert!(
                !event.is_current(&changed),
                "live but version-changed parent"
            );
            let mut consumed = parents;
            consumed[index].alive = false;
            assert!(
                !event.is_current(&consumed),
                "dead parent with matching version"
            );
        }
        let self_pair = PairEvent {
            parents: EventParents::Pair(
                [ParentVersion {
                    id: MoleculeId(0),
                    generation: 0,
                }; 2],
            ),
            ..event
        };
        assert!(!self_pair.is_current(&parents));
    }

    #[test]
    fn event_queue_rejects_invalid_and_backwards_times_without_mutation() {
        let mut queue = EventQueue::default();
        for time in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, 0.0, 1.0] {
            assert!(matches!(
                queue.push(event(time, 0), 2.0, 10),
                Err(ChemistryError::InvalidHistoryTime { .. })
            ));
            assert!(queue.heap.is_empty());
            assert_eq!(queue.next_ordinal, 0);
        }
    }

    #[test]
    fn queue_orders_times_then_insertion_without_comparing_payloads() {
        let mut queue = EventQueue::default();
        for (time, id) in [(3.0, 10), (2.0, 30), (2.0, 20), (2.0, 0)] {
            queue.push(event(time, id), 1.0, 4).unwrap();
        }
        assert_eq!(
            (0..4)
                .map(|_| queue.heap.pop().unwrap().reaction_id)
                .collect::<Vec<_>>(),
            [30, 20, 0, 10]
        );
    }

    #[test]
    fn ordinal_exhaustion_does_not_wrap_or_enqueue() {
        let mut queue = EventQueue {
            next_ordinal: u64::MAX,
            ..Default::default()
        };
        assert!(matches!(
            queue.push(event(1.0, 0), 1.0, 1),
            Err(ChemistryError::HistoryCapacity {
                resource: "event ordinals",
                ..
            })
        ));
        assert!(queue.heap.is_empty());
    }
}
