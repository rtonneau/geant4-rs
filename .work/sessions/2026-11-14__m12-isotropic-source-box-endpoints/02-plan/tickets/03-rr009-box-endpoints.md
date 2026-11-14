# Ticket 03: rr009-box-endpoints

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] Box rows in `rr009_finite_volume.rs` run with isotropic source; sphere rows keep +z; Z_FAMILY, MARGIN and ACCEPTED_INCONCLUSIVE unchanged; header limitation paragraph updated.
- [ ] Consumer run (release, ignored test) completes; all 24 endpoints evaluated; output saved under the session scratch dir.
- [ ] Verdict for the six box endpoints and the gate recorded in docs/07, docs/08, docs/gps-backlog-m12-geometry.md and validation/gates.json.

**Files to Touch:**
- `crates/g4-sim/tests/rr009_finite_volume.rs`
- `docs/07-validation-strategy.md`
- `docs/08-roadmap.md`
- `validation/gates.json`

**Verification Step:**

Run:
```bash
cargo test --release -p g4-sim --test rr009_finite_volume -- --ignored --nocapture
```

Expected:
24 endpoints reported with PASS/INCONCLUSIVE/FAIL; no NOT RUN.

**Notes:**

Any FAIL or new INCONCLUSIVE is reported to the user, not accepted by the agent.
