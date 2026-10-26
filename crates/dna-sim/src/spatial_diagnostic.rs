//! Diagnostic-only spatial records and inference for the M6 900 keV investigation.
//!
//! The preregistered contract is in `docs/09-open-questions-and-risks.md`. Positions
//! are relative to the primary origin, whose direction is +z; nothing is recentered.
//! Boundary 4 counts births across interleaved zero-time passes, whereas boundary 5
//! counts survivors. They must never be added together. These types do not change
//! the normal run JSON, transport, chemistry, or the existing M6 acceptance gates.
//!
//! The CSV has one row per entity per boundary (`count = 1`), or one explicit empty
//! row for a category with no entities (`count = 0`). IDs are local to an
//! implementation/seed/history, never cross-implementation matches. Physical-site
//! entity IDs are event ordinals, not track IDs; molecular IDs may be negative
//! Geant4 IDs or stable Rust indices. Optional fields use exactly `NA`. Empty rows
//! use a zero maximum sentinel. Category keys use the canonical chemistry names.
//! Histories must be contiguous in a file, but their blocks may be in any order.
//! Parsing retains only the current history's identity set and completed moments.
//!
//! Lifecycle sources, Geant4 11.4.1 (under `source/processes/electromagnetic/dna`):
//! `utils/src/G4DNAChemistryManager.cc::CreateWaterMolecule/CreateSolvatedElectron`,
//! `processes/src/G4DNAMolecularDissociation.cc::DecayIt`, and
//! `processes/src/G4DNAElectronHoleRecombination.cc::MakeReaction`. The source audit
//! and limitations of the physical-to-molecular ordering correspondence are in
//! `docs/02-cpp-reference-inventory.md`, B.7.9.

use dna_chemistry::engine::{DissociationBirth, PhysChemObserver};
use dna_core::physchem::{PhysChemEvent, PhysChemKind, PhysChemSite};
use dna_core::rng::{HistoryRng, RngStreams, Stage};
use dna_core::species::{Species, SpeciesInstance};
use dna_core::vec3::Vec3;
use dna_physics::transport::HistoryObserver;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, Write};
use std::str::FromStr;
use thiserror::Error;

/// Frozen independent production seed labels, used separately in each implementation.
pub const DIAGNOSTIC_SEEDS: [u64; 5] = [20261021, 20261022, 20261023, 20261024, 20261025];
/// Frozen seed of the diagnostic bootstrap, unrelated to simulation streams.
pub const ANALYSIS_SEED: u64 = 20261026;
/// Exactly this many replicates; no adaptive truncation or invalid-replicate filtering.
pub const BOOTSTRAP_REPLICATES: usize = 20_000;
/// The fixed seven species pooled at the final, pre-IRT boundary.
pub const SHARED_SPECIES: [Species; 7] = [
    Species::e_aq,
    Species::OH,
    Species::H3Op,
    Species::H,
    Species::H2,
    Species::OHm,
    Species::Oxy,
];
/// Underflow, sixteen half-decade bins, and overflow.
pub const RADIAL_BIN_COUNT: usize = 18;

/// A malformed/incomplete diagnostic, an I/O failure, or an undefined estimand.
#[derive(Debug, Error)]
pub enum DiagnosticError {
    #[error("invalid spatial diagnostic: {0}")]
    Invalid(String),
    #[error("spatial diagnostic I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("undefined RMS ratio at {boundary:?} ({context}); no interval was constructed")]
    UndefinedRatio { boundary: Boundary, context: String },
}

fn invalid(message: impl Into<String>) -> DiagnosticError {
    DiagnosticError::Invalid(message.into())
}

macro_rules! named_enum {
    ($(#[$meta:meta])* pub enum $name:ident { $($(#[$vmeta:meta])* $variant:ident => $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name { $($(#[$vmeta])* $variant),+ }
        impl $name {
            /// Stable CSV spelling.
            pub const fn name(self) -> &'static str {
                match self { $(Self::$variant => $text),+ }
            }
        }
        impl FromStr for $name {
            type Err = DiagnosticError;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($text => Ok(Self::$variant),)+
                    _ => Err(invalid(format!("unknown {} '{value}'", stringify!($name)))),
                }
            }
        }
    };
}

named_enum! {
    /// Fixed reporting order, not five consecutive global snapshots.
    pub enum Boundary {
        /// Actual excitation/ionisation/attachment sites and pre-displacement solvation origins.
        PhysicalSites => "physical_sites",
        /// Direct terminal-electron births, excluding auto-ionisation daughters.
        SolvatedElectronCreation => "solvated_electron_creation",
        /// Initial water configurations before any decay or recombination.
        PreDecayWater => "pre_decay_water",
        /// Actual daughter births, including recombined-water daughters.
        DissociationProducts => "dissociation_products",
        /// Alive shared species after all passes, before IRT or diffusion.
        FinalSurvivors => "final_survivors"
    }
}

impl Boundary {
    pub const ALL: [Self; 5] = [
        Self::PhysicalSites,
        Self::SolvatedElectronCreation,
        Self::PreDecayWater,
        Self::DissociationProducts,
        Self::FinalSurvivors,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    /// Every category requires either entity rows or exactly one explicit empty row.
    pub fn categories(self) -> Vec<Category> {
        match self {
            Self::PhysicalSites | Self::PreDecayWater => {
                let mut result: Vec<_> = (0..5)
                    .map(Category::Excitation)
                    .chain((0..5).map(Category::Ionisation))
                    .chain([Category::Attachment])
                    .collect();
                if self == Self::PhysicalSites {
                    result.push(Category::Solvation);
                }
                result
            }
            Self::SolvatedElectronCreation => vec![Category::Species(Species::e_aq)],
            Self::DissociationProducts | Self::FinalSurvivors => {
                SHARED_SPECIES.map(Category::Species).to_vec()
            }
        }
    }

    pub const fn stage(self) -> DiagnosticStage {
        match self {
            Self::PhysicalSites => DiagnosticStage::Physical,
            _ => DiagnosticStage::Physicochemical,
        }
    }
}

named_enum! {
    /// Independent implementation samples; matching numeric labels imply no pairing.
    pub enum Implementation { Rust => "rust", Geant4 => "geant4" }
}
named_enum! {
    /// Physical handoffs retain their stage meaning; molecular records are at 1 ps.
    pub enum DiagnosticStage { Physical => "physical", Physicochemical => "physicochemical" }
}
named_enum! {
    /// Distinguishes a site ordinal from a physical or molecular track/entity ID.
    pub enum Namespace { Event => "event", Physical => "physical", Molecular => "molecular" }
}
named_enum! {
    /// `Stopped` means extant (`fStopButAlive`), not killed.
    pub enum LifeState {
        Alive => "alive", Stopped => "stopped", Killed => "killed",
        /// Only empty-category rows have no observed state.
        Missing => "NA"
    }
}

/// Physical level/shell indices are 0..4, as in `PhysChemEvent`; not reversed orbit IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    Excitation(u8),
    Ionisation(u8),
    Attachment,
    Solvation,
    Species(Species),
}

