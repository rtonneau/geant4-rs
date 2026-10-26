# Ticket 02: irt-sbs-statistical-oracle

**Model:** medium
**Model (Jev):** sonnet-5.5 (confidence 0.16)
**Effort:** high

**Acceptance Criteria:**
- [ ] Compare production IRT with SBS for option3 H+H reaction ID 1 at `r0 = 0.2 nm` and `tau = [0.25, 0.5, 1, 2, 4, 16]`.
- [ ] Use 100,000 independent pairs per sampler, fixed independent deterministic streams, and a test header that pre-registers the combined three-sigma binomial comparison family.
- [ ] Survival includes every pair not reacted by the observation time, and all six absolute CDF/survival differences satisfy the registered bounds.
- [ ] The test performs no seed search, fitting, or post-run point or tolerance adjustment.

**Files to Touch:**
- `crates/dna-chemistry/tests/sbs_validation.rs`
- `crates/dna-chemistry/tests/common/mod.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-chemistry --test sbs_validation -- --nocapture
```

Expected:
Six pre-registered IRT/SBS comparisons pass and print their residuals.

**Notes:**

Use the time scale and parameters already fixed by V9. Do not make the statistical test depend on the future Geant4 CSV.
