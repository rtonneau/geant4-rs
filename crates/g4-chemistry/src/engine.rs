//! The physicochemical stage of one history: physical-stage events -> chemical species at the birth time.
//!
//! One history (one primary and its whole cascade) is converted at a time, with the generator of the stage
//! `Stage::PhysChem` of that history (docs/reproducibility.md). Geant4 runs the chemistry of one event at a
//! time too (`chem6`), so the e_aq that a hole can recombine with are those of the same history (isolated-track
//! approximation, docs/05 E.1).
//!
//! # Schedule: Geant4's zero-time steps at 1 ps
//! Geant4 11.4.1 decays the water molecules and runs the electron-hole recombination in zero-time steps of
//! the IT scheduler (`D` = `source/processes/electromagnetic/dna`). Every step first computes the
//! interaction lengths of all tracks, then performs the selected actions (`D/management/src/G4Scheduler.cc:584,682`).
//! The stage reproduces that schedule as *passes* (docs/08, "M6 residual audit"):
//! 1. The solvated electrons exist when the stage starts (`D/utils/src/G4DNAChemistryManager.cc:668-714`).
//! 2. At the start of a pass every pending ionised molecule finds the alive e_aq within 10 r_Onsager
//!    (`FindReactant`, `D/processes/src/G4DNAElectronHoleRecombination.cc:213-308`). `ComputeInteractionLength`
//!    calls it for every track before any action (`D/management/src/G4ITStepProcessor.cc:535-626`), on a
//!    KD tree rebuilt from the track list at each zero-time step
//!    (`D/models/src/G4DNAIRTMoleculeEncounterStepper.cc:80-87`).
//! 3. The molecules are then processed in Geant4's `DoIt` order. The main list holds one sub-list per molecular
//!    configuration, in ascending configuration ID (`D/management/src/G4ITTrackHolder.cc:288-356,401-443,620-658`,
//!    `D/management/include/G4ManyFastLists.hh:91-94`), with the molecules in creation (event) order inside each
//!    sub-list. The IDs follow the creation order of `G4ChemDissociationChannels_option1.cc:237-459`
//!    (`source/physics_lists/constructors/electromagnetic/src/`), which is the state order of the channel table.
//!    `G4ITStepProcessor::DoIt` walks the whole list backwards (`D/management/src/G4ITStepProcessor2.cc:110-187`).
//!    Hence: descending state index, and reverse event order within a state.
//! 4. An ionised molecule with uniform `u` and `p(d) <= u` for its nearest candidate at the pass start, or
//!    without a candidate, dissociates. Otherwise it tests its nearest *still alive* candidate with the same
//!    `u` (`MakeReaction`, cc:161-209). If `p > u` it recombines: the e_aq is killed and the molecule decays
//!    through `H2Ovib`. If not (the nearest one was taken earlier in the pass), the molecule stays ionised
//!    (cc:205-208) and is tested again in the next pass. It then draws a new uniform and sees the updated
//!    electron set (`GetMeanLifeTime`, cc:336-344).
//! 5. Products, including auto-ionisation e_aq, join only after the pass (`G4ITStepProcessor2.cc:184-185`).
//!    An auto-ionisation electron is therefore a candidate only for a molecule tested again in a later pass.
//!
//! A recombined molecule decays at once. Geant4 decays its `H2Ovib` in the next zero-time step, but no `H2Ovib`
//! channel makes an e_aq, so no candidate set depends on that delay.
//!
//! # Random numbers (per history, in order)
//! Pass by pass, molecule by molecule in the order above: [ionised molecule only: one uniform for the
//! recombination test, drawn in every pass in which the molecule is pending, also without a candidate or with
//! the process switched off]; then, if the molecule decays in this pass, one uniform for the channel and the
//! draws of the displacer (`displacer` module). Solvation events draw nothing (their position is already
//! displaced). Geant4 draws its `u` in `FindReactant`, before the actions; drawing it at the action is the same
//! distribution, because `u` is independent of everything else.

use crate::channels::{
    ChannelTable, DecayState, DisplacementKind, PROBABILITY_SUM_TOLERANCE, StateKind,
};
use crate::displacer::{autoionisation_r_mean_nm, displacements};
use crate::error::ChemistryError;
use crate::params::PhysChemParams;
use crate::species::{SpeciesTable, atoms};
use crate::{Species, SpeciesInstance};
use g4_core::physchem::{PhysChemEvent, PhysChemKind};
use g4_core::sampling::uniform;
use g4_core::sha256::{Sha256, hex};
use g4_core::units::Energy;
use g4_core::vec3::Vec3;
use rand::Rng;
use std::collections::HashMap;

