# Ticket 03: serial-syn

**Acceptance Criteria:**
- [ ] Implement competing reactions, identity, invalidation, unary channels and synchronized evolution.
- [ ] Expose resumable advance operations with typed errors and explicit state; retain a small all-pairs oracle.
- [ ] Validate ordering, conservation/accounting and many-molecule competition; do not claim option3 tests validate IRT-syn.

**Files to Touch:**
- `crates/g4-chemistry/src/`
- `crates/g4-chemistry/tests/`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected:
All three commands exit 0. Serial chemistry tests pass against the ticket-01 contract and oracle.

**Notes:**
Add focused tests for the acceptance criteria above; register exact scientific gate commands as tests and reference consumers are introduced. No blanket ignored-test execution. Preserve existing reference data. Depends on preceding tickets. New scientific functionality requires tests.
