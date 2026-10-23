# Ticket 06: document-oracles-and-open-gate

**Status:** ✅ Done

## Local Test Result

The exact verification chain passed:

`cargo fmt --all -- --check; if ($LASTEXITCODE -eq 0) { cargo clippy
--workspace --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) {
cargo test --workspace }`

Formatting produced no differences, Clippy produced no warnings, and the full
workspace test suite passed. The RR-003 consumer remained ignored with its
expected `awaiting committed two_body_h_h.csv and provenance` reason. Full log:
`.scratch/tests/2026-10-23__m7-sbs-oracle-and-ode/06-workspace-verification.log`.

## Review Notes

- Architecture and algorithm docs now distinguish production IRT from the
  validation-only source-faithful SBS and test-only Gillespie/LSODA paths.
- Recorded the executable encounter constant, inclusive SBS contact,
  observation-cap equality/bridge behavior, source bridge quirk, and strict
  IRT overlap distinction.
- Recorded the complete V10 closed system, explicit molar/volume conversion,
  512-history conservation checks, pinned public `rsoda` revision, 1% rule,
  and observed 0.0824% maximum error.
- Updated M7 status, validation, risks, dependencies, data requirements, and
  reproducibility. Every status statement keeps RR-003 `awaiting data`, rejects
  diagnostic scratch output as reference evidence, and points to RR-003 plus
  the later `m7-sbs-close-gate` session.

## Blockers / Challenges

None. The external clean Geant4 run remains an intentional future gate, not a
blocker to this local documentation ticket.

## Commits

- 991c02f docs: record M7 oracle status and open gate (ticket 06)

## Time Spent

5m (ticket-start.js to ticket-complete.js)