/// Counters of one or several histories (additive, see [`PhysChemStats::add`]).
#[derive(Debug, Clone, PartialEq)]
pub struct PhysChemStats {
    /// Events by kind: excitation, ionisation, attachment, solvation.
    pub events: [u64; 4],
    /// `channel_counts[state][channel]`, states and channels in table order.
    pub channel_counts: Vec<Vec<u64>>,
    /// Ionised molecules for which at least one alive e_aq lay within the search radius at their first test.
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

/// A daughter birth in the actual zero-time decay schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DissociationBirth {
    /// Index in the input physical event slice.
    pub source_event: usize,
    /// Stable slot before killed electrons are removed; not the final survivor index.
    pub species_index: usize,
    /// Zero-based Rust pass. Recombined H2Ovib decays immediately in this engine,
    /// so this is not an asserted Geant4 scheduler-step correspondence.
    pub zero_time_pass: u64,
    /// Zero-based water action ordinal across all passes, including unsuccessful retests.
    pub processing_ordinal: u64,
}

/// Read-only observations of the conversion lifecycle; callbacks receive no RNG.
/// Initial water and direct-electron callbacks precede every decay/recombination.
/// Daughter callbacks record births, while survivor callbacks omit killed electrons.
pub trait PhysChemObserver {
    fn on_solvated_electron(
        &mut self,
        _source_event: usize,
        _species_index: usize,
        _species: &SpeciesInstance,
    ) {
    }
    fn on_water(&mut self, _source_event: usize, _event: &PhysChemEvent) {}
    fn on_dissociation_product(&mut self, _birth: DissociationBirth, _species: &SpeciesInstance) {}
    fn on_survivor(&mut self, _species_index: usize, _species: &SpeciesInstance) {}
}

/// Production conversion has no diagnostic allocations or records.
pub struct NoPhysChemObserver;
impl PhysChemObserver for NoPhysChemObserver {}

/// Index of an event kind in [`PhysChemStats::events`] and [`PhysChemStats::species_produced_by_event`].
fn event_kind_index(kind: PhysChemKind) -> usize {
    match kind {
        PhysChemKind::Excitation => 0,
        PhysChemKind::Ionisation => 1,
        PhysChemKind::Attachment => 2,
        PhysChemKind::Solvation => 3,
    }
}

/// Uniform grid of the alive e_aq, cell edge = search radius, for the nearest-neighbour query.
struct ElectronGrid {
    cell_nm: f64,
    cells: HashMap<[i64; 3], Vec<u32>>,
}

fn validate_species(species: &SpeciesTable) -> Result<(), ChemistryError> {
    for (expected, props) in Species::ALL.into_iter().zip(species.iter()) {
        if props.species != expected
            || !props.diffusivity.nm2_per_ps().is_finite()
            || props.diffusivity.nm2_per_ps() <= 0.0
            || !props.radius.nm().is_finite()
            || props.radius.nm() <= 0.0
        {
            return Err(ChemistryError::invalid(
                "species table",
                format!("invalid properties for {}", expected.chemistry_name()),
            ));
        }
    }
    Ok(())
}

fn validate_channels(
    channels: &ChannelTable,
    species: &SpeciesTable,
) -> Result<(), ChemistryError> {
    for state in channels.states() {
        if state.channels.is_empty() {
            return Err(ChemistryError::invalid(
                "channel table",
                format!("{} has no channels", state.name),
            ));
        }
        let parent_charge = match state.kind {
            StateKind::Excitation | StateKind::Recombined => 0,
            StateKind::Ionisation => 1,
            StateKind::Attachment => -1,
        };
        let mut sum = 0.0;
        for channel in &state.channels {
            if !channel.probability.is_finite()
                || !(0.0..=1.0).contains(&channel.probability)
                || !channel.decay_energy.ev().is_finite()
                || channel.decay_energy.ev() < 0.0
                || channel.products.is_empty() != (channel.displacement == DisplacementKind::None)
            {
                return Err(ChemistryError::invalid(
                    "channel table",
                    format!("invalid channel {}", channel.name),
                ));
            }
            if !channel.products.is_empty() {
                let (h, o, q) = channel
                    .products
                    .iter()
                    .fold((0, 0, 0), |(h, o, q), &product| {
                        let atoms = atoms(product);
                        (
                            h + atoms.0,
                            o + atoms.1,
                            q + i32::from(species.get(product).charge),
                        )
                    });
                let extra_o = o - 1;
                if !(0..=1).contains(&extra_o) || h - 2 != 2 * extra_o || q != parent_charge {
                    return Err(ChemistryError::invalid(
                        "channel table",
                        format!(
                            "channel {} does not balance against the supplied species",
                            channel.name
                        ),
                    ));
                }
            }
            sum += channel.probability;
        }
        if !sum.is_finite() || (sum - 1.0).abs() > PROBABILITY_SUM_TOLERANCE {
            return Err(ChemistryError::invalid(
                "channel table",
                format!("{} probabilities do not sum to one", state.name),
            ));
        }
    }
    Ok(())
}

