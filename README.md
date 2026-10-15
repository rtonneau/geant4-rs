# geant4-rs
(Partial) Port of Geant4(-DNA) to Rust

## Layout
- `crates/dna-core` — units, `Vec3`, reproducible RNG streams, errors, provenance, data manifest (milestone M0).
- `data/` — committed subset of Geant4 G4EMLOW 8.8 `dna/` tables with SHA-256 manifest.
- `docs/` — planning package and roadmap (`docs/README.md`), `docs/units.md`, `docs/reproducibility.md`.

Checks: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
