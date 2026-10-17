# Ticket 04: record-audit-result

**Model:** sonnet
**Model (Jev):** sonnet-5.5, confidence 0.21
**Effort:** high

**Acceptance Criteria:**
- [ ] `docs/08-roadmap.md` has an "M6 residual audit" section: stage family or "undecided -> `m6-process-counts-geant4-request`", the relation table, the seed spread, the source table and the limitations.
- [ ] `docs/09-open-questions-and-risks.md` has a Q21/R14 update.
- [ ] `docs/gps-backlog-m6-residual.md` is updated: audit seed done, `m6-process-counts-geant4-request` kept or skipped per the verdict, and the "No model is changed" text matches this session.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` are run and their results reported.

**Files to Touch:**
- `docs/08-roadmap.md`
- `docs/09-open-questions-and-risks.md`
- `docs/gps-backlog-m6-residual.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected:
All three exit 0; the test count is stated, and the ignored tests that fail by design are reported separately.

**Notes:**

State the Geant4-side variance limitation. Never claim a pass that was not executed.
