# Ticket 01: pair-history

**Model:** medium
**Model (Jev):** sonnet-5.5 (confidence 0.1)
**Effort:** high

**Acceptance Criteria:**
- [ ] Build validated history input, stable molecule identities and deterministic event queue; execute isolated pair events and contact closure; test invalid times and capacity errors.

**Files to Touch:**
- `crates/g4-chemistry/src/history.rs`
- `crates/g4-chemistry/src/lib.rs`
- `crates/g4-chemistry/src/error.rs`
- `crates/g4-chemistry/tests/history_pair.rs`

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry --test history_pair
```

Expected: all tests pass.

**Notes:**

G1 must pass before implementation.
