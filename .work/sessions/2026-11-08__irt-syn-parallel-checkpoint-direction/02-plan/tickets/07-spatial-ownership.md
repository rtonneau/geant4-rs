# Ticket 07: spatial-ownership

**Acceptance Criteria:**
- [ ] Implement dynamically extensible partitions for unbounded water with unique molecule and pair ownership.
- [ ] Cover cross-region candidates, migration and products without physical walls or duplicate reactions.
- [ ] Derive candidate bounds from the approved scientific contract; quantify truncation rather than treating Brownian reach as bounded.
- [ ] Validate against the serial all-pairs oracle before threading.

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
All three commands exit 0. Face, edge, corner, migration and distant-candidate tests meet predefined criteria.

**Notes:**
Add focused tests for the acceptance criteria above; register exact scientific gate commands as tests and reference consumers are introduced. No blanket ignored-test execution. Preserve existing reference data. Depends on preceding tickets. New scientific functionality requires tests.
