# Ticket 01: audit-n1-thermalisation-boundary

**Model:** opus
**Model (Jev):** opus-5.5 (confidence 0.74)
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] The Geant4 11.4.1 path from `G4TDNAOneStepThermalizationModel::SampleSecondaries` through penetration sampling, `ComputeStep`, `safety <= displacementMag`, and the `0.8 * safety` replacement is checked against the Rust `Solvation::sample` and transport-to-`PhysChemEvent` path, with exact source paths and lines recorded.
- [ ] The audit proves from `validation/geant4/g4rs_ref/g4rs_ref.cc` that the M6 reference starts at the centre of a 1 m water sphere and compares this boundary distance with the thermalisation-displacement scale. It does not claim that clipping is generally absent from Geant4.
- [ ] Existing focused tests for the Meesungnoen displacement distribution, the transport identity `N_solv + N_att = 1 + N_sec`, solvation-to-`PhysChemEvent`, and one-`e_aq` creation are identified and run. A regression test is added only if an audited invariant is not already covered; statistical tests are not duplicated.
- [ ] `docs/08-roadmap.md` changes N1 from "not audited" to a source-backed verdict: the geometry clip is inapplicable to the infinite-water model and unreachable in the committed reference workload, so it cannot explain the 900 keV RMS excess. N2 stays explicitly out of scope and the RMS excess stays unresolved.
- [ ] Related descriptions in `docs/02-cpp-reference-inventory.md`, `docs/09-open-questions-and-risks.md`, `validation/geant4/README.md`, and `crates/dna-physics/src/models/solvation.rs` are checked for contradictions and changed only where needed. No reference CSV, provenance file, C++ generator mode, production model, tolerance, or run request changes.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` exit 0.

**Files to Touch:**
- `docs/08-roadmap.md`
- `docs/02-cpp-reference-inventory.md` (only if its current wording is incomplete or contradictory)
- `docs/09-open-questions-and-risks.md` (only if needed to keep N1/RMS status consistent)
- `validation/geant4/README.md` (only if needed to distinguish active clipping from its inapplicability to this geometry)
- `crates/dna-physics/src/models/solvation.rs` (documentation only, unless the source audit proves a separate defect)
- An existing focused Rust test file only if current coverage does not already establish an agreed invariant

**Verification Step:**

Run:
```powershell
cargo test -p dna-physics --test low_energy
cargo test -p dna-chemistry --test history solvation_creates_an_e_aq_at_the_event_position_without_random_numbers -- --exact
cargo test -p dna-sim --test physchem_a species_conservation_and_untouched_ledger_on_real_cascades -- --exact
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected:
Every command exits 0, each named targeted test passes, and the workspace test summary has zero failures.

**Notes:**

- Read the Geant4 `.cc` and `.hpp` implementation, not only declarations or existing Rust prose. The reference tree is read-only.
- Treat the active navigator clip and its workload-specific unreachability as separate facts. Do not describe the block as commented out.
- The current evidence is a 1 m radius world versus nanometre-scale thermalisation displacements. Quantify the bound from existing model and test evidence without inventing a physical cutoff.
- Formal M6 closure remains a project-lead decision. This ticket closes only N1.
- Do not add finite-geometry support, audit N2 beyond the already-covered count invariant, run Geant4, or create a generator or run request unless the existing evidence is genuinely insufficient. If it is insufficient, block the ticket rather than expanding scope.
