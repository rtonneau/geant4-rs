# Ticket 05: spatial-index

**Model:** heavy
**Model (Jev):** opus-5.5 (confidence 0.46)
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] Add canonical finite-horizon indexed candidate search and a small-system all-pairs oracle, the source cutoff formula and configurable multiplier; measure memory, event and candidate counts and test duplicate exclusion.

**Files to Touch:**
- `crates/g4-chemistry/src/history.rs`
- `crates/g4-chemistry/src/history_spatial.rs`
- `crates/g4-chemistry/tests/history_spatial.rs`

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry --test history_spatial
```

Expected: all tests pass.

**Notes:**

Never treat omitted probability as zero. Prerequisite satisfied: G1-F PASS at RR-005 200k (closure `7071bd3`); see ../plan.md for evidence and source-version scope. This does not pass R05/G2 or independent physical validation.
