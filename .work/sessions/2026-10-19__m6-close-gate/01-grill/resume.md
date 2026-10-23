# Session: m6-close-gate

**Date:** 2026-10-19T17:20:29.676Z
**Status:** Grill phase complete

## Problem Statement

Formally decide whether the validated M6 physicochemical output may be used as the 1 ps initial condition for M7 and M8. The stage-matched Geant4 comparison and subsequent residual audit are complete, but the project-lead closure decision has not been recorded in the GPS session.

## Context & Constraints

- **Current behavior:** The corrected Rust recombination schedule passes the stage-matched RR-001 gate against Geant4 11.4.1 for all 28 species/energy rows with unchanged criteria; the maximum absolute z-score is 2.04. The later RR-002 spatial diagnostic records additional evidence and explicitly neither reopens nor closes M6.
- **Pain point:** M7 and M8 remain procedurally blocked from consuming the M6 species and positions until the project lead formally lifts the M6 gate.
- **Dependencies:** Preserve the committed Geant4 data, provenance, comparison thresholds, source audit, retained-difference register, and the distinction between the pre-IRT stage-matched snapshot and the historical post-zero-time-reaction snapshot.
- **Tech stack:** Rust `dna-core`, `dna-chemistry`, and `dna-sim`; Geant4 11.4.1 option2 physics and option3 chemistry; committed RR-001 reference/provenance files and ignored comparison tests.

## Success Metrics

- The RR-001 provenance and stage-boundary guards remain valid and the stage-matched comparison passes 28/28 rows with the frozen criteria.
- The project lead explicitly accepts the documented residual assumptions and declares M6 formally closed for downstream consumption of the 1 ps species and positions.
- The closure record does not claim that RR-002 closes M6, that the historical post-reaction snapshot is pre-IRT, or that retained risks outside the gate have been eliminated.

## Architecture & Approach

Treat the committed RR-001 comparison and residual audit as the quantitative closure evidence. Record the project-lead decision to lift the gate without changing code, reference data, tolerances, or scientific models. Retain Q21/R14's documented assumptions and the separate RR-002 investigation as follow-up evidence rather than closure criteria.

## Assumptions & Trade-offs

The closure accepts a single Geant4 seed for RR-001, no per-event covariance, and source-derived rather than runtime-traced details of event ordering and failed-recombination retesting. Those limitations remain documented risks; they do not block the project lead's decision because the stage-matched gate passes unchanged and no contradictory evidence has been established.

## Open Questions

None for the M6 gate. The retained Q21/R14 assumptions and RR-002 spatial investigation remain documented follow-up work and must not be silently reclassified as resolved.

## Notes

The project lead confirmed in this conversation that the Geant4 simulations and comparisons were performed and directed that M6 be formally closed. This bounded session records that decision; it does not create a feature branch or implementation tickets.
