# Ticket 03: audit-cascade-and-scoring-semantics

**Model:** opus
**Effort:** high

**Acceptance Criteria:**
- [ ] Rust and Geant4 are shown to use equivalent initial-cascade settings: all secondaries followed through solvation or attachment, fluorescence/Auger disabled, Geant4 elastic rotation, 2 eV primary floor, and Meesungnoen2002 solvation.
- [ ] Count-time behavior and RMS radius are tested as alive species at global 1 ps and distance from the primary origin, not displacement from the parent event.
- [ ] Any confirmed transport or scoring defect is fixed with focused regression coverage; otherwise Q12/Q13/Q16/Q17 and the scoring audit are documented with evidence.
- [ ] The report and unchanged asserting gate are rerun after the audit.

**Files to Touch:**
- `crates/dna-sim/src/physchem_scoring.rs`
- `crates/dna-sim/src/run.rs`
- `crates/dna-sim/tests/physchem_b.rs`
- `crates/dna-sim/tests/physchem_reference.rs`
- `validation/geant4/g4rs_ref/g4rs_ref.cc`
- `docs/08-roadmap.md`
- `docs/09-open-questions-and-risks.md`

**Verification Step:**

Run:
```bash
cargo test -p dna-sim physchem && cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture
```

Expected:
Focused scoring tests pass, the comparison report completes, and the unchanged gate result is captured for the final closure decision.

**Notes:**

The existing `physchem` generator mode is read-only except for a separately evidenced bug fix, which would require documentation and a scratch rerun before replacement; this ticket must not regenerate the committed CSV. Avoid changing C++ unless the audit finds a generator bug that can be proven from source.
