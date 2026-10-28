# Ticket 04: observation-and-limits

**Model:** heavy
**Model (Jev):** opus-5.5 (confidence 0.37)
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] Expose the bounded history interface, immutable pre-IRT and post-contact counts, right-continuous inclusive observations, opt-in bounded traces and typed incomplete errors; prove observation grids preserve final state and RNG.

**Files to Touch:**
- `crates/g4-chemistry/src/history.rs`
- `crates/g4-chemistry/src/error.rs`
- `crates/g4-chemistry/tests/history_observation.rs`

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry --test history_observation
```

Expected: all tests pass.

**Notes:**

G1 must pass before implementation.
