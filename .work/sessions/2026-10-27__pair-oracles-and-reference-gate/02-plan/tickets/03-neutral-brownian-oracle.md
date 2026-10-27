# Ticket 03: neutral-brownian-oracle

**Model:** heavy
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] Add test-only independent Brownian stepping for neutral TDC and PDC with absorbing/radiation boundaries; do not call production diffusion, SBS or IRT sampling code.
- [ ] Record three or more timestep levels, >=10,000 independent pairs/case, precommitted precision doubling to at most 1,000,000, <0.005 final CDF difference and three-combined-SE compatibility.
- [ ] Emit INCONCLUSIVE when refinement or precision is inadequate; retain per-case output for the convergence report.

**Files to Touch:**
- `crates/g4-chemistry/tests/pair_oracle/mod.rs`
- `crates/g4-chemistry/tests/pair_oracle/neutral.rs`
- `crates/g4-chemistry/tests/pair_brownian.rs`
- `docs/validation/pair-oracle-convergence.md`

**Verification Step:**

Run:
```bash
cargo test --release -p g4-chemistry --test pair_brownian neutral -- --nocapture
```

Expected:
Neutral cases report preregistered convergence verdicts with no silent pass on insufficient precision.

**Notes:**

Boundary-hit handling must be independent of the existing source-faithful SBS stepper; preserve no-reaction mass and contact conventions.
