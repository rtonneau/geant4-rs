# Ticket 03: escape-summary

**Effort:** medium

**Acceptance Criteria:**
- [ ] `summary.escape` is present only with a volume: primary fraction ± binomial SE, tracks per history, energy fraction, spectra (primary/secondaries/all)
- [ ] Unbounded summary JSON has no `escape` key

**Files to Touch:**
- `crates/g4-sim/src/history.rs`
- `crates/g4-sim/src/scoring.rs`
- `crates/g4-sim/tests/`

**Verification Step:**

Run:
```bash
cargo test -p g4-sim
```

Expected:
All tests pass

**Notes:**

Reuse `spectrum_edges_ev` and SpectrumAcc.
