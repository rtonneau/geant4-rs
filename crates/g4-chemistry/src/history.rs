//! Serial history storage and the first, disjoint-pair execution slice.
//!
//! Source baseline: Geant4 **11.4.1**, `source/processes/electromagnetic/dna/`
//! `models/src/G4DNAIRT.cc`: `IRTSampling`/`Sampling` (161-244) initialize
//! pairs once and add global time to elapsed clocks; `MakeReaction` (393-508)
//! materializes conditional parent sites and source-ordered products;
//! `FindReaction` (513-549) processes events through the current global time.
//! The existing [`crate::irt`] and [`crate::placement`] kernels retain their
//! documented source/numerical limitations. The 11.4.3 G1-F evidence does not
//! migrate these scheduler semantics from their 11.4.1 baseline.
//!
//! This slice accepts a selected set of compiled **pair** descriptors and
//! rejects competing pairs before drawing randomness. Products are recorded
//! in [`PairReaction`], **not inserted or evolved**. Unary chemistry, product
//! cascades, spatial search and complete-network scientific scores are not
//! implemented here. Empty/nonreacting inputs and several disjoint pairs are
//! supported. No claim of complete chemistry or R05/G2 acceptance is made.
//!
//! Molecules keep creation-order identities, generation and last materialized
//! time. Survivors are never diffused merely to advance the clock. The input
//! snapshot remains immutable. Events order by absolute time, then insertion
//! ordinal; candidates order by ascending input IDs, independent of descriptor
//! order. Source reactant order is retained separately for conditional placement.
//! Unlike upstream's strict `irt < timeMax - globalTime`, the final boundary is
//! inclusive, as required by the approved R05 design.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::irt::{ReactionTime, sample_independent_reaction_time};
use crate::molecule::{Molecule, MoleculeId, MoleculePair};
use crate::placement::{ReactionPlacement, place_pair_reaction};
use crate::reaction_network::CompiledReaction;
use crate::{ChemistryError, SpeciesInstance};
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

/// Finite horizon and storage budgets for the isolated-pair slice.
#[derive(Debug, Clone, Copy)]
pub struct PairHistoryOptions {
    /// Inclusive absolute final time, from 1 ps through 1 us.
    pub end_time: Time,
    /// Maximum arena records, including dead records retained for identity.
    pub max_molecules: usize,
    /// Maximum simultaneous queued events.
    pub max_pending_events: usize,
    /// Maximum executed reactions (and retained reaction records).
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

    /// Whether this input molecule has not been consumed.
    pub fn is_alive(&self) -> bool {
        self.alive
    }
}

/// Executed pair lineage and placement; products have no history IDs in this slice.
#[derive(Debug, Clone, PartialEq)]
pub struct PairReaction {
    /// Absolute event time in ps.
    pub time: Time,
    /// Source-table reaction identity.
    pub reaction_id: u16,
    /// Parent IDs in source reactant order, including A+A's two distinct IDs.
    pub participants: [MoleculeId; 2],
    /// Versions at scheduling, before consumption increments them.
    pub generations: [u64; 2],
    /// Source conditional parent sites and ordered, unevolved products.
    pub placement: ReactionPlacement,
}

#[derive(Debug, Clone, Copy)]
struct PairEvent {
    time: f64,
    ordinal: u64,
    reaction_id: u16,
    participants: [MoleculeId; 2],
    generations: [u64; 2],
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
        self.next_ordinal = next_ordinal;
        Ok(())
    }
}

/// Caller-RNG-driven isolated pair history, not a complete chemistry result.
#[derive(Debug)]
pub struct PairHistory<'a> {
    input: HistoryInput,
    molecules: Vec<HistoryMolecule>,
    descriptors: Vec<&'a CompiledReaction>,
    queue: EventQueue,
    reactions: Vec<PairReaction>,
    options: PairHistoryOptions,
    now: f64,
    failed: bool,
}

