# Implementation Plan

**Session:** m6-n1-thermalisation-audit
**Date:** 2026-10-17T12:56:36.779Z
**Estimated effort:** 3-5 hours

## Strategy

Trace the active Geant4 11.4.1 thermalisation path and compare it with Rust, then decide N1 from the geometry and existing test evidence. Update only contradictory or incomplete documentation. Add a regression test only if the current focused tests leave an agreed invariant uncovered. Finish with the targeted tests and the full repository checks.

## Tickets Overview

- **Ticket 1:** Audit the N1 boundary-dependent thermalisation clip, verify existing invariants, and record the source-backed verdict.

## Sequencing Rationale

This is one cohesive scientific audit: the source reading determines the verdict, the existing tests establish the Rust invariants, and the documentation records both. Splitting those steps would permit an intermediate commit containing a claim without its evidence or evidence without its conclusion.

## Risks & Mitigation

- **Risk:** The audit mistakes a workload-specific no-op for dead Geant4 code. → **Mitigation:** Record separately that the clip is active and that its condition is unreachable only for the infinite-water model and committed reference geometry.
- **Risk:** The work grows into finite-geometry support or the N2 audit. → **Mitigation:** Stop at the agreed N1 verdict; block rather than expanding scope if existing evidence cannot decide it.
- **Risk:** Documentation claims more than the available statistics support about the 900 keV RMS excess. → **Mitigation:** Exclude N1 as its cause but leave the excess unresolved and retain the existing tolerance and M6-closure boundary.

## Assumptions

- The committed M6 reference generator's 1 m water sphere and centred source define the geometry used by the reference CSV.
- The project continues to model infinite liquid water; finite boundaries and navigator abstractions are out of scope.
- Existing Meesungnoen sampling, terminal-electron, physicochemical-event, and `e_aq` tests are reused when they already cover the agreed invariants.
- The Geant4 source repository remains read-only and no Geant4 run is performed.