impl Category {
    /// CSV key. Water states use their original process and physical level/shell.
    pub fn name(self) -> String {
        match self {
            Self::Excitation(i) => format!("excitation_{i}"),
            Self::Ionisation(i) => format!("ionisation_{i}"),
            Self::Attachment => "attachment".into(),
            Self::Solvation => "solvation".into(),
            Self::Species(species) => species.chemistry_name().into(),
        }
    }
}

impl FromStr for Category {
    type Err = DiagnosticError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Boundary::ALL
            .into_iter()
            .flat_map(Boundary::categories)
            .find(|category| category.name() == value)
            .ok_or_else(|| invalid(format!("unknown category '{value}'")))
    }
}

/// Identifier within one implementation, seed, and history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId {
    pub namespace: Namespace,
    pub id: i64,
}

/// Stable lineage; missing information must remain `None`, never an invented match.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Identity {
    pub entity: Option<EntityId>,
    /// Incoming physical track for a physical site, if tracked by the implementation.
    pub physical_track_id: Option<i64>,
    pub parent: Option<EntityId>,
    pub source_event: Option<u64>,
    pub creation_ordinal: Option<u64>,
    /// Birth/processing pass metadata when applicable; both fields must be present together.
    pub zero_time_pass: Option<u64>,
    pub processing_ordinal: Option<u64>,
}

/// Receiver of complete history blocks in deterministic run order.
pub trait SpatialDiagnosticSink: Send {
    fn write_history(&mut self, records: &[DiagnosticRecord]) -> Result<(), DiagnosticError>;
}

/// Streaming long-form CSV; one header even when several seed runs share a writer.
pub struct CsvSpatialDiagnosticWriter<W: Write> {
    writer: W,
}

impl<W: Write> CsvSpatialDiagnosticWriter<W> {
    pub fn new(mut writer: W) -> Result<Self, DiagnosticError> {
        writeln!(writer, "{}", csv_header())?;
        Ok(Self { writer })
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W: Write + Send> SpatialDiagnosticSink for CsvSpatialDiagnosticWriter<W> {
    fn write_history(&mut self, records: &[DiagnosticRecord]) -> Result<(), DiagnosticError> {
        for record in records {
            writeln!(self.writer, "{}", record.csv_row()?)?;
        }
        Ok(())
    }
}

fn event_category(event: &PhysChemEvent) -> Category {
    match event.kind {
        PhysChemKind::Excitation => Category::Excitation(event.level_or_shell),
        PhysChemKind::Ionisation => Category::Ionisation(event.level_or_shell),
        PhysChemKind::Attachment => Category::Attachment,
        PhysChemKind::Solvation => Category::Solvation,
    }
}

/// Observer for a single transport followed by its physicochemical conversion.
///
/// Physical IDs are Rust creation-order track IDs (primary 0). Initial molecular
/// IDs/creation ordinals are source-event indices; daughter IDs follow all initial
/// entities in actual decay birth order. This gives one molecular namespace without
/// pretending Rust species slots or creation order equal Geant4 IT track IDs.
/// Survivor rows retain the *birth* identity/pass, even after dead slots are removed.
/// Positions assume the run's common primary origin (zero) and direction (+z).
/// Observation errors are deferred to [`Self::finish`], never fed into model draws.
pub struct RustSpatialObserver {
    seed: u64,
    history: u64,
    sites: Vec<PhysChemSite>,
    species_identities: Vec<Identity>,
    next_daughter: u64,
    records: Vec<DiagnosticRecord>,
    error: Option<DiagnosticError>,
}

impl RustSpatialObserver {
    pub fn new(seed: u64, history: u64) -> Self {
        Self {
            seed,
            history,
            sites: Vec::new(),
            species_identities: Vec::new(),
            next_daughter: 0,
            records: Vec::new(),
            error: None,
        }
    }

    fn record(
        &mut self,
        boundary: Boundary,
        category: Category,
        identity: Identity,
        position: Vec3,
        state: LifeState,
    ) {
        if self.error.is_some() {
            return;
        }
        let result = SpatialMoments::from_position(position).and_then(|moments| {
            let record = DiagnosticRecord {
                implementation: Implementation::Rust,
                seed: self.seed,
                history: self.history,
                boundary,
                category,
                identity,
                stage: boundary.stage(),
                time_ps: (boundary != Boundary::PhysicalSites).then_some(1.0),
                state,
                moments,
            };
            record.validate()?;
            Ok(record)
        });
        match result {
            Ok(record) => self.records.push(record),
            Err(error) => self.error = Some(error),
        }
    }

    fn initial_identity(&mut self, source_event: usize) -> Option<Identity> {
        let Some(site) = self.sites.get(source_event) else {
            self.error = Some(invalid("molecular observation has no physical source site"));
            return None;
        };
        Some(Identity {
            entity: Some(EntityId {
                namespace: Namespace::Molecular,
                id: source_event as i64,
            }),
            physical_track_id: Some(site.physical_track_id as i64),
            parent: Some(EntityId {
                namespace: Namespace::Physical,
                id: site.physical_track_id as i64,
            }),
            source_event: Some(source_event as u64),
            creation_ordinal: Some(source_event as u64),
            ..Identity::default()
        })
    }

    fn retain_birth(&mut self, index: usize, identity: Identity) {
        if index != self.species_identities.len() {
            self.error = Some(invalid("noncontiguous species birth slots"));
            return;
        }
        self.species_identities.push(identity);
    }

    /// Finish one history and supply every absent category explicitly.
    pub fn finish(mut self) -> Result<Vec<DiagnosticRecord>, DiagnosticError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        let populated: BTreeSet<_> = self
            .records
            .iter()
            .map(|r| (r.boundary, r.category))
            .collect();
        for boundary in Boundary::ALL {
            for category in boundary.categories() {
                if !populated.contains(&(boundary, category)) {
                    self.records.push(DiagnosticRecord::empty(
                        Implementation::Rust,
                        self.seed,
                        self.history,
                        boundary,
                        category,
                    ));
                }
            }
        }
        Ok(self.records)
    }
}

impl HistoryObserver for RustSpatialObserver {
    fn on_physchem_site(&mut self, site: &PhysChemSite) {
        if site.event.parent_history != self.history || site.source_event != self.sites.len() as u64
        {
            self.error = Some(invalid("physical source history/ordinal mismatch"));
            return;
        }
        self.sites.push(*site);
        let identity = Identity {
            entity: Some(EntityId {
                namespace: Namespace::Event,
                id: site.source_event as i64,
            }),
            physical_track_id: Some(site.physical_track_id as i64),
            parent: site.parent_track_id.map(|id| EntityId {
                namespace: Namespace::Physical,
                id: id as i64,
            }),
            source_event: Some(site.source_event),
            creation_ordinal: Some(site.source_event),
            ..Identity::default()
        };
        let state = match site.event.kind {
            PhysChemKind::Solvation | PhysChemKind::Attachment => LifeState::Killed,
            _ => LifeState::Alive,
        };
        self.record(
            Boundary::PhysicalSites,
            event_category(&site.event),
            identity,
            site.origin,
            state,
        );
    }
}

impl PhysChemObserver for RustSpatialObserver {
    fn on_solvated_electron(
        &mut self,
        source_event: usize,
        species_index: usize,
        species: &SpeciesInstance,
    ) {
        let Some(identity) = self.initial_identity(source_event) else {
            return;
        };
        self.retain_birth(species_index, identity);
        self.record(
            Boundary::SolvatedElectronCreation,
            Category::Species(species.species),
            identity,
            species.position,
            LifeState::Alive,
        );
    }

