# Ticket 02: move-species-to-chemistry

**Model:** heavy
**Effort:** high

**Acceptance Criteria:**
- [ ] Move the complete `dna-core/src/species.rs` implementation and tests to private `dna-chemistry/src/species_types.rs`; export `Species`, `SpeciesInstance` and `SPECIES_CSV_HEADER` from the chemistry root. Keep `dna_chemistry::species::SpeciesTable` and shared `dna_core::physchem`/`penetration` contracts. Remove `dna_core::species` without a compatibility cycle.
- [ ] Update every source, test and example consumer. Add `crates/dna-chemistry/tests/species_interface.rs` proving all 15 names and order, strict and legacy parsing, CSV header `species,x_nm,y_nm,z_nm,time_ps`, and `e_aq,1.0,-2.5,0.25,1.0` for the specified instance. Start with the root-interface test failing to compile, then move the existing type without algorithm changes.
- [ ] Keep event fields, shell/level numbering, placement and energy-deposit origins, species serialization, creation order, RNG draws and 1 ps initialization unchanged. Update core docs to describe the shared water-specific contracts. Core has no chemistry dependency; physics and chemistry remain independent.
- [ ] Recapture both profiles in a new directory and compare against ticket 01. Exact scientific payload and input identity match. Run workspace and named M2/M3/M6 gates before committing.

**Files to Touch:**
- `crates/dna-core/src/species.rs`
- `crates/dna-core/src/lib.rs`
- `crates/dna-chemistry/src/species_types.rs`
- `crates/dna-chemistry/src/lib.rs`
- Chemistry and simulation imports of `dna_core::species`
- `crates/dna-chemistry/tests/species_interface.rs`
- `docs/crate-separation.md`

**Verification Step:**

Run:
```bash
cargo test -p dna-chemistry --test species_interface --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test -p dna-sim --test geant4_reference --locked -- --ignored
cargo test -p dna-sim --test slowing_down_reference --locked -- --ignored
cargo test -p dna-sim --test physchem_reference --locked m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact
python scripts/check_crate_split.py --compare <ticket-01-physics.json> <ticket-02-physics.json>
python scripts/check_crate_split.py --compare <ticket-01-physchem.json> <ticket-02-physchem.json>
```

Expected:
All commands exit 0 with their expected tests selected. Both comparisons report no scientific difference. Dependency trees show no core-to-chemistry or cross-stage edge, and no active source/test/example import of `dna_core::species` remains.

**Notes:**

Package names stay `dna-*`. S02 owns renaming and S03 owns optional chemistry. Do not change scientific models, regenerate references or add shim crates.
