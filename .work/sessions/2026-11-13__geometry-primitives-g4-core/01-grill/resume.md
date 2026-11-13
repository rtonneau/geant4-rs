# Session: geometry primitives g4-core

**Date:** 2026-11-13T17:49:40.021Z
**Status:** Grill phase complete

## Problem Statement

geant4-rs has no geometry: transport and chemistry run in infinite water. Finite volumes (UHDR boxes, the 1 m water sphere of the Geant4 reference generator) need primitive solids in `g4-core` with an inside test and distance-to-exit along a ray, validated config values, and a place in `RunConfig`.

## Context & Constraints

- **Current behavior:** Infinite medium only (docs/04, docs/08 N1 audit); `g4-core` has `Vec3` but no geometry module; `g4_sim::config::RunConfig` has no geometry field.
- **Pain point:** No primitive exists to build a finite boundary, the solvation navigator clip or finite-volume chemistry on.
- **Dependencies:** Existing outputs must stay byte-for-byte unchanged (None = infinite water). Geant4 11.4.1 `source/geometry/solids/CSG/src/G4Box.cc`, `G4Tubs.cc`, `G4Sphere.cc` are the semantic reference (Inside, DistanceToOut(p, v), tolerances).
- **Tech stack:** Rust, `g4-core` (`Vec3`, typed errors), `serde`/`toml`, `thiserror`, `proptest`, `approx`.

## Success Metrics

- `Volume` enum with Box (half-lengths), full G4Tubs (rmin, rmax, half-z, start/delta phi, z axis) and full G4Sphere (rmin, rmax, phi start/delta, theta start/delta), each with a centre offset, no rotation; units nm and rad.
- Checked constructors and validated serde deserialisation reject non-finite values, non-positive sizes, rmin >= rmax and out-of-range angles with a typed g4-core error.
- `inside(p) -> Inside {Inside, Surface, Outside}` with Geant4 tolerances (kCarTolerance = 1e-9 mm = 1e-3 nm, kAngTolerance, kRadTolerance) and cited source lines.
- `distance_to_out(p, v) -> Option<f64>`: None for an Outside start; 0 for a Surface start moving outward; follows 11.4.1 DistanceToOut(p, v) semantics.
- Tests: analytic cases per shape; proptest invariants (exit point classifies as Surface; Inside start gives d > 0); edge cases (start on surface inward/outward, rays parallel to faces, phi/theta segment faces, the rmin hole).
- `RunConfig.geometry: Option<Volume>`, None by default; `validate()` rejects `Some(_)` until transport honours boundaries; existing tests and outputs unchanged.

## Architecture & Approach

New module `g4_core::geometry` holding `Volume` (enum over shape structs with a common translation), `Inside`, and a `GeometryError`. Shape structs have private fields and checked constructors; serde deserialisation goes through the same validation (`try_from` a raw struct). The algorithms port the Geant4 11.4.1 CSG semantics into idiomatic Rust, one file per shape, with source line citations. `g4-sim` gains an optional `geometry` field in `RunConfig` that `validate()` refuses when set, so it cannot be silently ignored.

## Assumptions & Trade-offs

- The Geant4 tolerances are converted to nm once (1e-9 mm = 1e-3 nm).
- Not in this slice: DistanceToIn, safety distances, navigation, rotations, transport or chemistry use of the volume, and a Geant4 generator comparison run.
- Full G4Tubs/G4Sphere generality was chosen over simpler solids, which accepts the extra edge-case surface (phi/theta cones and planes) and its test cost.

## Open Questions

- Whether the Rust sphere theta-cone handling reproduces Geant4's exact tolerance behaviour near theta = pi/2 and the poles; verify against `G4Sphere.cc` during implementation and document any deviation.

## Notes

Design questions settled in the grill: shapes Box + full Tubs + full Sphere; Geant4 3-state tolerant surface; wire into RunConfig as Option (None = infinite water, Some rejected for now); outside start → None; centre offset, no rotation; follow G4 semantics with idiomatic code, no generator run request. Glossary term **Volume** added.
