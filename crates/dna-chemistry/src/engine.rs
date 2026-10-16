//! The physicochemical stage of one history: physical-stage events -> chemical species at the birth time.
//!
//! One history (one primary and its whole cascade) is converted at a time, with the generator of the stage
//! `Stage::PhysChem` of that history (docs/reproducibility.md). Geant4 runs the chemistry of one event at a
//! time too (`chem6`), so the e_aq that a hole can recombine with are those of the same history (isolated-track
//! approximation, docs/05 E.1).
//!
//! # Processing order and its assumption
//! Solvation events first create their `e_aq` (in Geant4 these tracks are pushed to the chemistry stack when the
//! electron is solvated, so they exist when the chemical stage starts). Then the water-molecule events
//! (excitation, ionisation, attachment) are processed **in the order of the event list**, which is the order of
//! the physical cascade. Auto-ionisation e_aq appear when their parent is processed, so they are candidates for
//! the holes that follow. Geant4 processes the molecules in the order of its track list, which is not specified by
//! the source read (docs/09 Q21); only the pairing of a hole with an e_aq depends on it.
//!
//! # Random numbers (per history, in order)
//! For each non-solvation event: [ionisation only: one uniform for the recombination test, always drawn];
//! one uniform for the channel; then the draws of the displacer (`displacer` module). Solvation events draw
//! nothing (their position is already displaced).

use crate::channels::{ChannelTable, DecayState};
use crate::displacer::{autoionisation_r_mean_nm, displacements};
use crate::error::ChemistryError;
use crate::params::PhysChemParams;
use crate::species::SpeciesTable;
use dna_core::physchem::{PhysChemEvent, PhysChemKind};
use dna_core::sampling::uniform;
use dna_core::species::{Species, SpeciesInstance};
use dna_core::units::Energy;
use dna_core::vec3::Vec3;
use rand::Rng;
use std::collections::HashMap;

/// Counters of one or several histories (additive, see [`PhysChemStats::add`]).
#[derive(Debug, Clone, PartialEq)]
pub struct PhysChemStats {
    /// Events by kind: excitation, ionisation, attachment, solvation.
    pub events: [u64; 4],
    /// `channel_counts[state][channel]`, states and channels in table order.
    pub channel_counts: Vec<Vec<u64>>,
    /// Ionised molecules for which at least one alive e_aq lay within the search radius.
    pub holes_with_candidate: u64,
    /// Ionised molecules that recombined with an e_aq.
    pub recombinations: u64,
    /// Sum of the energies booked by the chemistry-stage decays (relax channels), eV. Not part of the
    /// physical-stage ledger (docs/09 Q20).
    pub decay_energy_ev: f64,
    /// Number of alive species of each kind at the end of the stage, indexed by [`Species::index`].
    pub species_counts: [u64; 15],
    /// Products created by excitation, ionisation, attachment and solvation events, before
    /// electron-hole recombination removes an electron. Indexed `[event_kind][species]`.
    pub species_produced_by_event: [[u64; 15]; 4],
    /// Number and sum of squared displacements from the parent event, by product species.
    /// Solvation is excluded because its event position is already the displaced position.
    pub displacement_count: [u64; 15],
    pub displacement_sum_sq_nm2: [f64; 15],
    /// Same-decay OH--e_aq and OH--H3O+ pair counts, sums and squared sums (nm).
    pub pair_count: [u64; 2],
    pub pair_sum_nm: [f64; 2],
    pub pair_sum_sq_nm2: [f64; 2],
}

impl PhysChemStats {
    fn new(table: &ChannelTable) -> Self {
        Self {
            events: [0; 4],
            channel_counts: table
                .states()
                .iter()
                .map(|s| vec![0; s.channels.len()])
                .collect(),
            holes_with_candidate: 0,
            recombinations: 0,
            decay_energy_ev: 0.0,
            species_counts: [0; 15],
            species_produced_by_event: [[0; 15]; 4],
            displacement_count: [0; 15],
            displacement_sum_sq_nm2: [0.0; 15],
            pair_count: [0; 2],
            pair_sum_nm: [0.0; 2],
            pair_sum_sq_nm2: [0.0; 2],
        }
    }

    /// Add the counters of another run (same table).
    pub fn add(&mut self, other: &PhysChemStats) {
        for (a, b) in self.events.iter_mut().zip(&other.events) {
            *a += b;
        }
        for (sa, sb) in self.channel_counts.iter_mut().zip(&other.channel_counts) {
            for (a, b) in sa.iter_mut().zip(sb) {
                *a += b;
            }
        }
        self.holes_with_candidate += other.holes_with_candidate;
        self.recombinations += other.recombinations;
        self.decay_energy_ev += other.decay_energy_ev;
        for (a, b) in self.species_counts.iter_mut().zip(&other.species_counts) {
            *a += b;
        }
        for (a, b) in self
            .species_produced_by_event
            .iter_mut()
            .flatten()
            .zip(other.species_produced_by_event.iter().flatten())
        {
            *a += b;
        }
        for i in 0..15 {
            self.displacement_count[i] += other.displacement_count[i];
            self.displacement_sum_sq_nm2[i] += other.displacement_sum_sq_nm2[i];
        }
        for i in 0..2 {
            self.pair_count[i] += other.pair_count[i];
            self.pair_sum_nm[i] += other.pair_sum_nm[i];
            self.pair_sum_sq_nm2[i] += other.pair_sum_sq_nm2[i];
        }
    }
}

