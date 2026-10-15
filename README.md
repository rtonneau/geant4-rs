# geant4-rs
(Partial) Port of Geant4(-DNA) to Rust

## Layout
- `crates/dna-core` — units, `Vec3`, reproducible RNG streams, errors, provenance, data manifest (milestone M0).
- `crates/dna-data` — readers and Geant4-exact interpolation of the `G4EMLOW` `dna/` tables (M1).
- `crates/dna-physics` — option2 electron models (Champion elastic, Born excitation, Born1 ionisation), process table, single-electron transport with energy ledger (M2).
- `crates/dna-sim` — deterministic parallel histories, scorers, JSON output with provenance (M2).
- `validation/` — Geant4 reference generator and instructions (`validation/geant4/README.md`), reference CSV directory (`validation/reference/`, empty until generated).
- `data/` — committed subset of Geant4 G4EMLOW 8.8 `dna/` tables with SHA-256 manifest.
- `docs/` — planning package and roadmap (`docs/README.md`), `docs/units.md`, `docs/reproducibility.md`.

Checks: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
