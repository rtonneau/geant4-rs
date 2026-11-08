# Ticket 05: scoring

**Acceptance Criteria:**
- [ ] Score species counts and G-values on an explicit observation grid with documented deposited-energy denominator.
- [ ] Specify observations at injection/reaction boundaries, zero deposited energy and uncertainty across independent repeated volumes.
- [ ] Observation must not change random draws or dynamics.

**Files to Touch:**
- `crates/g4-sim/src/scoring.rs`
- `crates/g4-sim/src/`
- `crates/g4-sim/tests/`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected:
All three commands exit 0. Known-count/energy examples and observation invariance pass.

**Notes:**
Add focused tests for the acceptance criteria above; register exact scientific gate commands as tests and reference consumers are introduced. No blanket ignored-test execution. Preserve existing reference data. Depends on preceding tickets. New scientific functionality requires tests.
