# Ticket 08: parallel-syn

**Acceptance Criteria:**
- [ ] Use shared-memory workers for candidate work with coordinator-controlled time advancement and conflict resolution.
- [ ] Prevent shared-parent double consumption, stale proposals and duplicate RNG assignment.
- [ ] Compare one and multiple workers and alternative partitions; test checkpoint consistency with workers quiesced.
- [ ] Profile representative dense and sparse volumes; record speed and memory without assuming improvement.

**Files to Touch:**
- `crates/g4-chemistry/src/`
- `crates/g4-sim/src/`
- `crates/g4-chemistry/tests/`
- `docs/05-parallelisation.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected:
All three commands exit 0. Worker-count, boundary and serial-equivalence gates pass; benchmark report records bottlenecks.

**Notes:**
Add focused tests for the acceptance criteria above; register exact scientific gate commands as tests and reference consumers are introduced. No blanket ignored-test execution. Preserve existing reference data. Depends on preceding tickets. New scientific functionality requires tests.
