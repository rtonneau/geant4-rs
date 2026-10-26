# Ticket 05: preregister-geant4-comparison

**Status:** ✅ Done

## Local Test Result

PASS for the Ticket 05 verification step:

- `cargo test -p dna-chemistry --test geant4_two_body`: 5 passed, 0 failed,
  1 ignored (`m7_irt_survival_matches_geant4_sbs`, awaiting committed RR-003 data).
- `cargo fmt --all -- --check` passed after formatting the new integration test.
- The corrected C++ generator rebuilt and linked against Geant4 11.4.1; an oversized
  Windows seed exited 1 without creating output.

The full `cargo test -p dna-chemistry` run reached and passed the new Ticket 05 test,
but the already in-progress Ticket 02 `sbs_validation` test failed at tau=0.25. Its
fixed result (IRT 0.002240, SBS 0.000880, bound 0.000632) is the existing Ticket 02
investigation, not a Ticket 05 regression.

## Review Notes

The ignored consumer freezes the six tau points, N=100,000, Rust and Geant4 seeds,
and the unchanged V9 combined-binomial three-sigma rule. Before sampling Rust IRT it
strictly validates the CSV header, ordered coverage, totals, monotonicity and completion
marker, then validates every physics/statistics provenance field and nonempty build
metadata. Synthetic non-ignored tests cover missing/duplicate rows, incomplete counts,
bad completion and mismatched seed/model/command provenance.

RR-003 and its exact smoke, production, provenance and close-gate commands are documented
in RUN_REQUESTS, README and BUILD_NOTES. No reference CSV or provenance was added. The
user-authorized scratch run is explicitly recorded as preceding Ticket 05, so the docs do
not make a false retrospective-preregistration claim and do not treat scratch data as
reference evidence.

The scratch run exposed two generator defects after Ticket 04: reaction callbacks used
`H` instead of Geant4's canonical `H^0`, and `strtol` overflow silently changed a large
seed on Windows. Ticket 05 includes the minimal fixes because valid RR-003 data cannot be
produced without them.

## Blockers / Challenges

Literal preregistration before external data was no longer possible because the user
explicitly authorized the scratch run first. The statistical rule was inherited unchanged
and the chronology is disclosed. The gate remains awaiting a clean rerun after the corrected
generator commit. The unrelated Ticket 02 statistical failure remains open.

## Commits

- 53643e9 test: preregister Geant4 H+H comparison (ticket 05)

## Time Spent

5m (ticket-start.js to ticket-complete.js)
