# Ticket 09: cli

**Acceptance Criteria:**
- [ ] Provide supported configuration and primary-list input, outputs, stop-at-safe-boundary and resume commands.
- [ ] Make validated spatial parallel chemistry the default with explicit one-worker mode.
- [ ] Report model/version/data provenance and completed versus incomplete status; document usage.

**Files to Touch:**
- `Cargo.toml`
- `crates/`
- `docs/`
- `README.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected:
All three commands exit 0. CLI integration tests cover fresh run, requested stop, valid resume and invalid inputs.

**Notes:**
Add focused tests for the acceptance criteria above; register exact scientific gate commands as tests and reference consumers are introduced. No blanket ignored-test execution. Preserve existing reference data. Depends on preceding tickets. New scientific functionality requires tests.
