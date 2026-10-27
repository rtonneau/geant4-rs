# Ticket 01: register-pair-cases

**Model:** medium
**Effort:** medium

**Acceptance Criteria:**
- [ ] Commit the literal option3 and dimensionless case matrix before statistical runs; cover all four regimes, applicable A+B/A+A, contact/1.5/3 radii, five observation scales and eventual nonreaction.
- [ ] Freeze seeds, sample sizes, family alpha 0.001, multiplicity, refinement and precision schedules, effect limits and INCONCLUSIVE handling.

**Files to Touch:**
- `docs/validation/pair-case-matrix.md`
- `docs/07-validation-strategy.md`
- `crates/g4-chemistry/tests/pair_case_matrix.rs`

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry --test pair_case_matrix
```

Expected:
All matrix and contract tests pass.

**Notes:**

Preserve historical V9/RR-003 criteria. Name literal reaction IDs from the 72-ID execution matrix; label synthetic cases distinctly.
