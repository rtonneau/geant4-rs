# Ticket 02: competing-events

**Model:** heavy
**Model (Jev):** opus-5.5 (confidence 0.19)
**Effort:** high

**Acceptance Criteria:**
- [ ] Add canonical pair scheduling, A+A identity checks, exact tie insertion order and stale-event invalidation while preserving unaffected events; test triplet competition and lineage.

**Files to Touch:**
- `crates/g4-chemistry/src/history.rs`
- `crates/g4-chemistry/tests/history_competition.rs`

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry --test history_competition
```

Expected: all tests pass.

**Notes:**

G1 must pass before implementation.
