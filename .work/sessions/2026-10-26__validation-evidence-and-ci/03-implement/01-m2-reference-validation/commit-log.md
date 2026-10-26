# Ticket 01: m2-reference-validation

**Status:** ✅ Done

## Local Test Result

`cargo test -p dna-sim --test geant4_reference --locked`: 11 passed, 4 ignored.
`cargo test -p dna-sim --test geant4_reference --locked -- --ignored`: 4 passed.
`cargo clippy -p dna-sim --test geant4_reference --locked -- -D warnings`,
`cargo fmt --all -- --check`, and `git diff --check`: passed.

## Review Notes

The implementation subagent used `gpt-6-luna/high`. Checked the M2 energy/process grid against the registered Geant4 generator and committed
reference files, the provenance version, malformed-data rejection, and statistical
acceptance boundaries. Inline review added three regression tests and fixes for unequal
deterministic means with zero standard error, one-ULP energy rounding between platforms,
and conflicting repeated endpoint values. All four ignored M2 reference comparisons
passed locally on Windows. Scientific thresholds and reference files are unchanged.

## Blockers / Challenges

No blocker. C++ `std::pow` and Rust `powf` can differ by one ULP; the grid validator
now permits that numerical rounding while requiring the exact registered row count
and process/energy multiset.

## Commits

- e75d2d8 test: validate M2 reference evidence (ticket 01)

## Time Spent

25m (ticket-start.js to ticket-complete.js)
