# Ticket 01: volume-config-provenance-schema

**Effort:** medium

**Acceptance Criteria:**
- [ ] `RunConfig.volume` / `[volume]` replaces `geometry`; `[geometry]` is rejected
- [ ] `Provenance.volume: Option<Volume>`, skipped when None, round-trips; set by run from config
- [ ] Schemas are run/v5 and run-unverified/v2 (physics and chemistry)

**Files to Touch:**
- `crates/g4-sim/src/config.rs`
- `crates/g4-sim/src/history.rs`
- `crates/g4-sim/src/run.rs`
- `crates/g4-sim/src/run/chemistry.rs`
- `crates/g4-core/src/provenance.rs`
- `crates/g4-sim/tests/`

**Verification Step:**

Run:
```bash
cargo test --workspace
```

Expected:
All tests pass

**Notes:**

Search all tests and docs for `geometry` and `run/v4`.
