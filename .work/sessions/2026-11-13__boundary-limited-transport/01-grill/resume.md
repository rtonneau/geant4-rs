# Session: boundary-limited transport

**Date:** 2026-11-13T20:23:10.076Z
**Status:** Grill phase complete

## Problem Statement

Electron transport in `g4-physics` runs in infinite water and ignores `g4_core::geometry::Volume`. Steps must be limited by the boundary of an optional finite volume: a track that crosses it is killed and its kinetic energy is booked as `E_escaped`. Separately, the GEO-0 specification (PR #30: the docs/04 D.5 boundary source audit, the docs/07 finite-volume acceptance criteria, the docs/09 G-B1/G-B2 open questions and ADR 0005) was deleted from `main` by the geometry-primitives merge. It must be restored.

## Context & Constraints

- **Current behavior:** `transport_electron` draws `s = -ln(u)/Sigma_tot`, moves the track by s and samples a process. `Ledger.escaped` is always 0. `Termination` is {Completed, InteractionLimit, TrackingCut}. `RunConfig.validate()` rejects `geometry = Some(_)`. The GEO-0 text exists only at commit 4cf47d6.
- **Pain point:** Finite volumes (UHDR boxes, the 1 m sphere of `g4rs_ref`) cannot be simulated, and the GEO-0 acceptance criteria D1-D4 are not on main.
- **Dependencies:** Unbounded (None) output must stay bitwise identical (D4). Geant4 reference: `G4Transportation.cc:700-716` kills a track at the world exit (stop and kill, energy not deposited). `Volume::distance_to_out` and `Volume::inside` from `g4-core`. The 12 `transport_electron` call sites.
- **Tech stack:** Rust; g4-core geometry, g4-physics transport, g4-sim config/history/scoring; rayon; proptest/approx.

## Success Metrics

- Restored GEO-0 docs (docs/04 D.5 audit, docs/07 finite-volume criteria, docs/09 G-B1/G-B2, docs/08 and reproducibility edits, the 11.4.3 migration note if it was lost, ADR 0005), reconciled with the shipped `Volume` (Box/Tubs/Sphere with a centre offset; `Option<Volume>` None = unbounded) and keeping the existing D.5 primitives text.
- `transport_electron(table, config, primary, volume: Option<&Volume>, rng, observer)`: each step is `min(s, d_out)`. If `d_out < s`, the track moves to the surface, its energy goes to `ledger.escaped`, and it ends with no further draw for that track (no process draw).
- A primary that is Outside → typed `PhysicsError::PrimaryOutsideVolume`. A primary on the Surface moving outward (d_out = 0) escapes before any draw.
- `Termination::Escaped` when the primary escapes (primary-only, like TrackingCut); escaping secondaries add to `ledger.escaped` and an escape count in `HistoryOutcome`.
- g4-sim: `validate()` accepts geometry; `history.rs` passes it to transport; scoring has an Escaped termination count.
- Tests: unbounded bitwise identical to the current output; a very large volume equals unbounded; a tiny volume means the primary escapes; the ledger closes to 1e-9 with escapes; results identical on 1, 2 and 8 threads; the draw order is unchanged up to the first boundary-limited step.

## Architecture & Approach

Ticket 01 restores the deleted spec from 4cf47d6 and updates it. Ticket 02 changes g4-physics: an explicit `Option<&Volume>` parameter (TransportConfig stays Copy). Per step, `d_out` is computed before the free-path draw. If it is 0, the track escapes without drawing. Otherwise `s` is drawn as now; if `d_out < s`, the track escapes to the surface, else it interacts exactly as before. With None, the code path and the draws are identical. Ticket 03 wires g4-sim (config validation, history, scoring, the thread-determinism test) and updates the docs.

## Assumptions & Trade-offs

- Water only, so the only boundary is the world exit, where Geant4 kills the track; the remaining number of interaction lengths never needs carrying over (docs/04 D.5).
- A tie (`d_out == s`) is physics-limited: escape only when `d_out < s`.
- Not in this slice: the solvation and dissociation 0.8·safety clamp (needs `safety()`, GEO-0 D2), so solvated e_aq may sit outside the volume (documented limitation); the chemistry-stage boundary (G-B1/G-B2); the RR-009 Geant4 comparison; DistanceToIn and outside starts.
- Rejecting an Outside primary deviates from Geant4 (`G4PrimaryTransformer` warns and ignores it). It is chosen so a misconfiguration is never silent.

## Open Questions

- None blocking. G-B1/G-B2 stay open in docs/09.

## Notes

GEO-0 = gps session 2026-11-13__geometry-boundary-source-audit-and-spec (PR #30, merge 4cf47d6). The diff 4cf47d6..HEAD shows 285 deleted lines in docs and the loss of `.work/adr/0005-finite-volume-enum.md`.