    fn on_water(&mut self, source_event: usize, event: &PhysChemEvent) {
        let Some(identity) = self.initial_identity(source_event) else {
            return;
        };
        self.record(
            Boundary::PreDecayWater,
            event_category(event),
            identity,
            event.position,
            LifeState::Stopped,
        );
    }

    fn on_dissociation_product(&mut self, birth: DissociationBirth, species: &SpeciesInstance) {
        let Some(mut identity) = self.initial_identity(birth.source_event) else {
            return;
        };
        identity.parent = identity.entity;
        let ordinal = self.sites.len() as u64 + self.next_daughter;
        self.next_daughter += 1;
        identity.entity = Some(EntityId {
            namespace: Namespace::Molecular,
            id: ordinal as i64,
        });
        identity.creation_ordinal = Some(ordinal);
        identity.zero_time_pass = Some(birth.zero_time_pass);
        identity.processing_ordinal = Some(birth.processing_ordinal);
        self.retain_birth(birth.species_index, identity);
        self.record(
            Boundary::DissociationProducts,
            Category::Species(species.species),
            identity,
            species.position,
            LifeState::Alive,
        );
    }

    fn on_survivor(&mut self, species_index: usize, species: &SpeciesInstance) {
        let Some(&identity) = self.species_identities.get(species_index) else {
            self.error = Some(invalid("survivor has no observed birth"));
            return;
        };
        self.record(
            Boundary::FinalSurvivors,
            Category::Species(species.species),
            identity,
            species.position,
            LifeState::Alive,
        );
    }
}

/// The 17 edges `10^(k/2)` nm, `k = 0..16`.
pub fn radial_edges_nm() -> [f64; 17] {
    std::array::from_fn(|k| 10.0_f64.powf(k as f64 / 2.0))
}

/// Half-open bins: an exact edge is in the next bin, including overflow at 1e8 nm.
pub fn radial_bin(radius_nm: f64) -> Result<usize, DiagnosticError> {
    if !radius_nm.is_finite() || radius_nm < 0.0 {
        return Err(invalid("radius must be finite and nonnegative"));
    }
    Ok(radial_edges_nm().partition_point(|edge| *edge <= radius_nm))
}

/// Additive spatial moments. Maximum 0 is the documented sentinel for an empty set.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SpatialMoments {
    pub count: u64,
    pub sum_r2_nm2: f64,
    pub sum_z_nm: f64,
    pub sum_z2_nm2: f64,
    pub sum_x2_plus_y2_nm2: f64,
    pub max_radius_nm: f64,
    pub radial_bins: [u64; RADIAL_BIN_COUNT],
}

fn close(a: f64, b: f64) -> bool {
    a.is_finite()
        && b.is_finite()
        && (a - b).abs() <= 64.0 * f64::EPSILON * a.abs().max(b.abs()).max(f64::MIN_POSITIVE)
}

// Independent running sums can differ by more than a fixed number of ulps after
// a long track. Use a conservative gamma_n forward-error bound for accumulated
// identities, while single-entity checks retain the tighter fixed roundoff bound.
fn close_accumulated(a: f64, b: f64, count: u64) -> bool {
    if close(a, b) {
        return true;
    }
    let n_epsilon = count as f64 * f64::EPSILON;
    a.is_finite()
        && b.is_finite()
        && n_epsilon < 1.0
        && (a - b).abs() / a.abs().max(b.abs()).max(f64::MIN_POSITIVE)
            <= 8.0 * n_epsilon / (1.0 - n_epsilon)
}

impl SpatialMoments {
    /// One point in nm relative to the primary origin; rejects overflow and nonfinite input.
    pub fn from_position(position: Vec3) -> Result<Self, DiagnosticError> {
        let transverse = position.x * position.x + position.y * position.y;
        let z2 = position.z * position.z;
        let r2 = z2 + transverse;
        let mut result = Self {
            count: 1,
            sum_r2_nm2: r2,
            sum_z_nm: position.z,
            sum_z2_nm2: z2,
            sum_x2_plus_y2_nm2: transverse,
            max_radius_nm: r2.sqrt(),
            radial_bins: [0; RADIAL_BIN_COUNT],
        };
        result.radial_bins[radial_bin(result.max_radius_nm)?] = 1;
        result.validate()?;
        Ok(result)
    }

    /// Add moments without changing the receiver on invalid input or arithmetic overflow.
    pub fn merge(&mut self, other: &Self) -> Result<(), DiagnosticError> {
        self.validate()?;
        other.validate()?;
        let mut sum = *self;
        sum.count = sum
            .count
            .checked_add(other.count)
            .ok_or_else(|| invalid("count overflow"))?;
        sum.sum_r2_nm2 += other.sum_r2_nm2;
        sum.sum_z_nm += other.sum_z_nm;
        sum.sum_z2_nm2 += other.sum_z2_nm2;
        sum.sum_x2_plus_y2_nm2 += other.sum_x2_plus_y2_nm2;
        sum.max_radius_nm = sum.max_radius_nm.max(other.max_radius_nm);
        for (target, value) in sum.radial_bins.iter_mut().zip(other.radial_bins) {
            *target = target
                .checked_add(value)
                .ok_or_else(|| invalid("bin overflow"))?;
        }
        sum.validate()?;
        *self = sum;
        Ok(())
    }

    /// Validate finite values, moment consistency, histogram totals, and empty semantics.
    pub fn validate(&self) -> Result<(), DiagnosticError> {
        let nonnegative = [
            self.sum_r2_nm2,
            self.sum_z2_nm2,
            self.sum_x2_plus_y2_nm2,
            self.max_radius_nm,
        ];
        if nonnegative.iter().any(|x| !x.is_finite() || *x < 0.0) || !self.sum_z_nm.is_finite() {
            return Err(invalid(
                "moments must be finite and squared moments nonnegative",
            ));
        }
        let bin_total = self
            .radial_bins
            .iter()
            .try_fold(0_u64, |sum, n| sum.checked_add(*n));
        if bin_total != Some(self.count) {
            return Err(invalid("radial-bin total differs from count"));
        }
        let parts = self.sum_z2_nm2 + self.sum_x2_plus_y2_nm2;
        if !parts.is_finite() || !close_accumulated(self.sum_r2_nm2, parts, self.count) {
            return Err(invalid(
                "sum_r2 differs from longitudinal plus transverse moments",
            ));
        }
        if self.count == 0 {
            if nonnegative.iter().any(|x| *x != 0.0) || self.sum_z_nm != 0.0 {
                return Err(invalid(
                    "empty rows must have zero moments and zero maximum",
                ));
            }
        } else {
            let mean_r2 = self.sum_r2_nm2 / self.count as f64;
            let maximum2 = self.max_radius_nm * self.max_radius_nm;
            let mean_z = self.sum_z_nm / self.count as f64;
            let mean_z2 = self.sum_z2_nm2 / self.count as f64;
            if !maximum2.is_finite()
                || (mean_r2 > maximum2 && !close_accumulated(mean_r2, maximum2, self.count))
                || (maximum2 > self.sum_r2_nm2
                    && !close_accumulated(maximum2, self.sum_r2_nm2, self.count))
                || (mean_z * mean_z > mean_z2
                    && !close_accumulated(mean_z * mean_z, mean_z2, self.count))
            {
                return Err(invalid("maximum or longitudinal moments are inconsistent"));
            }
            let last = self.radial_bins.iter().rposition(|n| *n > 0).unwrap();
            if last != radial_bin(self.max_radius_nm)? {
                return Err(invalid(
                    "maximum radius disagrees with occupied radial bins",
                ));
            }
        }
        Ok(())
    }

