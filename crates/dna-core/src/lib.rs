//! Foundations shared by every geant4-rs crate: units, vectors, reproducible
//! random-number streams, typed errors, provenance metadata and the data manifest.
//!
//! Shared water-specific contracts also live here: [`physchem`] records the
//! physical-to-physicochemical handoff, and [`penetration`] supplies the
//! electron thermalisation law used by both stage crates. Chemical species and
//! their serialization belong to `dna-chemistry`; core has no stage dependency.

pub mod error;
pub mod manifest;
pub mod penetration;
pub mod physchem;
pub mod provenance;
pub mod rng;
pub mod sampling;
pub mod sha256;
pub mod testing;
pub mod units;
pub mod vec3;
