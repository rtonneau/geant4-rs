# Ticket 10: release-validation

**Acceptance Criteria:**
- [ ] Complete reference comparisons for physical transport, handoff, IRT-syn chemistry and time-dependent yields using predefined criteria.
- [ ] Validate instantaneous and timed shared-volume irradiation plus restart and spatial boundary cases.
- [ ] Reconcile roadmap and preserved G2/recovery work; distinguish deferred work, scientific limitations and acceptance evidence.
- [ ] Keep protons and mesoscopic chemistry deferred; do not run Geant4 generators in agent sessions.

**Files to Touch:**
- `validation/`
- `docs/`
- `README.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected:
All three commands exit 0. All mandatory checks and registered release gates pass; incomplete evidence blocks release claims.

**Notes:**
Add focused tests for the acceptance criteria above; register exact scientific gate commands as tests and reference consumers are introduced. No blanket ignored-test execution. Preserve existing reference data. Depends on preceding tickets. New scientific functionality requires tests.
