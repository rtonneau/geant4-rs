# Implementation Plan

**Session:** physics-only-simulation-build
**Date:** 2026-10-26T16:01:37.326Z
**Estimated effort:** 2-3 developer-days

## Strategy

Implement the optional chemistry boundary and its consumer tests first, preserving the default build and snapshots. Then extend the snapshot checker to prove the resolved dependency boundary, add no-default CI checks, run the full scientific regression matrix and record evidence. S02 is complete; there is no further prerequisite.

## Tickets Overview

- **Ticket 1:** Make chemistry optional in `g4-sim` while keeping physical orchestration and raw handoff available in both profiles.
- **Ticket 2:** Enforce the no-default dependency boundary, compare exact snapshots and add CI and verification documentation.

## Sequencing Rationale

Ticket 2 checks the feature boundary introduced by ticket 1. Each ticket ends in a compilable, tested commit, matching Tasks 4 and 5 of the approved separation plan.

## Risks & Mitigation

- **Risk:** Moving coupled run code changes physical RNG order, event placement or default chemistry output. → **Mitigation:** Keep shared physical helpers and batching unchanged; compare exact same-platform physics and physchem snapshots with the pre-S01 baseline.
- **Risk:** Cargo feature unification or dev dependencies conceal a chemistry edge. → **Mitigation:** Inspect package-scoped resolved graphs and run separate no-default `--all-targets` tests and Clippy in CI.
- **Risk:** Feature gates silently exclude tests. → **Mitigation:** Add explicit physical and chemistry consumers and record selected test counts for both profiles.

## Assumptions

- Use the approved Tasks 4–5, pinned toolchain and committed data. Do not run Geant4 generators, change reference data or add dependencies.
- Default public paths and scientific formats remain stable; no-default omits chemistry-only APIs and rejects `--physchem` clearly.
- No biology placeholder, new simulation mode, CLI or performance change is included.
