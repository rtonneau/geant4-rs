# Ticket 02: runconfig-isotropic-direction

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] `RunConfig.direction: PrimaryDirection` (untagged: `Fixed([f64;3])` | `Isotropic` as `"isotropic"`); existing TOML/JSON parse and serialise byte-identically; `"isotropic"` round-trips.
- [ ] Isotropic histories draw their direction from the `(seed, history, Source)` stream; fixed runs never create that stream.
- [ ] Existing determinism/regression tests pass unmodified; new test: isotropic run bitwise equal at 1/2/8 threads; physics stream of a history unchanged by the source mode (first draws).
- [ ] Validation rejects a zero or non-finite fixed direction if not already rejected; docs/reproducibility.md documents the Source stream; run schema bumped only if output changes.

**Files to Touch:**
- `crates/g4-sim/src/config.rs`
- `crates/g4-sim/src/run.rs`
- `crates/g4-sim/tests/determinism.rs`
- `docs/reproducibility.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected:
All clean; all tests pass.

**Notes:**

Callers that read `config.direction` as an array (tests, CLI, spatial_diagnostic) must be updated.