/// Result of the physicochemical stage of one history.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryChemistry {
    /// Alive species, in creation order, all at the birth time.
    pub species: Vec<SpeciesInstance>,
    pub stats: PhysChemStats,
}

/// Result of the decay of one water-molecule event taken alone (no recombination partner).
#[derive(Debug, Clone, PartialEq)]
pub struct EventDecay {
    /// Index of the decay state (see [`ChannelTable::states`]).
    pub state: usize,
    /// Index of the selected channel within the state.
    pub channel: usize,
    pub species: Vec<SpeciesInstance>,
    pub decay_energy: Energy,
}

/// Uniform grid of the alive e_aq, cell edge = search radius, for the nearest-neighbour query.
struct ElectronGrid {
    cell_nm: f64,
    cells: HashMap<[i64; 3], Vec<u32>>,
}

impl ElectronGrid {
    fn new(cell_nm: f64) -> Self {
        Self {
            cell_nm,
            cells: HashMap::new(),
        }
    }

    fn key(&self, p: Vec3) -> [i64; 3] {
        [
            (p.x / self.cell_nm).floor() as i64,
            (p.y / self.cell_nm).floor() as i64,
            (p.z / self.cell_nm).floor() as i64,
        ]
    }

    fn insert(&mut self, id: u32, p: Vec3) {
        let k = self.key(p);
        self.cells.entry(k).or_default().push(id);
    }

    /// Nearest alive electron within `radius_nm` (<= the cell edge) of `p`; ties go to the lowest id.
    fn nearest(
        &self,
        p: Vec3,
        radius_nm: f64,
        species: &[SpeciesInstance],
        alive: &[bool],
    ) -> Option<(u32, f64)> {
        let k = self.key(p);
        let mut best: Option<(u32, f64)> = None;
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let Some(ids) = self.cells.get(&[k[0] + dx, k[1] + dy, k[2] + dz]) else {
                        continue;
                    };
                    for &id in ids {
                        if !alive[id as usize] {
                            continue;
                        }
                        let q = species[id as usize].position;
                        let d = ((q.x - p.x).powi(2) + (q.y - p.y).powi(2) + (q.z - p.z).powi(2))
                            .sqrt();
                        if d <= radius_nm
                            && best.is_none_or(|(bid, bd)| d < bd || (d == bd && id < bid))
                        {
                            best = Some((id, d));
                        }
                    }
                }
            }
        }
        best
    }
}

/// The physicochemical stage: data plus the algorithm.
#[derive(Debug, Clone)]
pub struct PhysChemEngine {
    species: SpeciesTable,
    channels: ChannelTable,
    params: PhysChemParams,
    electron_r_mean_nm: f64,
}

impl PhysChemEngine {
    /// The option3 engine with the embedded data.
    pub fn option3() -> Result<Self, ChemistryError> {
        let species = SpeciesTable::option3()?;
        let channels = ChannelTable::option3(&species)?;
        Self::new(species, channels, PhysChemParams::option3()?)
    }

    pub fn new(
        species: SpeciesTable,
        channels: ChannelTable,
        params: PhysChemParams,
    ) -> Result<Self, ChemistryError> {
        let electron_r_mean_nm =
            autoionisation_r_mean_nm(params.autoionisation_electron_energy.ev());
        Ok(Self {
            species,
            channels,
            params,
            electron_r_mean_nm,
        })
    }

    pub fn species_table(&self) -> &SpeciesTable {
        &self.species
    }

    pub fn channel_table(&self) -> &ChannelTable {
        &self.channels
    }

    pub fn params(&self) -> &PhysChemParams {
        &self.params
    }

    /// Replace the recombination switch (ablation studies and tests).
    pub fn with_recombination_enabled(mut self, enabled: bool) -> Self {
        self.params.recombination.enabled = enabled;
        self
    }

    fn decay<R: Rng + ?Sized>(
        &self,
        state: &DecayState,
        position: Vec3,
        rng: &mut R,
        out: &mut Vec<SpeciesInstance>,
    ) -> (usize, Energy) {
        let u = uniform(rng);
        let ci = state.select(u);
        let channel = &state.channels[ci];
        let offsets = displacements(
            channel.displacement,
            &channel.products,
            self.electron_r_mean_nm,
            rng,
        );
        for (&species, &offset) in channel.products.iter().zip(&offsets) {
            out.push(SpeciesInstance {
                species,
                position: position + offset,
                time: self.params.birth_time,
            });
        }
        (ci, channel.decay_energy)
    }

