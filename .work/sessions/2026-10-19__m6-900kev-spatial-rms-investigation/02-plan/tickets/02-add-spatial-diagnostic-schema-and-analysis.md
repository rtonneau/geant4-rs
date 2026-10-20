# Ticket 02: add-spatial-diagnostic-schema-and-analysis

**Model:** heavy
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] A focused Rust module defines the five diagnostic boundaries, category/species keys, stable identity fields, count and spatial moments, maximum radius, and the fixed half-decade radial bins, with CSV serialization and strict parsing/completeness validation.
- [ ] The analyzer computes the all-shared-final-survivor pooled RMS ratio as the sole primary estimand, plus documented secondary count, per-history/per-species RMS, radial, maximum, longitudinal, and transverse summaries.
- [ ] A deterministic hierarchical bootstrap resamples seed groups and histories within groups for exactly 20,000 replicates, returns a 95% primary interval, and computes simultaneous stage intervals from the maximum absolute log-ratio deviation.
- [ ] Unit tests use small synthetic datasets to prove pooled weighting, seed/history resampling determinism, interval construction, first-detected-divergence wording, schema rejection of missing/duplicate rows, and radial underflow/overflow handling.
- [ ] No new crate dependency is added unless separately justified; existing scientific behavior and validation thresholds remain unchanged.

**Files to Touch:**
- `crates/dna-sim/src/spatial_diagnostic.rs`
- `crates/dna-sim/src/lib.rs`
- `crates/dna-sim/tests/spatial_diagnostic.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-sim --test spatial_diagnostic
```

Expected:
All schema and bootstrap-analysis tests pass, including deterministic repeated output for the recorded analysis seed.

**Notes:**

Use independent Rust and Geant4 samples; do not manufacture paired histories from equal numeric seeds. Keep the statistical API diagnostic-only and separate from the normal run JSON schema. Prefer existing `rand`/`rand_chacha` facilities and explicit finite-value checks.
