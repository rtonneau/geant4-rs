# Ticket 01: m2-reference-validation

**Model:** heavy
**Effort:** high

**Acceptance Criteria:**
- [ ] M2 reference CSV and provenance validation rejects missing or empty files, malformed or duplicate rows, absent keys, nonfinite/negative invalid values, wrong units/process/constructor/seed metadata and invalid count moments with actionable errors.
- [ ] The cross-section grid has exactly 208 energies times three M2 processes; MFP and track reference energies exactly match their registered 9- and 5-energy lists, including the independent second-seed files.
- [ ] Negative tests prove a deliberately perturbed valid fixture fails and every listed malformed-evidence class is rejected; a valid in-memory fixture passes.
- [ ] All four maintained ignored M2 comparisons pass with their existing tolerances and exclusions.

**Files to Touch:**
- `crates/dna-sim/tests/geant4_reference.rs`
- `crates/dna-sim/tests/` (focused fixture helpers only if needed)

**Verification Step:**

Run:
```bash
cargo test -p dna-sim --test geant4_reference --locked
cargo test -p dna-sim --test geant4_reference --locked -- --ignored
```

Expected:
All nonignored machinery/negative tests pass; four ignored M2 comparisons run and pass, with no changed scientific threshold.

**Notes:**

Use an expected grid independent of the parsed fixture; a row count alone cannot detect a duplicate replacing a missing key. Preserve the existing `ZERO_FLOOR` and V2/V3/V5 criteria. Do not modify reference files.
