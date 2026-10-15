//! Physicochemical event record. The types moved to `dna-core` at M6 (docs/03: the physics and
//! chemistry crates exchange only `dna-core` types); this module re-exports them so that the
//! M2/M3 paths (`dna_physics::physchem::PhysChemEvent`) keep working.

pub use dna_core::physchem::{PHYSCHEM_CSV_HEADER, PhysChemEvent, PhysChemKind};
