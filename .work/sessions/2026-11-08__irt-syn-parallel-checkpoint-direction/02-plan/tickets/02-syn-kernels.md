# Ticket 02: syn-kernels

**Acceptance Criteria:**
- [ ] Audit reuse of option3 data and kernels; implement only source-supported IRT-syn semantics.
- [ ] Test applicable reaction families, limits, contact, numerical failures and product placement.
- [ ] Add new reproducible reference modes and run requests without replacing existing files; reference acceptance waits for user-produced evidence.

**Files to Touch:**
- `crates/g4-chemistry/src/`
- `crates/g4-chemistry/tests/`
- `validation/geant4/`
- `validation/reference/`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected:
All three commands exit 0. Kernel tests pass; model-specific reference gates pass only after complete user-run evidence exists.

**Notes:**
Add focused tests for the acceptance criteria above; register exact scientific gate commands as tests and reference consumers are introduced. No blanket ignored-test execution. Preserve existing reference data. Depends on preceding tickets. New scientific functionality requires tests.
