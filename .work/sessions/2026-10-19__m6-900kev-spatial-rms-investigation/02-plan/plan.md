# Implementation Plan

**Session:** m6-900kev-spatial-rms-investigation
**Date:** 2026-10-19T20:27:49.365Z
**Estimated effort:** 4-6 days

## Strategy

Freeze the source-backed N2 and statistical contract before adding code. Implement and test the shared Rust schema and bootstrap analysis independently of production instrumentation, then observe the five Rust lifecycle boundaries without perturbing model behavior. Add the matching separately named Geant4 mode and compile-check it without running it. Finish by documenting the reproducible user-run request, provenance requirements, and awaiting-data consumer.

## Tickets Overview

- **Ticket 1:** Audit N2 and freeze the five-boundary diagnostic and statistical contract.
- **Ticket 2:** Add the long-form spatial diagnostic schema and deterministic bootstrap analysis.
- **Ticket 3:** Instrument the five Rust boundaries without changing scientific behavior.
- **Ticket 4:** Add and compile-check the separate Geant4 spatial diagnostic mode.
- **Ticket 5:** Document the mode and queue the user-run Geant4 reference request.

## Sequencing Rationale

The source audit settles observable semantics and locks the analysis before diagnostic data exist. The schema and analyzer then give both implementations one tested contract. Rust instrumentation can be checked for non-perturbation before the C++ mode mirrors it. Documentation and the run request come last so their commands and schemas describe compile-checked code. The user-run data and any later scientific conclusion are deliberately deferred.

## Risks & Mitigation

- **Risk:** Diagnostic hooks alter RNG consumption or lifecycle ordering. → **Mitigation:** Keep observers diagnostic-only and add bitwise non-perturbation and thread-count tests.
- **Risk:** Successive boundaries contain different populations and invite causal overstatement. → **Mitigation:** Report only the first detected divergence and combine simultaneous intervals with the source trace.
- **Risk:** The 900 keV sample is too imprecise. → **Mitigation:** Use the preregistered 40/80/160 histories-per-seed rule and stop at 800 total histories per implementation.
- **Risk:** A Geant4 boundary cannot be observed safely. → **Mitigation:** Stop and document the source limitation rather than approximate it or perturb the scheduler.
- **Risk:** New data overwrite historical evidence. → **Mitigation:** Use a new mode, CSV, provenance file, and run request; never modify existing modes or reference files.

## Assumptions

- Geant4 11.4.1 is the reference and `DEV_DIR`/`G4_ROOT` plus `.claude/.claude-project.json` resolve all machine paths.
- Rust and Geant4 samples are independent even when numeric seed labels match.
- The project lead, not the agent, runs the Geant4 executable and commits complete data and provenance.
- Existing M6 gates, tolerances, and closure status remain unchanged throughout this session.
