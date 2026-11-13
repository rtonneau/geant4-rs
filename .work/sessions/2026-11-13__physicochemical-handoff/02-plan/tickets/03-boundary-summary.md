# Ticket 03: boundary-summary

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] `summary.boundary`, present only with a volume (zeros included), reports per-species counts: shortened placements, outside after placement, outside at 1 ps, outside at the end of chemistry.
- [ ] Schema bumped to run/v6; unbounded output unchanged apart from the schema string.
- [ ] Counts do not depend on the thread count.

**Files to Touch:**
- `crates/dna-sim/src/`
- `crates/dna-sim/tests/`
- `docs/04-scientific-algorithms.md`

**Verification Step:**

Run:
```bash
cargo test --workspace
```

Expected:
All tests pass.

**Notes:**

Mirror how `summary.escape` is gated and serialised.