    /// Particle-pooled RMS; an empty population is undefined, a point at the origin is zero.
    pub fn rms_radius_nm(&self) -> Option<f64> {
        (self.count > 0).then(|| (self.sum_r2_nm2 / self.count as f64).sqrt())
    }

    /// Bin containing the nearest-rank quantile. No interpolation inside a bin is claimed.
    /// `upper_nm = None` denotes unbounded overflow; empty populations return `None`.
    pub fn radial_quantile(
        &self,
        probability: f64,
    ) -> Result<Option<RadialBounds>, DiagnosticError> {
        self.validate()?;
        if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
            return Err(invalid("quantile probability must be in [0, 1]"));
        }
        if self.count == 0 {
            return Ok(None);
        }
        let rank = (probability * self.count as f64).ceil().max(1.0) as u64;
        let mut cumulative = 0;
        for (bin, count) in self.radial_bins.iter().enumerate() {
            cumulative += count;
            if cumulative >= rank {
                let edges = radial_edges_nm();
                return Ok(Some(RadialBounds {
                    lower_nm: if bin == 0 { 0.0 } else { edges[bin - 1] },
                    upper_nm: edges.get(bin).copied(),
                }));
            }
        }
        Err(invalid("quantile rank exceeds histogram"))
    }
}

/// Half-open radial bounds, not an exact point estimate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadialBounds {
    pub lower_nm: f64,
    pub upper_nm: Option<f64>,
}

/// One entity observation, or an explicit empty category.
#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosticRecord {
    pub implementation: Implementation,
    pub seed: u64,
    pub history: u64,
    pub boundary: Boundary,
    pub category: Category,
    pub identity: Identity,
    pub stage: DiagnosticStage,
    /// Physical flight time may be unavailable; molecular observations are exactly 1 ps.
    pub time_ps: Option<f64>,
    pub state: LifeState,
    pub moments: SpatialMoments,
}

impl DiagnosticRecord {
    /// Explicit zero population; all entity-specific fields are missing.
    pub fn empty(
        implementation: Implementation,
        seed: u64,
        history: u64,
        boundary: Boundary,
        category: Category,
    ) -> Self {
        Self {
            implementation,
            seed,
            history,
            boundary,
            category,
            identity: Identity::default(),
            stage: boundary.stage(),
            time_ps: None,
            state: LifeState::Missing,
            moments: SpatialMoments::default(),
        }
    }

    pub fn validate(&self) -> Result<(), DiagnosticError> {
        self.moments.validate()?;
        if !self.boundary.categories().contains(&self.category) {
            return Err(invalid(
                "category is not in the boundary's fixed population",
            ));
        }
        if self.stage != self.boundary.stage() {
            return Err(invalid("stage disagrees with boundary"));
        }
        if self.moments.count == 0 {
            if self.identity != Identity::default()
                || self.time_ps.is_some()
                || self.state != LifeState::Missing
            {
                return Err(invalid("empty category has entity-specific metadata"));
            }
            return Ok(());
        }
        if self.moments.count != 1 {
            return Err(invalid(
                "entity rows must have count 1; aggregate only after validation",
            ));
        }
        let id = self
            .identity
            .entity
            .ok_or_else(|| invalid("missing entity ID"))?;
        let namespace = if self.boundary == Boundary::PhysicalSites {
            Namespace::Event
        } else {
            Namespace::Molecular
        };
        if id.namespace != namespace
            || self.identity.source_event.is_none()
            || self.identity.creation_ordinal.is_none()
        {
            return Err(invalid(
                "missing source/creation ordinal or wrong entity namespace",
            ));
        }
        if self.identity.zero_time_pass.is_some() != self.identity.processing_ordinal.is_some() {
            return Err(invalid(
                "pass and processing ordinal must be present together",
            ));
        }
        if self.boundary == Boundary::DissociationProducts && self.identity.zero_time_pass.is_none()
        {
            return Err(invalid("daughter birth is missing its zero-time pass"));
        }
        if let Some(time) = self.time_ps
            && (!time.is_finite() || time < 0.0)
        {
            return Err(invalid("time must be finite and nonnegative"));
        }
        if self.stage == DiagnosticStage::Physicochemical && self.time_ps != Some(1.0) {
            return Err(invalid("molecular boundary must be at global time 1 ps"));
        }
        let valid_state = match self.boundary {
            Boundary::PhysicalSites => self.state != LifeState::Missing,
            Boundary::PreDecayWater => self.state == LifeState::Stopped,
            _ => self.state == LifeState::Alive,
        };
        if !valid_state {
            return Err(invalid("life state disagrees with boundary population"));
        }
        if !close(self.moments.sum_r2_nm2, self.moments.max_radius_nm.powi(2))
            || !close(self.moments.sum_z2_nm2, self.moments.sum_z_nm.powi(2))
        {
            return Err(invalid("entity row does not describe a single position"));
        }
        Ok(())
    }

    /// Shortest round-trip floats, no quoting or whitespace, no trailing newline.
    pub fn csv_row(&self) -> Result<String, DiagnosticError> {
        self.validate()?;
        let entity = self.identity.entity;
        let parent = self.identity.parent;
        let mut fields = vec![
            self.implementation.name().into(),
            self.seed.to_string(),
            self.history.to_string(),
            self.boundary.name().into(),
            self.category.name(),
            optional(entity.map(|x| x.namespace.name())),
            optional(entity.map(|x| x.id)),
            optional(self.identity.physical_track_id),
            optional(parent.map(|x| x.namespace.name())),
            optional(parent.map(|x| x.id)),
            optional(self.identity.source_event),
            optional(self.identity.creation_ordinal),
            optional(self.identity.zero_time_pass),
            optional(self.identity.processing_ordinal),
            self.stage.name().into(),
            optional(self.time_ps),
            self.state.name().into(),
            self.moments.count.to_string(),
            self.moments.sum_r2_nm2.to_string(),
            self.moments.sum_z_nm.to_string(),
            self.moments.sum_z2_nm2.to_string(),
            self.moments.sum_x2_plus_y2_nm2.to_string(),
            self.moments.max_radius_nm.to_string(),
        ];
        fields.extend(self.moments.radial_bins.map(|n| n.to_string()));
        Ok(fields.join(","))
    }

