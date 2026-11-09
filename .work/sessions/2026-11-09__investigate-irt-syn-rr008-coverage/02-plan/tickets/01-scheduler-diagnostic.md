# Ticket 01: scheduler-diagnostic

**Acceptance Criteria:**
- [ ] Separate producer and consumer implement the frozen contract in 01-grill/resume.md, including collision checks, strict schema and verdicts.
- [ ] Source bound and exact thresholds independently verified; compile-check and appropriate tests completed or limitations recorded.
- [ ] Independent review completed and new request registered with exact reproducible commands; no generator run.

**Files to Touch:**
- `validation/geant4/`
- `crates/g4-chemistry/tests/`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected:
All checks pass; generator is compile-checked only.

**Notes:**
Preserve existing data, ticket history, caps and inconclusive verdicts. Follow the approved frozen contract and record independent review.
