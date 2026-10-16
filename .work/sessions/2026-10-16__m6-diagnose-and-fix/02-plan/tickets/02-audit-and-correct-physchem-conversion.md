# Ticket 02: audit-and-correct-physchem-conversion

**Model:** opus
**Effort:** high

**Acceptance Criteria:**
- [ ] Every Rust excitation, ionisation, attachment, and recombined-water channel is checked against `G4ChemDissociationChannels_option1` for state mapping, probability, products, decay timing, and displacement selection.
- [ ] Electron-hole recombination is tested with ordering ablations that quantify Q21/R14 and preserve deterministic history-local RNG streams.
- [ ] Any confirmed Rust conversion defect is corrected with a focused failing-then-passing test; if none is found, the source-backed audit and measured ablation are documented.
- [ ] The comparison report and focused `dna-chemistry` tests are rerun after the audit.

**Files to Touch:**
- `data/chemistry/dissociation_option3.toml`
- `crates/dna-chemistry/src/channels.rs`
- `crates/dna-chemistry/src/engine.rs`
- `crates/dna-chemistry/tests/channel_statistics.rs`
- `crates/dna-chemistry/tests/history.rs`
- `docs/02-cpp-reference-inventory.md`
- `docs/04-scientific-algorithms.md`
- `docs/09-open-questions-and-risks.md`

**Verification Step:**

Run:
```bash
cargo test -p dna-chemistry && cargo test -p dna-sim --test physchem_reference m6_initial_species_comparison_report -- --ignored --nocapture
```

Expected:
All `dna-chemistry` tests pass and the report completes for 1, 10, 100, and 900 keV with any changed result explained by a cited correction.

**Notes:**

Do not change a source value without verifying it in the Geant4 11.4.1 C++ tree. Keep physical, physicochemical, and chemical-stage products distinct. A finding that the reference includes chemical reactions is not permission to add M7 reactions to M6.
