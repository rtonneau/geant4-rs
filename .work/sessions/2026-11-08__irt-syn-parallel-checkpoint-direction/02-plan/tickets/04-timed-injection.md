# Ticket 04: timed-injection

**Acceptance Criteria:**
- [ ] Validate primary energy, position, direction and arrival-time input.
- [ ] Support equal-time pulses and injections during chemistry in one interacting volume.
- [ ] Define physical/physicochemical handoff times and equal-time ordering without shifting all tracks to one artificial time.
- [ ] Verify transport and handoff regressions and cross-track reaction cases.

**Files to Touch:**
- `crates/g4-sim/src/`
- `crates/g4-sim/tests/`
- `crates/g4-physics/src/`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected:
All three commands exit 0. Instantaneous, separated and coincident arrivals pass timeline and physical regression tests.

**Notes:**
Add focused tests for the acceptance criteria above; register exact scientific gate commands as tests and reference consumers are introduced. No blanket ignored-test execution. Preserve existing reference data. Depends on preceding tickets. New scientific functionality requires tests.
