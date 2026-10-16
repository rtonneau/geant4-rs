# Ticket 02: add-m6-comparison-report

**Model:** sonnet
**Effort:** medium

**Acceptance Criteria:**
- [ ] A second ignored, non-asserting test in `crates/dna-sim/tests/physchem_reference.rs` prints one fixed-width table for each reference energy: 1, 10, 100, and 900 keV.
- [ ] Every table contains all 15 entries from `Species::ALL` in canonical chemistry-name order; a missing Geant4 row is treated as zero observations.
- [ ] Every row displays Geant4 mean +/- SE, Rust mean +/- SE, signed combined-error z-score, signed relative difference, Geant4 RMS radius, and Rust RMS radius.
- [ ] A zero combined standard error, zero Geant4 mean, or absent RMS value prints `N/A` for the affected metric.
- [ ] The existing asserting test and every pre-registered threshold remain unchanged, and no file under `validation/reference/` is modified.

**Files to Touch:**
- `crates/dna-sim/tests/physchem_reference.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-sim --test physchem_reference m6_initial_species_comparison_report -- --ignored --nocapture
```

Expected:
The test passes without asserting scientific agreement and prints four energy tables with 15 species rows each and the agreed columns.

**Notes:**

Reuse the current CSV loader, `RunConfig`, Geant4 elastic rotation, option3 physicochemical engine, history counts, and master seed `20261017`. Use `z = (Rust - Geant4) / sqrt(SE_G4^2 + SE_Rust^2)` and relative difference `(Rust - Geant4) / Geant4`. Keep reporting helpers local to this integration test unless reuse proves necessary.
