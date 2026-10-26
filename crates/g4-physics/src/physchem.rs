//! Physicochemical event record. The types moved to `g4-core` at M6 (docs/03: the physics and
//! chemistry crates exchange only `g4-core` types); this module re-exports them so that the
//! M2/M3 paths (`g4_physics::physchem::PhysChemEvent`) keep working.

pub use g4_core::physchem::{PHYSCHEM_CSV_HEADER, PhysChemEvent, PhysChemKind};
