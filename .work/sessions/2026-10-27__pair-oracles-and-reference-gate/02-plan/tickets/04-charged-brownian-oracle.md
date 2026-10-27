# Ticket 04: charged-brownian-oracle

**Model:** heavy
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] Extend the independent Brownian oracle to attractive/repulsive Coulomb drift and absorbing/radiation boundaries for charged TDC/PDC, including applicable A+A.
- [ ] Check drift, boundary and timestep behavior against independent high-precision physical-law anchors; report physical-versus-Geant4 approximation separately.
- [ ] Apply the frozen convergence and precision contract; any outside-limit difference leaves G1 open.

**Files to Touch:**
- `crates/g4-chemistry/tests/pair_oracle/charged.rs`
- `crates/g4-chemistry/tests/pair_brownian.rs`
- `validation/pair_oracles/analytic.py`
- `docs/validation/pair-oracle-convergence.md`

**Verification Step:**

Run:
```bash
cargo test --release -p g4-chemistry --test pair_brownian charged -- --nocapture
```

Expected:
Charged cases produce explicit PASS, FAIL or INCONCLUSIVE verdicts; a physical discrepancy cannot be hidden.

**Notes:**

Use the Coulomb potential and reaction boundary, not the IRT effective-distance approximation, in Brownian dynamics. Document numerical discretization.
