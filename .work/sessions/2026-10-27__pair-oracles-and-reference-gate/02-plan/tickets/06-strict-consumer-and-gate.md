# Ticket 06: strict-consumer-and-gate

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] Add a strict ignored Rust consumer rejecting missing/duplicate rows, wrong settings, incomplete counts, invalid completion/provenance and insufficient precision.
- [ ] Compare independently seeded Rust and source-matched Geant4 option3 IRT with frozen family alpha 0.001 and <0.01 absolute effect limit, reporting analytic, Brownian and Geant4 verdicts separately.
- [ ] Record evidence REQUESTED while user-run data are absent; G1 passes only when all three verdicts pass. Preserve legacy V9 and RR-003 verdicts.

**Files to Touch:**
- `crates/g4-chemistry/tests/geant4_pair_r04.rs`
- `docs/validation/pair-oracle-convergence.md`
- `docs/08-roadmap.md`
- `validation/reference/README.md`

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry --test geant4_pair_r04
```

Expected:
Parser/provenance/bias tests pass; the comparison remains ignored until complete user-run data and provenance are committed.

**Notes:**

Do not invent Geant4 values or treat a missing fixture as a pass. Run cargo fmt, clippy and workspace tests before completion.
