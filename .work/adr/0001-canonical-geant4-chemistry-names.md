# Use canonical Geant4 chemistry names throughout Rust

_ADR 0001, recorded 2026-10-16 (session 2026-10-16__for-simplicity-rust-should-use-exact-same-as-geant4-c-sources)._

<!-- Format adapted from mattpocock-skills' domain-modeling (ADR-FORMAT.md): https://github.com/mattpocock/skills.git -->

Chemical species use the byte-for-byte Geant4 11.4.1 `G4MoleculeTable` configuration key as their canonical identity throughout the API, data, serialization, and output. Rust enum variants preserve that chemistry spelling and casing wherever Rust syntax permits; names containing `°` use the closest legal identifier but expose the exact key through `chemistry_name()`, while legacy ASCII spellings are accepted only by an explicit compatibility parser and are never emitted.
