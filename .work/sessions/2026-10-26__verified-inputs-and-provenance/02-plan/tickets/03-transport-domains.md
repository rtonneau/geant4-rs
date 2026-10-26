# Ticket 03: transport-domains

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] `transport_electron` rejects non-finite primary position components and ledger tolerances that are NaN, infinite or negative before any RNG draw.
- [ ] Every macroscopic cross section and their total are finite and nonnegative; overflow or invalid model output is a typed error rather than a silent transport branch.
- [ ] Existing valid histories, ledger results and RNG draws are unchanged.

**Files to Touch:**
- `crates/g4-physics/src/transport.rs`
- `crates/g4-physics/src/table.rs`
- `crates/g4-physics/src/error.rs`
- `crates/g4-physics/tests/transport.rs`

**Verification Step:**

Run:
```bash
cargo test -p g4-physics --locked
```

Expected:
All tests pass; invalid input tests return typed errors before sampling and valid deterministic transport fixtures match their prior results.

**Notes:**

Do not change physical cross-section formulas or the existing ledger acceptance threshold. Check the complete position vector and tolerance at the public transport boundary. Keep diagnostic error context useful to callers.