    /// Strict schema parser. Only the literal `NA` represents a missing optional field.
    pub fn from_csv_row(line: &str) -> Result<Self, DiagnosticError> {
        let f: Vec<_> = line.split(',').collect();
        if f.len() != 23 + RADIAL_BIN_COUNT || f.iter().any(|s| s.is_empty() || s.trim() != *s) {
            return Err(invalid(
                "wrong CSV column count, empty field, or surrounding whitespace",
            ));
        }
        let entity = parse_entity(f[5], f[6])?;
        let parent = parse_entity(f[8], f[9])?;
        let mut bins = [0; RADIAL_BIN_COUNT];
        for (target, value) in bins.iter_mut().zip(&f[23..]) {
            *target = parse(value)?;
        }
        let row = Self {
            implementation: parse(f[0])?,
            seed: parse(f[1])?,
            history: parse(f[2])?,
            boundary: parse(f[3])?,
            category: parse(f[4])?,
            identity: Identity {
                entity,
                physical_track_id: parse_optional(f[7])?,
                parent,
                source_event: parse_optional(f[10])?,
                creation_ordinal: parse_optional(f[11])?,
                zero_time_pass: parse_optional(f[12])?,
                processing_ordinal: parse_optional(f[13])?,
            },
            stage: parse(f[14])?,
            time_ps: parse_optional(f[15])?,
            state: parse(f[16])?,
            moments: SpatialMoments {
                count: parse(f[17])?,
                sum_r2_nm2: parse(f[18])?,
                sum_z_nm: parse(f[19])?,
                sum_z2_nm2: parse(f[20])?,
                sum_x2_plus_y2_nm2: parse(f[21])?,
                max_radius_nm: parse(f[22])?,
                radial_bins: bins,
            },
        };
        row.validate()?;
        Ok(row)
    }
}

fn optional(value: Option<impl ToString>) -> String {
    value.map_or_else(|| "NA".into(), |x| x.to_string())
}

fn parse<T: FromStr>(value: &str) -> Result<T, DiagnosticError> {
    value
        .parse()
        .map_err(|_| invalid(format!("invalid field '{value}'")))
}

fn parse_optional<T: FromStr>(value: &str) -> Result<Option<T>, DiagnosticError> {
    if value == "NA" {
        Ok(None)
    } else {
        parse(value).map(Some)
    }
}

fn parse_entity(namespace: &str, id: &str) -> Result<Option<EntityId>, DiagnosticError> {
    match (parse_optional(namespace)?, parse_optional(id)?) {
        (None, None) => Ok(None),
        (Some(namespace), Some(id)) => Ok(Some(EntityId { namespace, id })),
        _ => Err(invalid("ID and its namespace must be present together")),
    }
}

/// Exact header, including bin indices 0 (underflow) through 17 (overflow).
pub fn csv_header() -> String {
    let mut header = "implementation,seed,history,boundary,category,entity_namespace,entity_id,physical_track_id,parent_namespace,parent_id,source_event,creation_ordinal,zero_time_pass,processing_ordinal,stage,time_ps,state,count,sum_r2_nm2,sum_z_nm,sum_z2_nm2,sum_x2_plus_y2_nm2,max_radius_nm".to_string();
    for i in 0..RADIAL_BIN_COUNT {
        header.push_str(&format!(",radial_bin_{i}"));
    }
    header
}

/// Write validated rows. Dataset-level completeness is checked by [`DiagnosticDataset`].
pub fn write_csv<'a>(
    mut writer: impl Write,
    records: impl IntoIterator<Item = &'a DiagnosticRecord>,
) -> Result<(), DiagnosticError> {
    writeln!(writer, "{}", csv_header())?;
    for record in records {
        writeln!(writer, "{}", record.csv_row()?)?;
    }
    Ok(())
}

/// Expected coverage supplied independently of the rows, so a missing whole history/seed fails.
/// Unequal small groups are supported for synthetic tests; production uses five equal groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SamplePlan {
    groups: BTreeMap<u64, Vec<u64>>,
}

impl SamplePlan {
    pub fn new(mut groups: BTreeMap<u64, Vec<u64>>) -> Result<Self, DiagnosticError> {
        if groups.is_empty() || groups.values().any(Vec::is_empty) {
            return Err(invalid(
                "sample plan requires nonempty seed groups and histories",
            ));
        }
        for histories in groups.values_mut() {
            histories.sort_unstable();
            if histories.windows(2).any(|pair| pair[0] == pair[1]) {
                return Err(invalid("duplicate history in sample plan"));
            }
        }
        Ok(Self { groups })
    }

    /// Histories `0..histories_per_seed` for each distinct seed.
    pub fn uniform(seeds: &[u64], histories_per_seed: u64) -> Result<Self, DiagnosticError> {
        if seeds.iter().copied().collect::<BTreeSet<_>>().len() != seeds.len() {
            return Err(invalid("duplicate seed in sample plan"));
        }
        Self::new(
            seeds
                .iter()
                .map(|seed| (*seed, (0..histories_per_seed).collect()))
                .collect(),
        )
    }

    /// The preregistered production levels, without changing the existing M6 gate.
    pub fn production(histories_per_seed: u64) -> Result<Self, DiagnosticError> {
        if ![40, 80, 160].contains(&histories_per_seed) {
            return Err(invalid(
                "production sample must have 40, 80, or 160 histories per seed",
            ));
        }
        Self::uniform(&DIAGNOSTIC_SEEDS, histories_per_seed)
    }

    pub fn groups(&self) -> &BTreeMap<u64, Vec<u64>> {
        &self.groups
    }

    pub fn history_count(&self) -> usize {
        self.groups.values().map(Vec::len).sum()
    }

    fn contains(&self, seed: u64, history: u64) -> bool {
        self.groups
            .get(&seed)
            .is_some_and(|ids| ids.binary_search(&history).is_ok())
    }
}

/// Complete reduced history. Every boundary/category exists, including empty populations.
#[derive(Debug, Clone, PartialEq)]
pub struct HistorySummary {
    pub seed: u64,
    pub history: u64,
    pub boundaries: [SpatialMoments; 5],
    pub categories: BTreeMap<(Boundary, Category), SpatialMoments>,
}

struct HistoryBuilder {
    summary: HistorySummary,
    identities: BTreeSet<(Boundary, EntityId)>,
}

impl HistoryBuilder {
    fn new(seed: u64, history: u64) -> Self {
        Self {
            summary: HistorySummary {
                seed,
                history,
                boundaries: [SpatialMoments::default(); 5],
                categories: BTreeMap::new(),
            },
            identities: BTreeSet::new(),
        }
    }

    fn add(&mut self, row: DiagnosticRecord) -> Result<(), DiagnosticError> {
        let key = (row.boundary, row.category);
        if let Some(id) = row.identity.entity
            && !self.identities.insert((row.boundary, id))
        {
            return Err(invalid(format!(
                "duplicate entity at {:?}: {id:?}",
                row.boundary
            )));
        }
        if let Some(existing) = self.summary.categories.get(&key)
            && (existing.count == 0 || row.moments.count == 0)
        {
            return Err(invalid(
                "duplicate empty category or empty and populated category",
            ));
        }
        self.summary
            .categories
            .entry(key)
            .or_default()
            .merge(&row.moments)?;
        self.summary.boundaries[row.boundary.index()].merge(&row.moments)?;
        Ok(())
    }

