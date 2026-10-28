# Ticket 02: analytic-pair-laws

**Model:** heavy
**Effort:** light

**Acceptance Criteria:**
- [ ] Evaluate neutral formulas analytically and charged/PDC CDF and eventual nonreaction through independently computed high-precision anchors; never call the production sampler to construct expectations.
- [ ] Run preregistered >=100,000-sample tests per case with pointwise z, Bonferroni family alpha 0.001 and separate analytic verdicts.
- [ ] Reject a deliberately biased sampler.

**Files to Touch:**
- `validation/pair_oracles/analytic.py`
- `crates/g4-chemistry/tests/fixtures/pair_analytic_anchors.json`
- `crates/g4-chemistry/tests/pair_analytic.rs`
- `docs/validation/pair-case-matrix.md`

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry --test pair_analytic
```

Expected:
Analytic comparisons pass and the injected bias is rejected.

**Notes:**

Record mpmath version, precision, equations, source paths and a reproducible anchor command. Keep physical Coulomb and Geant4 effective-distance laws distinguishable.
