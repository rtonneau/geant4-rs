# Session: physicochemical handoff

**Date:** 2026-11-13T21:58:33.069Z
**Status:** Grill phase complete

## Problem Statement

With a finite `Volume`, solvated electrons and dissociation products are placed without any boundary check, so they can land outside the water. Geant4 (GEO-0 audit, docs/04 D.7) clamps these physicochemical displacements to 0.8 x safety. The IRT stage behaviour for out-of-volume species is not visible in the Geant4 source (G-B1/G-B2, docs/09) and must be measured by RR-009.

## Context & Constraints

- **Current behavior:** Transport is boundary-limited (docs/04 D.6), but `g4-chemistry/src/displacer.rs` and the thermalisation placement ignore the volume. `Volume` has `inside` and `distance_to_out(p, v)` but no `safety`. The chemistry stage ignores the boundary.
- **Pain point:** Species can lie outside the volume, unlike Geant4, and nothing counts them. RR-009 has no generator.
- **Dependencies:** Unbounded runs must stay bit-identical, with the same random stream (D4). Existing g4rs_ref modes and reference files are unchanged. The spec lives on `test/rr008-scheduler-followup` (PR #34, includes GEO-0 PR #30).
- **Tech stack:** Rust workspace (g4-core geometry, g4-chemistry, dna-sim run layer), Geant4 11.4.1 C++ generator `validation/geant4/g4rs_ref`.

## Success Metrics

- `Volume::safety(p)` for box, tube and sphere matches Geant4 `DistanceToOut(p)` (G4Box/G4Tubs/G4Sphere 11.4.1) in analytic tests.
- With a volume, every clamped placement lies inside the volume and on the displacement ray at `min(0.8 x safety, |d|)`; D2: no species outside except counted dissociation products.
- D4: unbounded output and random stream are bit-identical to before.
- `summary.boundary` (schema run/v6, only with a volume) reports per-species counts: placements shortened by the clamp, species outside after placement, species outside at 1 ps and at the end of chemistry (S3, G-B1 diagnostic).
- A new g4rs_ref `finite_volume` mode for RR-009 (12 settings of docs/07) compiles; the run is logged in RUN_REQUESTS.md and not executed.

## Architecture & Approach

1. g4-core: `Volume::safety(p)`, ported from the Geant4 11.4.1 solids, with source citations.
2. g4-chemistry: the 0.8 x safety clamp, applied only with a volume, on e_aq thermalisation (G4DNAOneStepThermalizationModel.hpp) and on each dissociation product along `d = d_mother + d_product` (G4DNAMolecularDissociation::CheckNextStep). A product still outside is kept and counted, as in Geant4.
3. IRT unchanged; out-of-volume counts at 1 ps and at the end of chemistry as a diagnostic.
4. dna-sim: `summary.boundary` block, schema run/v6.
5. validation: g4rs_ref `finite_volume` mode (shape, size, energy, seed, histories, output path on the command line), README/BUILD_NOTES/RUN_REQUESTS updates.
6. Docs: docs/04 D.6/D.7 status, docs/08 M12 progress.

## Assumptions & Trade-offs

- The safety of a single water world equals the solid's DistanceToOut(p) (no daughters).
- The clamp's isotropic-safety bias (R-B1) is reproduced, not fixed.
- Not doing: any IRT-stage removal or confinement rule (waits for RR-009), running RR-009, multi-material geometry.

## Open Questions

- G-B1, G-B2 remain open until RR-009 is run by the user.

## Notes

- The user merged nothing yet: PR #34 carries the GEO-0 spec; this session builds on its branch.
- Choices: summary.boundary block over extending summary.escape; new g4rs_ref mode over a separate program.