    fn finish(self) -> Result<HistorySummary, DiagnosticError> {
        for boundary in Boundary::ALL {
            for category in boundary.categories() {
                if !self.summary.categories.contains_key(&(boundary, category)) {
                    return Err(invalid(format!(
                        "missing category {} at {} for seed {}, history {}",
                        category.name(),
                        boundary.name(),
                        self.summary.seed,
                        self.summary.history
                    )));
                }
            }
        }
        Ok(self.summary)
    }
}

/// Validated sample with entity-level CSV data reduced to whole-history statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosticDataset {
    implementation: Implementation,
    plan: SamplePlan,
    histories: Vec<HistorySummary>,
}

impl DiagnosticDataset {
    /// Parse exactly this schema, rejecting malformed, duplicate, missing, or unexpected data.
    /// Each `(seed, history)` must occupy one contiguous block, bounding identity-check memory.
    pub fn read_csv(
        reader: impl BufRead,
        implementation: Implementation,
        plan: &SamplePlan,
    ) -> Result<Self, DiagnosticError> {
        let mut lines = reader.lines().enumerate();
        let header = loop {
            match lines.next() {
                Some((_, Ok(line))) if line.starts_with('#') => {}
                Some((_, Ok(line))) => break line,
                Some((_, Err(error))) => return Err(error.into()),
                None => return Err(invalid("missing CSV header")),
            }
        };
        if header != csv_header() {
            return Err(invalid(
                "CSV header does not match the spatial diagnostic schema",
            ));
        }
        Self::from_fallible_records(
            lines.filter_map(|(i, line)| match line {
                Ok(line) if line.starts_with('#') => None,
                Ok(line) => Some(
                    DiagnosticRecord::from_csv_row(&line)
                        .map_err(|e| invalid(format!("CSV line {}: {e}", i + 1))),
                ),
                Err(error) => Some(Err(error.into())),
            }),
            implementation,
            plan,
        )
    }

    /// Equivalent validation for in-memory records, using the same streaming reducer.
    pub fn from_records(
        rows: impl IntoIterator<Item = DiagnosticRecord>,
        implementation: Implementation,
        plan: &SamplePlan,
    ) -> Result<Self, DiagnosticError> {
        Self::from_fallible_records(rows.into_iter().map(Ok), implementation, plan)
    }

    fn from_fallible_records(
        rows: impl IntoIterator<Item = Result<DiagnosticRecord, DiagnosticError>>,
        implementation: Implementation,
        plan: &SamplePlan,
    ) -> Result<Self, DiagnosticError> {
        let mut current: Option<HistoryBuilder> = None;
        let mut completed = BTreeMap::new();
        for row in rows {
            let row = row?;
            row.validate()?;
            if row.implementation != implementation {
                return Err(invalid("mixed or unexpected implementation"));
            }
            if !plan.contains(row.seed, row.history) {
                return Err(invalid(format!(
                    "unexpected seed/history {}/{}",
                    row.seed, row.history
                )));
            }
            let key = (row.seed, row.history);
            if current
                .as_ref()
                .is_some_and(|builder| (builder.summary.seed, builder.summary.history) != key)
            {
                let summary = current.take().unwrap().finish()?;
                completed.insert((summary.seed, summary.history), summary);
            }
            if current.is_none() {
                if completed.contains_key(&key) {
                    return Err(invalid("duplicate or noncontiguous history block"));
                }
                current = Some(HistoryBuilder::new(row.seed, row.history));
            }
            current.as_mut().unwrap().add(row)?;
        }
        if let Some(builder) = current {
            let summary = builder.finish()?;
            completed.insert((summary.seed, summary.history), summary);
        }
        for (seed, histories) in &plan.groups {
            for history in histories {
                if !completed.contains_key(&(*seed, *history)) {
                    return Err(invalid(format!("missing seed/history {seed}/{history}")));
                }
            }
        }
        Ok(Self {
            implementation,
            plan: plan.clone(),
            histories: completed.into_values().collect(),
        })
    }

    pub fn implementation(&self) -> Implementation {
        self.implementation
    }

    pub fn plan(&self) -> &SamplePlan {
        &self.plan
    }

    /// Deterministic `(seed, history)` order, independent of the order of history blocks.
    pub fn histories(&self) -> &[HistorySummary] {
        &self.histories
    }

    /// Secondary summaries. Per-category and per-history RMS are descriptive, never the primary.
    pub fn summarize(&self) -> Result<ImplementationSummary, DiagnosticError> {
        let mut boundaries = [SpatialMoments::default(); 5];
        let mut categories: BTreeMap<(Boundary, Category), SpatialMoments> = BTreeMap::new();
        for history in &self.histories {
            for (total, value) in boundaries.iter_mut().zip(&history.boundaries) {
                total.merge(value)?;
            }
            for (key, value) in &history.categories {
                categories.entry(*key).or_default().merge(value)?;
            }
        }
        let n = self.histories.len();
        let population_summaries: Vec<_> = boundaries
            .into_iter()
            .map(|moments| PopulationSummary::new(moments, n))
            .collect::<Result<_, _>>()?;
        Ok(ImplementationSummary {
            implementation: self.implementation,
            plan: self.plan.clone(),
            boundaries: population_summaries.try_into().unwrap(),
            categories: categories
                .into_iter()
                .map(|(key, value)| Ok((key, PopulationSummary::new(value, n)?)))
                .collect::<Result<_, DiagnosticError>>()?,
            histories: self.histories.clone(),
        })
    }

    /// Precision of this implementation's final pooled RMS component, with the same
    /// frozen hierarchical resampling as the comparison. This is descriptive only:
    /// the four-percentage-point expansion rule requires the Rust/Geant4 ratio.
    pub fn pooled_rms_precision(&self) -> Result<RmsPrecision, DiagnosticError> {
        let summary = self.summarize()?;
        let boundary = Boundary::FinalSurvivors;
        let pooled_rms_nm = RadiusTotals::from(summary.boundaries[boundary.index()].moments)
            .positive_rms()
            .ok_or_else(|| invalid("undefined final pooled RMS"))?;
        let groups = self.bootstrap_groups();
        let stream = u64::from(self.implementation == Implementation::Geant4);
        let mut rng = RngStreams::new(ANALYSIS_SEED)
            .stream(stream, Stage::Physics)
            .expect("fixed analysis stream is valid");
        let mut samples = Vec::with_capacity(BOOTSTRAP_REPLICATES);
        for _ in 0..BOOTSTRAP_REPLICATES {
            samples.push(
                resample(&groups, &mut rng)?[boundary.index()]
                    .positive_rms()
                    .ok_or_else(|| invalid("undefined resampled final pooled RMS"))?,
            );
        }
        let lower_nm = percentile(&samples, 0.025)?;
        let upper_nm = percentile(&samples, 0.975)?;
        Ok(RmsPrecision {
            pooled_rms_nm,
            lower_nm,
            upper_nm,
            relative_half_width: (upper_nm - lower_nm) / pooled_rms_nm / 2.0,
        })
    }