    /// Decay one excitation / ionisation / attachment event alone (without recombination). Used to validate the
    /// channel statistics and the displacements; the history conversion is [`PhysChemEngine::convert_history`].
    pub fn decay_event<R: Rng + ?Sized>(
        &self,
        event: &PhysChemEvent,
        rng: &mut R,
    ) -> Result<EventDecay, ChemistryError> {
        let si = self
            .channels
            .state_index(event.kind, event.level_or_shell)?;
        Ok(self.decay_in_state(si, event.position, rng))
    }

    /// Decay a water molecule that sits in decay state `state` (index in [`ChannelTable::states`], e.g. the
    /// recombined `H2Ovib` state) at `position`.
    pub fn decay_in_state<R: Rng + ?Sized>(
        &self,
        state: usize,
        position: Vec3,
        rng: &mut R,
    ) -> EventDecay {
        let mut species = Vec::new();
        let (channel, decay_energy) =
            self.decay(&self.channels.states()[state], position, rng, &mut species);
        EventDecay {
            state,
            channel,
            species,
            decay_energy,
        }
    }

    /// The whole stage for one history (see the module documentation for the processing order).
    pub fn convert_history<R: Rng + ?Sized>(
        &self,
        events: &[PhysChemEvent],
        rng: &mut R,
    ) -> Result<HistoryChemistry, ChemistryError> {
        let rec = &self.params.recombination;
        let mut stats = PhysChemStats::new(&self.channels);
        let mut species: Vec<SpeciesInstance> = Vec::with_capacity(events.len() * 2);
        let mut alive: Vec<bool> = Vec::with_capacity(events.len() * 2);
        let mut grid = ElectronGrid::new(rec.search_radius.nm());

        // 1. the solvated electrons exist when the chemical stage starts
        for ev in events.iter().filter(|e| e.kind == PhysChemKind::Solvation) {
            stats.events[3] += 1;
            let id = species.len() as u32;
            species.push(SpeciesInstance {
                species: Species::e_aq,
                position: ev.position,
                time: self.params.birth_time,
            });
            alive.push(true);
            grid.insert(id, ev.position);
            stats.species_produced_by_event[3][Species::e_aq.index()] += 1;
        }

        // 2. the water molecules, in event order
        let recombined = self.channels.recombined_index();
        let mut scratch: Vec<SpeciesInstance> = Vec::with_capacity(3);
        for ev in events.iter().filter(|e| e.kind != PhysChemKind::Solvation) {
            let mut si = self.channels.state_index(ev.kind, ev.level_or_shell)?;
            stats.events[match ev.kind {
                PhysChemKind::Excitation => 0,
                PhysChemKind::Ionisation => 1,
                PhysChemKind::Attachment => 2,
                PhysChemKind::Solvation => 3,
            }] += 1;

            if ev.kind == PhysChemKind::Ionisation {
                let u_rec = uniform(rng); // always drawn (docs/reproducibility.md)
                if rec.enabled {
                    if let Some((id, d)) =
                        grid.nearest(ev.position, rec.search_radius.nm(), &species, &alive)
                    {
                        stats.holes_with_candidate += 1;
                        if rec.probability(d) > u_rec {
                            alive[id as usize] = false;
                            stats.recombinations += 1;
                            si = recombined;
                        }
                    }
                }
            }

            scratch.clear();
            let state = &self.channels.states()[si];
            let (ci, decay_energy) = self.decay(state, ev.position, rng, &mut scratch);
            stats.channel_counts[si][ci] += 1;
            stats.decay_energy_ev += decay_energy.ev();
            let event_index = match ev.kind {
                PhysChemKind::Excitation => 0,
                PhysChemKind::Ionisation => 1,
                PhysChemKind::Attachment => 2,
                PhysChemKind::Solvation => unreachable!(),
            };
            for inst in &scratch {
                let i = inst.species.index();
                stats.species_produced_by_event[event_index][i] += 1;
                stats.displacement_count[i] += 1;
                let displacement = inst.position - ev.position;
                stats.displacement_sum_sq_nm2[i] += displacement.dot(displacement);
            }
            for (pair_index, other) in [Species::e_aq, Species::H3Op].into_iter().enumerate() {
                for oh in scratch.iter().filter(|s| s.species == Species::OH) {
                    for partner in scratch.iter().filter(|s| s.species == other) {
                        let d = (oh.position - partner.position).norm();
                        stats.pair_count[pair_index] += 1;
                        stats.pair_sum_nm[pair_index] += d;
                        stats.pair_sum_sq_nm2[pair_index] += d * d;
                    }
                }
            }
            for inst in scratch.drain(..) {
                if inst.species == Species::e_aq {
                    grid.insert(species.len() as u32, inst.position);
                }
                species.push(inst);
                alive.push(true);
            }
        }

        let species: Vec<SpeciesInstance> = species
            .into_iter()
            .zip(alive)
            .filter_map(|(s, a)| a.then_some(s))
            .collect();
        for s in &species {
            stats.species_counts[s.species.index()] += 1;
        }
        Ok(HistoryChemistry { species, stats })
    }
}