fn semantic_sha256(
    species: &SpeciesTable,
    channels: &ChannelTable,
    params: &PhysChemParams,
) -> String {
    fn byte(hash: &mut Sha256, value: u8) {
        hash.update(&[value]);
    }
    fn count(hash: &mut Sha256, value: usize) {
        hash.update(&(value as u64).to_le_bytes());
    }
    fn number(hash: &mut Sha256, value: f64) {
        hash.update(&(if value == 0.0 { 0.0 } else { value }).to_le_bytes());
    }
    fn name(hash: &mut Sha256, value: &str) {
        count(hash, value.len());
        hash.update(value.as_bytes());
    }
    let mut hash = Sha256::new();
    hash.update(b"g4-chemistry/physchem-semantic/v1\0");
    count(&mut hash, Species::ALL.len());
    for props in species.iter() {
        byte(&mut hash, props.species.index() as u8);
        number(&mut hash, props.diffusivity.nm2_per_ps());
        byte(&mut hash, props.charge as u8);
        number(&mut hash, props.radius.nm());
        byte(&mut hash, u8::from(props.reachable_from_physchem));
    }
    count(&mut hash, channels.states().len());
    for state in channels.states() {
        name(&mut hash, &state.name);
        byte(
            &mut hash,
            match state.kind {
                StateKind::Excitation => 0,
                StateKind::Ionisation => 1,
                StateKind::Attachment => 2,
                StateKind::Recombined => 3,
            },
        );
        byte(&mut hash, state.index);
        match state.orbit_vacated {
            Some(orbit) => {
                byte(&mut hash, 1);
                byte(&mut hash, orbit);
            }
            None => byte(&mut hash, 0),
        }
        count(&mut hash, state.channels.len());
        for channel in &state.channels {
            name(&mut hash, &channel.name);
            number(&mut hash, channel.probability);
            number(&mut hash, channel.decay_energy.ev());
            count(&mut hash, channel.products.len());
            for product in &channel.products {
                byte(&mut hash, product.index() as u8);
            }
            byte(
                &mut hash,
                match channel.displacement {
                    DisplacementKind::None => 0,
                    DisplacementKind::A1b1Dissociation => 1,
                    DisplacementKind::B1a1Dissociation => 2,
                    DisplacementKind::B1a1Dissociation2 => 3,
                    DisplacementKind::AutoIonisation => 4,
                    DisplacementKind::IonisationDissociation => 5,
                    DisplacementKind::DissociativeAttachment => 6,
                },
            );
        }
    }
    number(&mut hash, params.birth_time.ps());
    number(&mut hash, params.autoionisation_electron_energy.ev());
    let rec = &params.recombination;
    byte(&mut hash, u8::from(rec.enabled));
    number(&mut hash, rec.temperature_k);
    number(&mut hash, rec.density_g_cm3);
    number(&mut hash, rec.dielectric_constant);
    number(&mut hash, rec.onsager_radius.nm());
    number(&mut hash, rec.search_radius.nm());
    hex(&hash.finalize())
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
    semantic_sha256: String,
    source_file_sha256: Option<String>,
}

impl PhysChemEngine {
    /// The option3 engine with the embedded data.
    pub fn option3() -> Result<Self, ChemistryError> {
        let species = SpeciesTable::option3()?;
        let channels = ChannelTable::option3(&species)?;
        let mut engine = Self::new(species, channels, PhysChemParams::option3()?)?;
        engine.source_file_sha256 = Some(crate::option3_data_sha256());
        Ok(engine)
    }

