# Ticket 01: register-pair-cases

**Status:** ✅ Done

## Local Test Result

`cargo test -p g4-chemistry --test pair_case_matrix` passed: 2 tests, 0 failures. Cargo reported only Windows incremental-cache hard-link fallback warnings.

## Review Notes

Reviewed the eight literal option3 IDs and four synthetic cases, all four kinetic regimes, applicable A+B/A+A pairs, three separations, five positive time scales and eventual nonreaction. The contract fixes 216 analytic endpoints, 144 Geant4 endpoints, independent seeds, multiplicity, sample and refinement schedules, effect limits, and explicit INCONCLUSIVE/REQUESTED states. Historical V9 and RR-003 criteria remain separate. No inline correction was needed.

## Blockers / Challenges

None. The incremental-cache hard-link warning did not affect verification.

## Commits

- dadbe13 test: preregister R04 pair cases (ticket 01)

## Time Spent

5m (ticket-start.js to ticket-complete.js)
