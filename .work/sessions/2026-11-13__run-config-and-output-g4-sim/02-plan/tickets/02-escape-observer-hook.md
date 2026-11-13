# Ticket 02: escape-observer-hook

**Effort:** medium

**Acceptance Criteria:**
- [ ] `ExitEvent { generation, position, direction, energy }` and a default `on_escape` hook
- [ ] The exit point lies on the surface; summed energies equal `ledger.escaped`; the random stream is unchanged

**Files to Touch:**
- `crates/g4-physics/src/transport.rs`
- `crates/g4-physics/tests/`

**Verification Step:**

Run:
```bash
cargo test -p g4-physics
```

Expected:
All tests pass

**Notes:**

For a d_out = 0 escape, the position is the current position.
