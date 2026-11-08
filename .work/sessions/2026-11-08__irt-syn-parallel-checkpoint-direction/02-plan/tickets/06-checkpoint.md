# Ticket 06: checkpoint

**Acceptance Criteria:**
- [ ] Persist molecules, time, identity counters, stochastic/scheduling state, future primaries, scoring and provenance sufficient for conditional-law correctness.
- [ ] Use versioned, checked, atomically published files; reject corruption and incompatible models without partial successful output.
- [ ] Test interrupted writes, stop/resume at injection and reaction boundaries, and predefined statistical comparisons against uninterrupted runs.
- [ ] Do not require bitwise replay or silently discard conditional reaction information.

**Files to Touch:**
- `crates/g4-chemistry/src/`
- `crates/g4-sim/src/`
- `crates/g4-sim/tests/`
- `docs/reproducibility.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected:
All three commands exit 0. Round-trip, corruption/failure and conditional-continuation tests pass.

**Notes:**
Add focused tests for the acceptance criteria above; register exact scientific gate commands as tests and reference consumers are introduced. No blanket ignored-test execution. Preserve existing reference data. Depends on preceding tickets. New scientific functionality requires tests.
