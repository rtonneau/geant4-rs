# Ticket 03: g4-sim-geometry-wiring

**Model:** medium
**Effort:** medium

**Acceptance Criteria:**
- [ ] `RunConfig.validate()` accepts geometry; `history.rs` passes `config.geometry.as_ref()`.
- [ ] `TerminationCounts` has an escaped count; existing serialised output for unbounded runs is unchanged.
- [ ] Test: a finite-volume run gives identical results on 1, 2 and 8 threads, and its ledger closes.
- [ ] docs/04 D.1/D.5 and the transport rustdoc describe the boundary, the outside-primary rule and the deferred solvation clamp.

**Files to Touch:**
- `crates/g4-sim/src/config.rs`
- `crates/g4-sim/src/history.rs`
- `crates/g4-sim/src/scoring.rs`
- `crates/g4-sim/tests/`
- `docs/04-scientific-algorithms.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected:
All pass.

**Notes:**

Chemistry stays boundary-free (G-B1); document it.
