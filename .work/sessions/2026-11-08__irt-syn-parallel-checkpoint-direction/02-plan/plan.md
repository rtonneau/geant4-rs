# Implementation Plan

**Session:** irt-syn-parallel-checkpoint-direction
**Date:** 2026-11-08T21:27:30.510Z
**Estimated effort:** Not yet credibly estimable; re-estimate after reference-contract and serial-syn gates.

## Strategy

Start the implementation branch from verified up-to-date main, not fix/g2-supervisor-recovery. Preserve that branch and its four current extra commits; selectively reuse only reviewed necessary work. Establish scientific semantics before kernels, serial evolution before parallel evolution, and restartable state before workers. No implementation is authorized by this draft. Geant4 generator execution remains user-owned.

## Tickets Overview

- **Ticket 1:** Establish IRT-syn reference and acceptance contract
- **Ticket 2:** Implement and validate IRT-syn reaction kernels
- **Ticket 3:** Build serial synchronized chemistry state machine
- **Ticket 4:** Integrate explicit timed electron irradiation
- **Ticket 5:** Add time-dependent shared-volume scoring
- **Ticket 6:** Implement synchronized checkpoint and restart
- **Ticket 7:** Introduce transparent computational regions
- **Ticket 8:** Execute spatial chemistry with coordinator and workers
- **Ticket 9:** Expose run stop resume and parallel defaults
- **Ticket 10:** Validate integrated electron IRT-syn release

## Sequencing Rationale

Tickets execute in the listed order. Each is independently reviewable and leaves workspace checks passing. Reference evidence is a scientific gate, not a reason to fabricate results. Tickets 02 and 10 may wait on user-run generators. Ticket 08 requires the serial and region oracles; ticket 09 enables defaults only after parallel validation.

## Risks & Mitigation

- **Risk:** Option3 and IRT-syn differ → **Mitigation:** source-defined contract and separate validation.
- **Risk:** Cross-region conflicts or missing pairs bias chemistry → **Mitigation:** canonical ownership, coordinator arbitration and all-pairs comparisons.
- **Risk:** Restart resampling changes conditional probabilities → **Mitigation:** retain sufficient scheduling state and test conditional continuation.
- **Risk:** Timed injections corrupt chronology or normalization → **Mitigation:** explicit handoff timeline and observation conventions.
- **Risk:** Coordinator limits speedup → **Mitigation:** measure sparse and dense workloads before performance claims.
- **Risk:** Existing recovery work is accidentally inherited → **Mitigation:** branch from main and preserve recovery separately.

## Assumptions

- Approved scope: electron-only, single-machine shared-memory, interacting tracks, unbounded water, explicit timed primaries, IRT-syn production chemistry and statistical restart.
- Exact scientific ranges and tolerances are established before evaluation in ticket 01.
- Branch creation waits for ticket approval and verification of remote main.
