# Ticket 03: m6-stage-evidence

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] The named stage-matched M6 reader rejects wrong snapshot boundary, constructor, seed, missing/duplicate species-energy row, invalid count or spatial moment and incomplete metadata.
- [ ] When combined standard error is zero, unequal finite means fail and equal means pass; nonfinite comparison quantities fail rather than bypassing the z test.
- [ ] Negative tests cover those cases without changing the registered z <= 4.5, 15% count and 20% RMS criteria.
- [ ] The named M6 prechem ignored test passes; historical post-zero-time and scratch-producing M6 diagnostics remain excluded.

**Files to Touch:**
- `crates/dna-sim/tests/physchem_reference.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-sim --test physchem_reference --locked
cargo test -p dna-sim --test physchem_reference --locked m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact
```

Expected:
New nonignored negative tests and the one named M6 prechem comparison pass; existing thresholds are unchanged.

**Notes:**

Keep the 1 ps pre-IRT boundary distinct from `physchem.csv` post-zero-time evidence. Preserve the qualified M6 closure and report any new harness regression separately.
