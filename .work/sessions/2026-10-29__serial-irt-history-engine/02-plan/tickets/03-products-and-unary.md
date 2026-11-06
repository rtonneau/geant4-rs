# Ticket 03: products-and-unary

**Model:** heavy
**Model (Jev):** opus-5.5 (confidence 0.45)
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] Implement source-ordered products, lazy partner-time and conditional placement, unary clocks and pair/unary competition; enforce event-by-event reservoir accounting and the reaction 53 exception.

**Files to Touch:**
- `crates/g4-chemistry/src/history.rs`
- `crates/g4-chemistry/tests/history_reactions.rs`

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry --test history_reactions
```

Expected: all tests pass.

**Notes:**

Consult Geant4 source and document any deviation. Prerequisite satisfied: G1-F PASS at RR-005 200k (closure `7071bd3`); see ../plan.md for evidence and source-version scope. This does not pass R05/G2 or independent physical validation.