    pub fn new(
        species: SpeciesTable,
        channels: ChannelTable,
        params: PhysChemParams,
    ) -> Result<Self, ChemistryError> {
        validate_species(&species)?;
        validate_channels(&channels, &species)?;
        params.validate()?;
        let electron_r_mean_nm =
            autoionisation_r_mean_nm(params.autoionisation_electron_energy.ev());
        if !electron_r_mean_nm.is_finite() || electron_r_mean_nm <= 0.0 {
            return Err(ChemistryError::invalid(
                "physchem engine",
                "invalid auto-ionisation displacement scale",
            ));
        }
        let semantic_sha256 = semantic_sha256(&species, &channels, &params);
        Ok(Self {
            species,
            channels,
            params,
            electron_r_mean_nm,
            semantic_sha256,
            source_file_sha256: None,
        })
    }

    /// SHA-256 of the engine's validated semantic values, independent of input text and paths.
    /// Version 1 encodes `g4-chemistry/physchem-semantic/v1\0`, then species in
    /// [`Species::ALL`] order (species index, diffusivity in nm²/ps, charge as i8,
    /// radius in nm, reachable flag), then states and channels in execution order.
    /// Each state encodes name, kind tag, index, optional orbit; each channel encodes
    /// name, probability, decay energy in eV, ordered product indices, displacement tag.
    /// Finally it encodes birth time in ps, auto-ionisation energy in eV, and the
    /// recombination enabled flag, temperature K, density g/cm³, dielectric constant,
    /// Onsager radius nm and search radius nm, in that order. All counts and UTF-8
    /// string lengths are little-endian u64; floats are little-endian IEEE-754 f64,
    /// with negative zero normalized to positive zero; tags and flags are u8.
    pub fn semantic_sha256(&self) -> &str {
        &self.semantic_sha256
    }

    /// SHA-256 of the four embedded option3 source files when constructed by
    /// [`Self::option3`]. A custom engine has no embedded source-file claim.
    pub fn source_file_sha256(&self) -> Option<&str> {
        self.source_file_sha256.as_deref()
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
        self.semantic_sha256 = semantic_sha256(&self.species, &self.channels, &self.params);
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

    /// The whole stage for one history (see the module documentation for the schedule).
    pub fn convert_history<R: Rng + ?Sized>(
        &self,
        events: &[PhysChemEvent],
        rng: &mut R,
    ) -> Result<HistoryChemistry, ChemistryError> {
        self.convert_history_with_observer(events, rng, &mut NoPhysChemObserver)
    }

    /// The same conversion with optional observations; species ordering and RNG draws
    /// are shared with [`Self::convert_history`], including every zero-time pass.
    pub fn convert_history_with_observer<R: Rng + ?Sized, O: PhysChemObserver + ?Sized>(
        &self,
        events: &[PhysChemEvent],
        rng: &mut R,
        observer: &mut O,
    ) -> Result<HistoryChemistry, ChemistryError> {
        let rec = &self.params.recombination;
        let radius_nm = rec.search_radius.nm();
        let mut stats = PhysChemStats::new(&self.channels);
        let mut species: Vec<SpeciesInstance> = Vec::with_capacity(events.len() * 2);
        let mut alive: Vec<bool> = Vec::with_capacity(events.len() * 2);
        let mut grid = ElectronGrid::new(radius_nm);

        // 1. the solvated electrons exist when the chemical stage starts
        for (source_event, ev) in events
            .iter()
            .enumerate()
            .filter(|(_, e)| e.kind == PhysChemKind::Solvation)
        {
            stats.events[3] += 1;
            let id = species.len() as u32;
            species.push(SpeciesInstance {
                species: Species::e_aq,
                position: ev.position,
                time: self.params.birth_time,
            });
            observer.on_solvated_electron(source_event, id as usize, &species[id as usize]);
            alive.push(true);
            grid.insert(id, ev.position);
            stats.species_produced_by_event[3][Species::e_aq.index()] += 1;
        }

        // 2. the water molecules, in Geant4's DoIt order: descending state index (configuration ID),
        //    reverse event order within a state
        let mut pending: Vec<(usize, usize)> = Vec::new(); // (event index, state index)
        for (i, ev) in events.iter().enumerate() {
            if ev.kind == PhysChemKind::Solvation {
                continue;
            }
            pending.push((i, self.channels.state_index(ev.kind, ev.level_or_shell)?));
            observer.on_water(i, ev);
            stats.events[event_kind_index(ev.kind)] += 1;
        }
        pending.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));