    fn bootstrap_groups(&self) -> Vec<Vec<[RadiusTotals; 5]>> {
        self.plan
            .groups
            .keys()
            .map(|seed| {
                self.histories
                    .iter()
                    .filter(|history| history.seed == *seed)
                    .map(|history| history.boundaries.map(RadiusTotals::from))
                    .collect()
            })
            .collect()
    }
}

/// Secondary spatial/count summaries of one population; all distances are origin-based.
#[derive(Debug, Clone, PartialEq)]
pub struct PopulationSummary {
    /// Additive moments include counts, histogram, and the sample-wide maximum.
    pub moments: SpatialMoments,
    pub histories: usize,
    /// Includes zero-population histories in the denominator.
    pub count_per_history: f64,
    pub rms_radius_nm: Option<f64>,
    pub mean_z_nm: Option<f64>,
    /// sqrt(mean(z^2)), not a centered standard deviation.
    pub longitudinal_rms_nm: Option<f64>,
    /// sqrt(mean(x^2 + y^2)), not one transverse component.
    pub transverse_rms_nm: Option<f64>,
    pub max_radius_nm: Option<f64>,
    /// Particle proportions; undefined for an empty population.
    pub radial_fractions: Option<[f64; RADIAL_BIN_COUNT]>,
    /// Bin bounds for p = 0.50, 0.90 and 0.99 respectively.
    pub radial_quantiles: [Option<RadialBounds>; 3],
}

impl PopulationSummary {
    fn new(moments: SpatialMoments, histories: usize) -> Result<Self, DiagnosticError> {
        moments.validate()?;
        let n = moments.count as f64;
        Ok(Self {
            moments,
            histories,
            count_per_history: n / histories as f64,
            rms_radius_nm: moments.rms_radius_nm(),
            mean_z_nm: (n > 0.0).then(|| moments.sum_z_nm / n),
            longitudinal_rms_nm: (n > 0.0).then(|| (moments.sum_z2_nm2 / n).sqrt()),
            transverse_rms_nm: (n > 0.0).then(|| (moments.sum_x2_plus_y2_nm2 / n).sqrt()),
            max_radius_nm: (n > 0.0).then_some(moments.max_radius_nm),
            radial_fractions: (n > 0.0).then(|| moments.radial_bins.map(|count| count as f64 / n)),
            radial_quantiles: [
                moments.radial_quantile(0.5)?,
                moments.radial_quantile(0.9)?,
                moments.radial_quantile(0.99)?,
            ],
        })
    }
}

/// Every secondary population, including per-history/per-species moments and RMS accessors.
#[derive(Debug, Clone, PartialEq)]
pub struct ImplementationSummary {
    pub implementation: Implementation,
    pub plan: SamplePlan,
    pub boundaries: [PopulationSummary; 5],
    pub categories: BTreeMap<(Boundary, Category), PopulationSummary>,
    pub histories: Vec<HistorySummary>,
}

/// Descriptive one-implementation precision, not a Rust/Geant4 comparison interval.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RmsPrecision {
    pub pooled_rms_nm: f64,
    pub lower_nm: f64,
    pub upper_nm: f64,
    pub relative_half_width: f64,
}

/// Positive finite interval on the Rust/Geant4 RMS ratio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RatioInterval {
    pub lower: f64,
    pub upper: f64,
}

impl RatioInterval {
    /// Includes endpoints: an interval touching 1 does not detect a residual.
    pub fn excludes_one(self) -> bool {
        self.lower > 1.0 || self.upper < 1.0
    }
}

/// Boundary estimate and its member of the simultaneous five-boundary family.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundaryInterval {
    pub boundary: Boundary,
    pub ratio: f64,
    pub interval: RatioInterval,
}

/// Diagnostic comparison, separate from the normal run schema and existing M6 gate.
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialAnalysis {
    pub analysis_seed: u64,
    pub bootstrap_replicates: usize,
    /// Sole primary: sqrt(sum_r2 / count) across all seven final-survivor species.
    pub primary_ratio: f64,
    pub primary_interval: RatioInterval,
    /// `(upper - lower) / (2 * primary_ratio)`; multiply by 100 for percentage points.
    pub relative_half_width: f64,
    pub simultaneous_log_radius: f64,
    pub boundaries: [BoundaryInterval; 5],
    pub rust: ImplementationSummary,
    pub geant4: ImplementationSummary,
}

impl SpatialAnalysis {
    /// Earliest detectable boundary in reporting order, not a causal attribution.
    pub fn first_detected_divergence(&self) -> Option<Boundary> {
        self.boundaries
            .iter()
            .find(|b| b.interval.excludes_one())
            .map(|b| b.boundary)
    }

    /// Wording deliberately retains the interpretation limit for downstream reports.
    pub fn divergence_statement(&self) -> String {
        match self.first_detected_divergence() {
            Some(boundary) => format!(
                "first detected divergence: {}; this does not establish causation or equivalence at earlier boundaries.",
                boundary.name()
            ),
            None => "No first detected divergence: all simultaneous intervals include 1; this is inconclusive, not equivalence.".into(),
        }
    }

    /// Precision-only rule for a validated production level; significance never enters.
    /// Samples must have matching 40/80/160 coverage before making an expansion decision.
    pub fn precision_decision(&self) -> Result<PrecisionDecision, DiagnosticError> {
        let level = [40, 80, 160]
            .into_iter()
            .find(|level| {
                let plan = SamplePlan::production(*level).unwrap();
                self.rust.plan == plan && self.geant4.plan == plan
            })
            .ok_or_else(|| {
                invalid("precision rule requires matching preregistered production samples")
            })?;
        Ok(if self.relative_half_width <= 0.04 {
            PrecisionDecision::TargetReached
        } else if level == 160 {
            PrecisionDecision::CapReached
        } else {
            PrecisionDecision::ExpandToHistoriesPerSeed(level * 2)
        })
    }
}

/// The preregistered precision rule; the cap is 800 histories per implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecisionDecision {
    TargetReached,
    ExpandToHistoriesPerSeed(u64),
    CapReached,
}

#[derive(Debug, Clone, Copy, Default)]
struct RadiusTotals {
    count: u64,
    sum_r2: f64,
}

impl From<SpatialMoments> for RadiusTotals {
    fn from(value: SpatialMoments) -> Self {
        Self {
            count: value.count,
            sum_r2: value.sum_r2_nm2,
        }
    }
}

impl RadiusTotals {
    fn add(&mut self, other: Self) -> Result<(), DiagnosticError> {
        self.count = self
            .count
            .checked_add(other.count)
            .ok_or_else(|| invalid("bootstrap count overflow"))?;
        self.sum_r2 += other.sum_r2;
        if !self.sum_r2.is_finite() {
            return Err(invalid("bootstrap squared-radius overflow"));
        }
        Ok(())
    }

