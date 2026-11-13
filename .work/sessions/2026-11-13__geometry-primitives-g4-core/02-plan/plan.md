# Implementation Plan

**Session:** geometry primitives g4-core
**Date:** 2026-11-13T18:07:17.739Z
**Estimated effort:** 2 days

## Strategy

Build `g4_core::geometry` shape by shape. Ticket 1 lays down the shared vocabulary (`Volume`, `Inside`, `GeometryError`, tolerances, translation, validated serde) with the box as its first solid. G4Tubs and G4Sphere follow as separate reviewable changes, because each carries its own Geant4 port and edge-case suite. RunConfig wiring comes last, once the public type is stable.

## Tickets Overview

- **Ticket 1:** Geometry module skeleton plus Box: Volume/Inside/GeometryError, tolerances, translation, checked constructors, validated serde, inside and distance_to_out with tests.
- **Ticket 2:** Full G4Tubs (rmin, rmax, half-z, phi segment): validation, inside, distance_to_out with analytic, proptest and edge-case tests.
- **Ticket 3:** Full G4Sphere (rmin, rmax, phi and theta segments): validation, inside, distance_to_out with analytic, proptest and edge-case tests.
- **Ticket 4:** `RunConfig.geometry: Option<Volume>` with validate() rejecting Some, TOML round-trip, and docs.

## Sequencing Rationale

Tubs and Sphere reuse the module, error type, tolerances and shared proptest helpers from ticket 1. The sphere's phi handling reuses the tubs phi-segment logic. RunConfig depends on the final `Volume` enum.

## Risks & Mitigation

- **Risk:** Mis-porting Geant4's tolerant surface logic (phi or theta cones, rmin) → **Mitigation:** Read the 11.4.1 `.cc` before each shape, cite lines, and run proptest invariants plus explicit surface/parallel/segment-face cases.
- **Risk:** The exit-point-on-surface proptest fails from floating-point error at large offsets → **Mitigation:** Bound the generated sizes and offsets to documented ranges, and state the tolerance used in the test.
- **Risk:** Theta-cone behaviour near pi/2 and the poles differs from Geant4 → **Mitigation:** Follow `G4Sphere.cc` branches, and document any deviation in docs/02 or docs/04.

## Assumptions

- Geant4 11.4.1 source is available locally under `${DEV_DIR}/GEANT4/geant4-v11.4.1-source`.
- `proptest` and `approx` are approved dev-dependencies (AGENTS.md).
- No Geant4 generator run is needed in this session.