        let recombined = self.channels.recombined_index();
        let mut scratch: Vec<SpeciesInstance> = Vec::with_capacity(3);
        let mut born: Vec<SpeciesInstance> = Vec::new();
        let mut nearest_at_start: Vec<Option<f64>> = Vec::with_capacity(pending.len());
        let mut first_pass = true;
        let mut zero_time_pass = 0;
        let mut next_processing_ordinal = 0;
        while !pending.is_empty() {
            // FindReactant of every pending ionised molecule, on the electrons alive at the pass start
            nearest_at_start.clear();
            nearest_at_start.extend(pending.iter().map(|&(i, _)| {
                let ev = &events[i];
                (rec.enabled && ev.kind == PhysChemKind::Ionisation)
                    .then(|| grid.nearest(ev.position, radius_nm, &species, &alive))
                    .flatten()
                    .map(|(_, d)| d)
            }));
            let mut retested = Vec::new();
            for (&(i, state_index), &nearest) in pending.iter().zip(&nearest_at_start) {
                let processing_ordinal = next_processing_ordinal;
                next_processing_ordinal += 1;
                let ev = &events[i];
                let mut si = state_index;
                if ev.kind == PhysChemKind::Ionisation {
                    let u_rec = uniform(rng); // always drawn (docs/reproducibility.md)
                    if first_pass && nearest.is_some() {
                        stats.holes_with_candidate += 1;
                    }
                    if nearest.is_some_and(|d| rec.probability(d) > u_rec) {
                        // MakeReaction: the nearest candidate still alive, with the same uniform
                        match grid.nearest(ev.position, radius_nm, &species, &alive) {
                            Some((id, d)) if rec.probability(d) > u_rec => {
                                alive[id as usize] = false;
                                stats.recombinations += 1;
                                si = recombined;
                            }
                            // stays ionised: tested again in the next pass
                            _ => {
                                retested.push((i, state_index));
                                continue;
                            }
                        }
                    }
                }

                scratch.clear();
                let state = &self.channels.states()[si];
                let (ci, decay_energy) = self.decay(state, ev.position, rng, &mut scratch);
                for (offset, inst) in scratch.iter().enumerate() {
                    observer.on_dissociation_product(
                        DissociationBirth {
                            source_event: i,
                            species_index: species.len() + born.len() + offset,
                            zero_time_pass,
                            processing_ordinal,
                        },
                        inst,
                    );
                }
                stats.channel_counts[si][ci] += 1;
                stats.decay_energy_ev += decay_energy.ev();
                let event_index = event_kind_index(ev.kind);
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
                born.append(&mut scratch);
            }
            // the products join after the pass
            for inst in born.drain(..) {
                if inst.species == Species::e_aq {
                    grid.insert(species.len() as u32, inst.position);
                }
                species.push(inst);
                alive.push(true);
            }
            // Each retest needs an electron killed earlier in its pass, so the number of passes is
            // bounded by the number of e_aq plus one.
            pending = retested;
            first_pass = false;
            zero_time_pass += 1;
        }

