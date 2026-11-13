# Implementation Plan

**Session:** physicochemical handoff
**Date:** 2026-11-13T22:11:49.591Z
**Estimated effort:** 1-2 days

## Strategy

Build bottom-up: geometry safety first, then the clamp in chemistry placement, then run-layer scoring, then the RR-009 generator and docs. Each ticket keeps unbounded runs bit-identical.

## Tickets Overview

- **Ticket 1:** `Volume::safety(p)` for box, tube and sphere, ported from Geant4 11.4.1.
- **Ticket 2:** 0.8 x safety clamp on e_aq thermalisation and dissociation products, with per-species placement counters.
- **Ticket 3:** `summary.boundary` block (schema run/v6) with shortened/outside counts after placement, at 1 ps and at the end of chemistry.
- **Ticket 4:** g4rs_ref `finite_volume` mode for RR-009 (compile-checked, logged) and docs updates.

## Sequencing Rationale

The clamp needs safety; scoring needs the clamp's counters; the generator mirrors the Rust observables, so it comes last with the docs.

## Risks & Mitigation

- **Risk:** the clamp changes the random stream or output of unbounded runs → **Mitigation:** apply it only with a volume and keep the D4 regression test.
- **Risk:** Geant4's sphere/tubs safety has edge cases like DistanceToOut(p,v) → **Mitigation:** port with line citations and test the invariant safety <= distance_to_out along random rays.
- **Risk:** the C++ mode cannot be compile-checked locally → **Mitigation:** say so explicitly and log it in RUN_REQUESTS.

## Assumptions

- Single water world: navigator safety equals the solid's DistanceToOut(p).
- IRT stays unchanged until RR-009 answers G-B1/G-B2.
