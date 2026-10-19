# Ticket 05: document-and-queue-geant4-run

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] `validation/geant4/README.md` documents the exact mode arguments, long-form schema, five boundaries, radial edges, completeness checks, statistical protocol, precision expansion rule, and the paired ignored Rust consumer.
- [ ] `validation/geant4/BUILD_NOTES.md` gives a path-independent Geant4 11.4.1 configure/build recipe and records the successful compile/link check and any fixes encountered while staying below 300 lines.
- [ ] `validation/geant4/RUN_REQUESTS.md` contains a new open request with the exact command, initial five seeds x 40 histories, expected new CSV and provenance TOML names, completeness requirements, consumer command, and the 80/160 expansion rule; existing requests remain unchanged.
- [ ] The ignored consumer fails clearly when data/provenance are absent or incomplete and, once supplied, reports the primary interval, simultaneous boundary intervals, secondary diagnostics, first detected divergence if any, and whether more histories are required solely by the precision rule.
- [ ] `docs/08-roadmap.md` and `docs/09-open-questions-and-risks.md` record that the investigation is awaiting user-run data and neither reopen nor close M6.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` pass.

**Files to Touch:**
- `validation/geant4/README.md`
- `validation/geant4/BUILD_NOTES.md`
- `validation/geant4/RUN_REQUESTS.md`
- `crates/dna-sim/tests/physchem_reference.rs`
- `docs/08-roadmap.md`
- `docs/09-open-questions-and-risks.md`

**Verification Step:**

Run:
```bash
cargo test -p dna-sim --test physchem_reference m6_900kev_spatial_diagnostic -- --ignored --exact --nocapture
```

Expected:
Before the user run, the test exits with the explicit `awaiting data` message naming the new CSV and provenance files; all non-ignored workspace checks pass.

**Notes:**

Do not create, run, overwrite, or partially commit reference data. The project lead runs Geant4 and commits complete data and provenance. A later close-gate session interprets the result or creates a separate source-proven fix session.