impl<'a> PairHistory<'a> {
    /// Initialize each eligible unordered pair once, in canonical identity order.
    ///
    /// The supplied descriptors explicitly define this limited pair experiment.
    /// Duplicate channels, unary channels and molecules eligible for several
    /// pairs are rejected before any RNG draws. All-pairs scanning is deliberate
    /// for this initial slice; it is not the production spatial search.
    pub fn new(
        input: &HistoryInput,
        descriptors: &[&'a CompiledReaction],
        options: PairHistoryOptions,
        rng: &mut impl Rng,
    ) -> Result<Self, ChemistryError> {
        validate_time("end time", options.end_time.ps(), START_PS, MAX_END_PS)?;
        capacity("molecules", input.snapshot.len(), options.max_molecules)?;
        for (index, reaction) in descriptors.iter().enumerate() {
            if reaction.pair().is_none() {
                return Err(ChemistryError::UnsupportedReactionOrder {
                    reaction_id: reaction.source().id,
                });
            }
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
        // Validate the slice before sampling: unsupported competition must not
        // depend on a stochastic NoReaction outcome. At most N/2 candidates.
        let mut used = vec![false; molecules.len()];
        let mut candidates = Vec::new();
        for first in 0..molecules.len() {
            for second in (first + 1)..molecules.len() {
                let species = [
                    molecules[first].instance().species,
                    molecules[second].instance().species,
                ];
                for (reaction_index, reaction) in descriptors.iter().enumerate() {
                    let reactants = reaction
                        .pair()
                        .expect("validated pair descriptor")
                        .reactants();
                    let participants = if species == reactants {
                        [first, second]
                    } else if [species[1], species[0]] == reactants {
                        [second, first]
                    } else {
                        continue;
                    };
                    for index in [first, second] {
                        if used[index] {
                            return Err(ChemistryError::CompetingHistoryPair {
                                id: molecules[index].id(),
                            });
                        }
                        used[index] = true;
                    }
                    candidates.push((reaction_index, participants));
                }
            }
        }
        let mut history = Self {
            input: input.clone(),
            molecules,
            descriptors: descriptors.to_vec(),
            queue: EventQueue::default(),
            reactions: Vec::new(),
            options,
            now: START_PS,
            failed: false,
        };
        for (reaction_index, participants) in candidates {
            let pair = MoleculePair::new(
                &history.molecules[participants[0]].molecule,
                &history.molecules[participants[1]].molecule,
            )?;
            let parameters = descriptors[reaction_index]
                .pair()
                .expect("validated pair descriptor");
            let time = match sample_independent_reaction_time(pair, parameters, rng)? {
                ReactionTime::Immediate => START_PS,
                ReactionTime::After(delay) => {
                    let time = START_PS + delay.ps();
                    if !time.is_finite() || time <= START_PS {
                        return Err(ChemistryError::InvalidHistoryTime {
                            field: "absolute pair time",
                            value: time,
                            minimum: START_PS,
                            maximum: options.end_time.ps(),
                        });
                    }
                    time
                }
                ReactionTime::NoReaction => continue,
            };
            if time <= options.end_time.ps() {
                history.queue.push(
                    PairEvent {
                        time,
                        ordinal: 0,
                        reaction_id: descriptors[reaction_index].source().id,
                        participants: participants.map(|index| history.molecules[index].id()),
                        generations: [0; 2],
                    },
                    START_PS,
                    options.max_pending_events,
                )?;
            }
        }
        Ok(history)
    }

    /// Preserve the immutable input, even after contacts consume input molecules.
    pub fn pre_irt_snapshot(&self) -> &[SpeciesInstance] {
        self.input.snapshot()
    }

    /// Arena in identity order; includes dead parents and no unevolved products.
    pub fn molecules(&self) -> &[HistoryMolecule] {
        &self.molecules
    }

    /// Executed reaction records in deterministic queue order.
    pub fn reactions(&self) -> &[PairReaction] {
        &self.reactions
    }

    /// Number of remaining finite-horizon pair events.
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
            if event.time > target {
                break;
            }
            capacity(
                "reactions",
                self.reactions.len().saturating_add(1),
                self.options.max_reactions,
            )?;
            let parents = event.participants.map(|id| &self.molecules[id.0 as usize]);
            // Disjoint initialization guarantees liveness; retain versions in
            // the event so the next slice can invalidate competing events.
            debug_assert!(
                parents
                    .iter()
                    .zip(event.generations)
                    .all(|(parent, generation)| parent.alive && parent.generation == generation)
            );
            let reaction = self
                .descriptors
                .iter()
                .find(|reaction| reaction.source().id == event.reaction_id)
                .expect("queued reaction belongs to immutable descriptors");
            let elapsed = Time::from_ps(event.time - parents[0].instance().time.ps());
            let placement = place_pair_reaction(
                MoleculePair::new(&parents[0].molecule, &parents[1].molecule)?,
                reaction,
                elapsed,
                rng,
            )?;
            let record = PairReaction {
                time: Time::from_ps(event.time),
                reaction_id: event.reaction_id,
                participants: event.participants,
                generations: event.generations,
                placement,
            };
            self.queue.heap.pop();
            for (id, position) in event
                .participants
                .into_iter()
                .zip(record.placement.parent_positions)
            {
                let parent = &mut self.molecules[id.0 as usize];
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
            self.now = event.time;
            self.reactions.push(record);
        }
        Ok(())
    }
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

    fn event(time: f64, reaction_id: u16) -> PairEvent {
        PairEvent {
            time,
            ordinal: 0,
            reaction_id,
            participants: [MoleculeId(0), MoleculeId(1)],
            generations: [0, 0],
        }
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
