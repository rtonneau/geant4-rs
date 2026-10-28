# Implementation Plan

**Session:** serial-irt-history-engine
**Date:** 2026-10-28T23:36:14.673Z
**Estimated effort:** 10–20 days (rough XL estimate; external G1 evidence excluded)

## Strategy

Complete the R05 vertical slices in order and preserve the approved grill. Implementation is blocked until R04/G1 passes.

## Tickets Overview

- **Ticket 1:** Pair history and event storage.
- **Ticket 2:** Competition, ties and stale events.
- **Ticket 3:** Products, unary reactions and accounting.
- **Ticket 4:** Observations, traces and resource limits.
- **Ticket 5:** Spatial search and all-pairs oracle.
- **Ticket 6:** Full-network and reproducibility checks.
- **Ticket 7:** Cutoff convergence and G2 evidence.

## Sequencing Rationale

Each slice builds on the preceding history semantics; the spatial and full-network evidence tests the completed engine. The production pipeline remains R06.

## Risks & Mitigation

- **Risk:** G1 is open. → **Mitigation:** Do not implement any ticket until G1 passes.
- **Risk:** Search cutoff bias or insufficient statistical precision. → **Mitigation:** Preregister and apply R05 criteria; report INCONCLUSIVE when evidence cannot resolve them.
- **Risk:** Source semantics and physical approximation diverge. → **Mitigation:** Consult Geant4 11.4.1 source and document intentional deviations and limits.

## Assumptions

- Follow the approved grill and roadmap R05, including all tests, acceptance criteria, exclusions and physics-only build requirements.
- Preserve reference files and do not run Geant4 generators in agent sessions.
