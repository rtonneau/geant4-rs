# Ticket 04: runconfig-geometry

**Model:** medium
**Effort:** medium

**Acceptance Criteria:**
- [ ] `RunConfig.geometry: Option<Volume>`, defaulting to None, with existing outputs unchanged.
- [ ] `validate()` rejects `Some(_)` with a clear InvalidConfig message (transport does not honour boundaries yet).
- [ ] A TOML or serde round-trip test covers each shape, and invalid values are rejected at deserialisation.
- [ ] The geometry module, its units, tolerances, source paths and limitations are documented in docs/04 (or docs/02).
- [ ] fmt, clippy -D warnings and the workspace tests all pass.

**Files to Touch:**
- `crates/g4-sim/src/config.rs`
- `docs/04-scientific-algorithms.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected:
All three commands exit 0.

**Notes:**

If RunConfig is only Serialize today, add only what the round-trip needs. Do not wire the volume into transport.
