# Ticket 06: v9-validation-and-reproducibility

**Model:** heavy
**Model (Jev):** opus-5.5 (confidence 0.42)
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] The test header pre-registers one family of 18 comparisons: six dimensionless time points for each of the three agreed option3 regimes, at least 10,000 pairs per regime, and an absolute 3-sigma threshold at every point.
- [ ] Empirical CDF/survival tests cover neutral A+A TDC `H + H`, neutral A+B PDC `°OH + H`, and charged A+B TDC `H3Op + OHm`, including each law's no-reaction mass.
- [ ] Separate tests cover A+A exchange symmetry, same-identity rejection, contact/extreme-value policies, and the charged effective-distance interpretation.
- [ ] For one fixed workload and seed, serialized outcomes and diffused positions are bitwise identical with 1, 2, and 8 Rayon threads using per-history `Stage::Chemistry` streams.

**Files to Touch:**
- `crates/dna-chemistry/tests/two_body_validation.rs`
- `crates/dna-chemistry/tests/thread_independence.rs`
- `crates/dna-chemistry/tests/common/mod.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-chemistry --test two_body_validation --test thread_independence
```

Expected:
All 18 pre-registered analytic comparisons, boundary checks, and 1/2/8-thread bitwise comparisons pass.

**Notes:**

Freeze seeds, sample sizes, time points, families, and thresholds in the test headers before the first statistical run. The charged case is a Geant4 effective-distance fidelity test, not a general exact electrostatic claim. Use TDD and commit as `test: validate two-body IRT statistics and reproducibility`.
