# Implementation Plan

**Session:** serial-irt-history-engine
**Date:** 2026-10-28T23:36:14.673Z
**Estimated effort:** 10–20 days (rough XL estimate; external G1 evidence excluded)

## Strategy

Complete the R05 vertical slices in order and preserve the approved grill. G1-F passed on 2026-11-05 (RR-005, Geant4 11.4.3, 200k pairs; closure commit `7071bd3`). The approved G1-F replacement prerequisite is satisfied; begin with Ticket 01 when implementation is requested.

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

- **Prerequisite resolved:** G1-F PASS (`7071bd3`) permits real-model integration. The original combined G1 was not passed; physical-law limitations and R05/G2 acceptance remain open.
- **Risk:** Search cutoff bias or insufficient statistical precision. → **Mitigation:** Preregister and apply R05 criteria; report INCONCLUSIVE when evidence cannot resolve them.
- **Risk:** Source semantics and physical approximation diverge. → **Mitigation:** Consult Geant4 11.4.1 source and document intentional deviations and limits.

## Assumptions

- Follow the approved grill and roadmap R05, including all tests, acceptance criteria, exclusions and physics-only build requirements.
- Preserve reference files and do not run Geant4 generators in agent sessions.

## Prerequisite evidence and source scope

RR-005 at 200k: 120 included endpoints PASS, 24 exact PDC contact endpoints
excluded. Analytic and contact regressions passed; no further pair expansion
is needed. Data `51c22b6`, provenance `0b96afe`, closure `7071bd3`.
See `docs/validation/g1-f-source-fidelity.md` and `docs/08-roadmap.md`.

RR-005 uses Geant4 11.4.3; the planned R05 source baseline remains 11.4.1.
Verify the relevant source/version and document differences when implementing
each behavior; do not infer that the entire scheduler was validated by RR-005.
All seven ticket acceptance criteria remain unchanged. R05/G2 convergence,
R06 production integration and R07 full-track acceptance remain separate.