        let species: Vec<SpeciesInstance> = species
            .into_iter()
            .zip(alive)
            .enumerate()
            .filter_map(|(i, (s, a))| {
                if a {
                    observer.on_survivor(i, &s);
                    Some(s)
                } else {
                    None
                }
            })
            .collect();
        for s in &species {
            stats.species_counts[s.species.index()] += 1;
        }
        Ok(HistoryChemistry { species, stats })
    }
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    use crate::channels::DEFAULT_DISSOCIATION_TOML;
    use crate::species::DEFAULT_SPECIES_TOML;
    use g4_core::rng::{RngStreams, Stage};

    fn custom(
        species_text: &str,
        channel_text: &str,
        params: PhysChemParams,
    ) -> Result<PhysChemEngine, ChemistryError> {
        let species = SpeciesTable::from_toml(species_text)?;
        let channels = ChannelTable::from_toml(channel_text, &species)?;
        PhysChemEngine::new(species, channels, params)
    }

    #[test]
    fn identity_tracks_all_constructed_inputs_and_source_origin() {
        let default = PhysChemEngine::option3().unwrap();
        let base = custom(
            DEFAULT_SPECIES_TOML,
            DEFAULT_DISSOCIATION_TOML,
            PhysChemParams::option3().unwrap(),
        )
        .unwrap();
        assert_eq!(base.semantic_sha256(), default.semantic_sha256());
        assert_eq!(base.source_file_sha256(), None);
        assert_eq!(
            default.source_file_sha256(),
            Some(crate::option3_data_sha256().as_str())
        );

        let species = DEFAULT_SPECIES_TOML.replacen(
            "van_der_waals_radius_nm = 0.19",
            "van_der_waals_radius_nm = 0.20",
            1,
        );
        let changed_species = custom(
            &species,
            DEFAULT_DISSOCIATION_TOML,
            PhysChemParams::option3().unwrap(),
        )
        .unwrap();
        assert_ne!(base.semantic_sha256(), changed_species.semantic_sha256());

        let channels = DEFAULT_DISSOCIATION_TOML
            .replacen("probability = 0.35", "probability = 0.40", 1)
            .replacen("probability = 0.65", "probability = 0.60", 1);
        let changed_channel = custom(
            DEFAULT_SPECIES_TOML,
            &channels,
            PhysChemParams::option3().unwrap(),
        )
        .unwrap();
        assert_ne!(base.semantic_sha256(), changed_channel.semantic_sha256());

        let mut params = PhysChemParams::option3().unwrap();
        params.birth_time = g4_core::units::Time::from_ps(2.0);
        let changed_param =
            custom(DEFAULT_SPECIES_TOML, DEFAULT_DISSOCIATION_TOML, params).unwrap();
        assert_ne!(base.semantic_sha256(), changed_param.semantic_sha256());

        let disabled = default.clone().with_recombination_enabled(false);
        assert_ne!(default.semantic_sha256(), disabled.semantic_sha256());
        assert_eq!(
            default.semantic_sha256(),
            disabled
                .clone()
                .with_recombination_enabled(true)
                .semantic_sha256()
        );
        assert_eq!(default.source_file_sha256(), disabled.source_file_sha256());
    }

    #[test]
    fn species_input_order_does_not_change_identity() {
        let mut blocks: Vec<_> = DEFAULT_SPECIES_TOML.split("[[species]]").collect();
        let header = blocks.remove(0);
        blocks.reverse();
        let reordered = format!("{header}[[species]]{}", blocks.join("[[species]]"));
        let base = PhysChemEngine::option3().unwrap();
        let permuted = custom(
            &reordered,
            DEFAULT_DISSOCIATION_TOML,
            PhysChemParams::option3().unwrap(),
        )
        .unwrap();
        assert_eq!(base.semantic_sha256(), permuted.semantic_sha256());
    }

    #[test]
    fn invalid_custom_values_are_rejected_and_default_rng_is_unchanged() {
        let species = SpeciesTable::option3().unwrap();
        let channels = ChannelTable::option3(&species).unwrap();
        let mut params = PhysChemParams::option3().unwrap();
        params.recombination.search_radius = g4_core::units::Length::from_nm(f64::NAN);
        assert!(matches!(
            PhysChemEngine::new(species.clone(), channels.clone(), params),
            Err(ChemistryError::Invalid { .. })
        ));

        let invalid_species = DEFAULT_SPECIES_TOML.replacen(
            "diffusion_m2_per_s = 7.0e-9",
            "diffusion_m2_per_s = nan",
            1,
        );
        let invalid_species = SpeciesTable::from_toml(&invalid_species).unwrap();
        assert!(matches!(
            PhysChemEngine::new(
                invalid_species,
                channels.clone(),
                PhysChemParams::option3().unwrap()
            ),
            Err(ChemistryError::Invalid { .. })
        ));

        let changed_charge = DEFAULT_SPECIES_TOML.replacen("charge = 1", "charge = 2", 1);
        let changed_charge = SpeciesTable::from_toml(&changed_charge).unwrap();
        assert!(matches!(
            PhysChemEngine::new(changed_charge, channels, PhysChemParams::option3().unwrap()),
            Err(ChemistryError::Invalid { .. })
        ));

        let default = PhysChemEngine::option3().unwrap();
        let equivalent = custom(
            DEFAULT_SPECIES_TOML,
            DEFAULT_DISSOCIATION_TOML,
            PhysChemParams::option3().unwrap(),
        )
        .unwrap();
        let event = PhysChemEvent {
            kind: PhysChemKind::Excitation,
            level_or_shell: 0,
            position: Vec3::new(0.0, 0.0, 0.0),
            deposited_energy: Energy::from_ev(8.0),
            parent_history: 0,
        };
        let streams = RngStreams::new(123);
        let mut first = streams.stream(0, Stage::PhysChem).unwrap();
        let mut second = streams.stream(0, Stage::PhysChem).unwrap();
        assert_eq!(
            default.convert_history(&[event], &mut first).unwrap(),
            equivalent.convert_history(&[event], &mut second).unwrap()
        );
        assert_eq!(first.next_u64(), second.next_u64());
    }
}
