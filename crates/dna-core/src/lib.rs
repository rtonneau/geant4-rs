//! Foundations shared by every geant4-rs crate: units, vectors, reproducible
//! random-number streams, typed errors, provenance metadata and the data manifest.
//!
//! No physics lives here (milestone M0).

pub mod error;
pub mod manifest;
pub mod penetration;
pub mod physchem;
pub mod provenance;
pub mod rng;
pub mod sampling;
pub mod sha256;
pub mod species;
pub mod units;
pub mod vec3;
