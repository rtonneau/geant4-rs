# Ticket 02: m3-reference-validation

**Model:** heavy
**Effort:** high

**Acceptance Criteria:**
- [ ] M3 reference CSV and provenance validation rejects missing or empty files, malformed or duplicate rows, absent keys, nonfinite/negative invalid values, wrong units/process/constructor/seed metadata and invalid count moments.
- [ ] Exact coverage is required for the 214-energy times five-process cross-section grid, seven slowing-down summary energies, 7 x 4 x 72 spectrum rows and 7 x 42 radial rows.
- [ ] Negative tests prove every malformed-evidence class, including a duplicate key replacing a missing key, fails; valid fixtures retain their existing verdict.
- [ ] Both maintained ignored M3 comparisons pass with their existing thresholds and documented low-floor and low-effective-sample exclusions.

**Files to Touch:**
- `crates/dna-sim/tests/slowing_down_reference.rs`
- `crates/dna-sim/tests/` (focused fixture helpers only if needed)

**Verification Step:**

Run:
```bash
cargo test -p dna-sim --test slowing_down_reference --locked
cargo test -p dna-sim --test slowing_down_reference --locked -- --ignored
```

Expected:
All nonignored machinery/negative tests pass; two ignored M3 comparisons run and pass, with no changed scientific threshold or exclusion.

**Notes:**

Validate each source grid before it is projected into spectra or radial arrays, since array assignment currently lets later duplicate rows overwrite earlier rows. Do not alter reference files.
