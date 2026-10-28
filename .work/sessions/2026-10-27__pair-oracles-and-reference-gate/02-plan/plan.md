# Implementation Plan

**Session:** pair-oracles-and-reference-gate
**Date:** 2026-10-27T06:41:50.842Z
**Estimated effort:** 10-15 engineering days plus user-run Geant4 reference latency

## Strategy

Freeze the case matrix and statistical contract first. Add independent analytic anchors and sampler checks, then the neutral and charged Brownian oracles with recorded timestep convergence. Add an option3 IRT Geant4 producer without running it, and finish with a strict consumer and separate analytic, Brownian and Geant4 verdicts. R03 is complete on main. Scientific closure waits for the user's complete reference data and provenance.

## Tickets Overview

- **Ticket 1:** Commit representative cases and preregister the statistical contract.
- **Ticket 2:** Check pair laws against independent analytic and high-precision anchors, including bias rejection.
- **Ticket 3:** Implement and converge the neutral Brownian pair oracle.
- **Ticket 4:** Extend the physical Brownian oracle to charged Coulomb pairs.
- **Ticket 5:** Add the option3 IRT Geant4 producer, compile-check it and log the user run.
- **Ticket 6:** Reject malformed evidence and report separate gate verdicts.

## Sequencing Rationale

The fixed matrix precedes every statistical run. The analytic law checks establish expected probabilities before Brownian comparisons. Neutral stepping establishes boundary and convergence machinery before charged drift. The Geant4 producer and consumer follow the frozen case contract and never alter legacy V9 or RR-003 evidence.

## Risks & Mitigation

- **Risk:** Coulomb dynamics may differ from Geant4's effective-distance approximation. → **Mitigation:** Report the physical-law and source-fidelity verdicts separately; keep G1 open for a beyond-limit discrepancy.
- **Risk:** Brownian boundary discretization may bias early reactions. → **Mitigation:** Require three or more timestep levels, the frozen 0.005 limit and three-combined-SE compatibility; report INCONCLUSIVE when convergence is unresolved.
- **Risk:** Geant4 output may be incomplete or lack provenance. → **Mitigation:** Require complete-only marker, exact row/parameter checks and strict provenance before comparison; agent sessions only compile-check.

## Assumptions

- R01 and R03 are present on main, including charged PDC.
- The user performs every Geant4 run and supplies complete output and provenance.
- Existing generator modes, reference evidence, historical V9 and RR-003 verdicts remain intact.
