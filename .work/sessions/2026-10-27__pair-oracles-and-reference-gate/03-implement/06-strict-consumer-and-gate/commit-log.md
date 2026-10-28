# Ticket 06: strict-consumer-and-gate

**Status:** ✅ Done

## Local Test Result

`cargo test -p g4-chemistry --test geant4_pair_r04`: 5 passed, 1 ignored (RR-004 user-run evidence REQUESTED). Parent reran `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; all exited 0. Cargo emitted Windows incremental-cache hard-link copy warnings only.

## Review Notes

Reviewed gpt-6-sol (high) parser, exact row ordering/coverage, source parameter checks, completion marker, count invariants, SHA-256 and provenance validation, independent Rust seed and Bonferroni/effect/precision verdict. Tests reject malformed and biased evidence. Roadmap and validation docs keep ANALYTIC, BROWNIAN and GEANT4 verdicts separate, with G1 OPEN. Existing V9 and RR-003 tests and fixtures are untouched. No parent code changes.

## Blockers / Challenges

RR-004 CSV and provenance require a user-run Geant4 generator and are absent. The Brownian physical-law verdict also remains INCONCLUSIVE, and charged physical-versus-approximation discrepancies are recorded. Neither condition is interpreted as G1 PASS.

## Commits

- d91567e test: add strict R04 pair reference gate (ticket 06)

## Time Spent

22m (ticket-start.js to ticket-complete.js)
