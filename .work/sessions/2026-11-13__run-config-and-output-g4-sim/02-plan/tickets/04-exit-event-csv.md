# Ticket 04: exit-event-csv

**Effort:** medium

**Acceptance Criteria:**
- [ ] `EscapeSink` trait + `CsvEscapeWriter<W>`, one row per exit event (history, generation, x,y,z, ux,uy,uz, energy_ev)
- [ ] Byte-identical across thread counts

**Files to Touch:**
- `crates/g4-sim/src/run.rs`
- `crates/g4-sim/src/history.rs`
- `crates/g4-sim/src/lib.rs`
- `crates/g4-sim/tests/`

**Verification Step:**

Run:
```bash
cargo test -p g4-sim
```

Expected:
All tests pass

**Notes:**

Mirror the physchem sink plumbing.
