# Ticket 01: establish-snapshot-semantics

**Model:** opus
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] The existing comparison is rerun and its full species/count/RMS pattern is recorded, including the Geant4-only `H2O2`, `HO2°`, `O^-`, and `O2` products.
- [ ] Geant4 11.4.1 scheduler, dissociation, and reaction call order is cited from exact C++ paths and used to decide whether the first `dt > 0` callback is before or after zero-time reactions.
- [ ] A focused regression or diagnostic test distinguishes species reachable from the option1 dissociation channels from species that require chemical reactions.
- [ ] `docs/08-roadmap.md` and `docs/09-open-questions-and-risks.md` state the evidenced snapshot semantics without changing the gate tolerances.

**Files to Touch:**
- `crates/dna-sim/tests/physchem_reference.rs`
- `docs/08-roadmap.md`
- `docs/09-open-questions-and-risks.md`

**Verification Step:**

Run:
```bash
cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture
```

Expected:
The non-asserting report completes and the added semantic diagnostic passes; the unchanged M6 gate either passes or fails with the same fixed acceptance criteria.

**Notes:**

Treat the appearance of reaction products as evidence to explain, not proof by itself. Read `G4Scheduler.cc`, `G4ITModelProcessor.cc`, `G4DNAMolecularDissociation.cc`, `G4ChemDissociationChannels_option1.cc`, and the option3 reaction table. Do not modify `validation/reference/` or the existing generator mode.
