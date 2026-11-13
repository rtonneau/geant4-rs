# Ticket 03: boundary-summary

**Status:** ✅ Done

## Local Test Result

`cargo test --workspace`: 0 failures (new `g4-sim/tests/boundary_summary.rs`: 2 passed, real data, not skipped). clippy -D warnings and fmt clean.

## Review Notes

Base 1835553 (ticket started late, after implementation; no other writer). Inline, reviewed inline.
- `PhysChemSummary.boundary` (skipped when None) with per-species shortened / not_inside_after_placement / outside_at_1ps / not_inside_at_1ps; e_aq solvation counts come from HistoryOutcome.
- Schema run/v5 -> v6, run-unverified/v2 -> v3; tests and docs/crate-separation.md updated.
- Thread independence: 1 vs 4 threads identical block. D2: outside_at_1ps == 0 in a 20 nm sphere.
- Deviation: 'outside at the end of chemistry' is not in the run summary, because the g4-sim run layer has no IRT stage; the 1 ps snapshot is reported and docs/04 D.6 says so.

## Blockers / Challenges

End-of-chemistry counts need IRT in the run layer (not present); left for the IRT integration.

## Commits

- 0d3cb0e feat(g4-sim): physchem.boundary placement and 1 ps boundary counts (schema run/v6) (ticket 03)

## Time Spent

0m (ticket-start.js to ticket-complete.js)