    fn positive_rms(self) -> Option<f64> {
        if self.count == 0 {
            return None;
        }
        let rms = (self.sum_r2 / self.count as f64).sqrt();
        (rms.is_finite() && rms > 0.0).then_some(rms)
    }
}

fn ratios(
    rust: &[RadiusTotals; 5],
    geant4: &[RadiusTotals; 5],
    context: &str,
) -> Result<[f64; 5], DiagnosticError> {
    let mut result = [0.0; 5];
    for boundary in Boundary::ALL {
        let i = boundary.index();
        let ratio = rust[i]
            .positive_rms()
            .zip(geant4[i].positive_rms())
            .map(|(r, g)| r / g);
        result[i] = ratio.filter(|r| r.is_finite() && *r > 0.0).ok_or_else(|| {
            DiagnosticError::UndefinedRatio {
                boundary,
                context: context.into(),
            }
        })?;
    }
    Ok(result)
}

// Use the existing ChaCha12-backed wrapper. Its uniform has 2^52 equiprobable midpoints.
// Rejection removes modulo bias, without adding a dependency on rand to dna-sim.
fn sample_index(rng: &mut HistoryRng, upper: usize) -> usize {
    const WORDS: u64 = 1 << 52;
    let upper = upper as u64;
    assert!(upper > 0 && upper <= WORDS);
    let limit = WORDS - WORDS % upper;
    loop {
        let word = (rng.uniform_open01() * WORDS as f64) as u64;
        if word < limit {
            return (word % upper) as usize;
        }
    }
}

fn resample(
    groups: &[Vec<[RadiusTotals; 5]>],
    rng: &mut HistoryRng,
) -> Result<[RadiusTotals; 5], DiagnosticError> {
    let mut totals = [RadiusTotals::default(); 5];
    for _ in 0..groups.len() {
        let group = &groups[sample_index(rng, groups.len())];
        for _ in 0..group.len() {
            let history = &group[sample_index(rng, group.len())];
            for (total, value) in totals.iter_mut().zip(history) {
                total.add(*value)?;
            }
        }
    }
    Ok(totals)
}

/// Linear interpolation at sorted rank `(B - 1) p`. Inputs must be finite and nonempty.
pub fn percentile(values: &[f64], probability: f64) -> Result<f64, DiagnosticError> {
    if values.is_empty()
        || values.iter().any(|x| !x.is_finite())
        || !probability.is_finite()
        || !(0.0..=1.0).contains(&probability)
    {
        return Err(invalid(
            "percentile requires finite nonempty values and p in [0, 1]",
        ));
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = (sorted.len() - 1) as f64 * probability;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    let fraction = rank - lo as f64;
    // Convex combination avoids overflowing the subtraction for opposite-sign inputs.
    let result = sorted[lo] * (1.0 - fraction) + sorted[hi] * fraction;
    if !result.is_finite() {
        return Err(invalid("percentile arithmetic overflow"));
    }
    Ok(result)
}

/// Exactly 20,000 independent hierarchical replicates, with the frozen analysis seed.
/// Each draw selects seed groups then whole histories; all five boundaries travel together.
/// Any undefined observed or resampled boundary ratio fails the full analysis explicitly.
pub fn analyze(
    rust: &DiagnosticDataset,
    geant4: &DiagnosticDataset,
) -> Result<SpatialAnalysis, DiagnosticError> {
    analyze_with_seed(rust, geant4, ANALYSIS_SEED)
}

/// Explicit analysis seed for reproducibility checks; production uses [`analyze`].
/// Rust and Geant4 use disjoint ChaCha streams (history 0 and 1 on this analysis key),
/// regardless of whether their simulation seed labels happen to match.
pub fn analyze_with_seed(
    rust: &DiagnosticDataset,
    geant4: &DiagnosticDataset,
    analysis_seed: u64,
) -> Result<SpatialAnalysis, DiagnosticError> {
    if rust.implementation != Implementation::Rust
        || geant4.implementation != Implementation::Geant4
    {
        return Err(invalid(
            "analyzer expects Rust then Geant4 independent samples",
        ));
    }
    let rust_summary = rust.summarize()?;
    let geant4_summary = geant4.summarize()?;
    let observed = ratios(
        &rust_summary
            .boundaries
            .each_ref()
            .map(|s| RadiusTotals::from(s.moments)),
        &geant4_summary
            .boundaries
            .each_ref()
            .map(|s| RadiusTotals::from(s.moments)),
        "observed sample",
    )?;
    let rust_groups = rust.bootstrap_groups();
    let geant4_groups = geant4.bootstrap_groups();
    let streams = RngStreams::new(analysis_seed);
    let mut rust_rng = streams
        .stream(0, Stage::Physics)
        .expect("fixed analysis stream is valid");
    let mut geant4_rng = streams
        .stream(1, Stage::Physics)
        .expect("fixed analysis stream is valid");
    let mut primary = Vec::with_capacity(BOOTSTRAP_REPLICATES);
    let mut deviations = Vec::with_capacity(BOOTSTRAP_REPLICATES);
    for replicate in 0..BOOTSTRAP_REPLICATES {
        let r = resample(&rust_groups, &mut rust_rng)?;
        let g = resample(&geant4_groups, &mut geant4_rng)?;
        let replicate_ratios = ratios(&r, &g, &format!("bootstrap replicate {replicate}"))?;
        primary.push(replicate_ratios[Boundary::FinalSurvivors.index()]);
        deviations.push(
            replicate_ratios
                .into_iter()
                .zip(observed)
                .map(|(sample, center)| (sample.ln() - center.ln()).abs())
                .fold(0.0, f64::max),
        );
    }
    let q = percentile(&deviations, 0.95)?;
    let mut intervals = Vec::with_capacity(5);
    for boundary in Boundary::ALL {
        let ratio = observed[boundary.index()];
        let interval = RatioInterval {
            lower: (ratio.ln() - q).exp(),
            upper: (ratio.ln() + q).exp(),
        };
        if !interval.lower.is_finite() || interval.lower <= 0.0 || !interval.upper.is_finite() {
            return Err(invalid("simultaneous interval overflow or underflow"));
        }
        intervals.push(BoundaryInterval {
            boundary,
            ratio,
            interval,
        });
    }
    let primary_ratio = observed[Boundary::FinalSurvivors.index()];
    let primary_interval = RatioInterval {
        lower: percentile(&primary, 0.025)?,
        upper: percentile(&primary, 0.975)?,
    };
    if primary_interval.lower <= 0.0 || primary_interval.upper <= 0.0 {
        return Err(invalid("primary percentile interval underflow"));
    }
    let relative_half_width =
        (primary_interval.upper - primary_interval.lower) / primary_ratio / 2.0;
    if !relative_half_width.is_finite() {
        return Err(invalid("relative interval half-width overflow"));
    }
    Ok(SpatialAnalysis {
        analysis_seed,
        bootstrap_replicates: BOOTSTRAP_REPLICATES,
        primary_ratio,
        primary_interval,
        relative_half_width,
        simultaneous_log_radius: q,
        boundaries: intervals.try_into().unwrap(),
        rust: rust_summary,
        geant4: geant4_summary,
    })
}
