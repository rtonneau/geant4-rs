# Session: m12-isotropic-source-box-endpoints

**Date:** 2026-11-14T08:43:32.961Z
**Status:** Grill phase complete

## Problem Statement

The RR-009 finite-volume gate is open: its six box endpoints (1000x1000x100 nm, 1/10/100 keV) are reported NOT RUN because the Rust run layer only has a fixed primary direction, while Geant4 `g4rs_ref finite_volume` draws isotropic primaries (`G4RandomDirection()`, one per event, `validation/geant4/g4rs_ref/g4rs_ref.cc:884`).

## Context & Constraints

- **Current behavior:** `RunConfig.direction: [f64; 3]` is a required fixed direction; `crates/g4-sim/tests/rr009_finite_volume.rs` runs the 18 sphere endpoints with +z (rotation invariant) and skips the box.
- **Pain point:** the box is not rotation invariant, so the gate cannot pass without an isotropic source.
- **Dependencies:** existing TOML/JSON run configs and run metadata must parse and serialise unchanged; fixed-direction runs must stay bitwise identical; stream scheme `(history_id << 2) | stage` of docs/reproducibility.md.
- **Tech stack:** Rust workspace (g4-core, g4-physics, g4-sim), serde/toml, rand_chacha.

## Success Metrics

- Pre-registered KS tests on cos(theta) and phi of the isotropic source pass; 1/2/8-thread bitwise equality holds for an isotropic run.
- Fixed-direction runs are bitwise unchanged (existing determinism/regression tests pass unmodified) and old configs round-trip unchanged.
- All 24 RR-009 endpoints evaluated with unchanged criteria (z = 4.098038123646298, margin +-0.01); verdict recorded in docs/07, docs/08 and validation/gates.json.

## Architecture & Approach

1. `RunConfig.direction` becomes an untagged serde enum: `[x, y, z]` (fixed, same serialisation as today) or `"isotropic"`.
2. Isotropic direction: cos(theta) = 2u - 1, phi = 2 pi u (two uniforms, fixed count), the distribution of Geant4 `G4RandomDirection` (verify against the Geant4 .cc and document). Drawn from a new per-history stream `Stage::Source = 3` (ADR 0006), so the Physics stream and its draw order are untouched.
3. Provenance / run schema updated if output changes; docs/reproducibility.md gets the Source stream draw order.
4. Tests as in the success metrics, criteria written before running.
5. RR-009 consumer: only the six box rows use isotropic; sphere rows keep +z and their recorded verdicts (including the two accepted INCONCLUSIVE ones).
6. Run the ignored consumer, record the gate verdict.

## Assumptions & Trade-offs

- Reproduce the distribution, not the Geant4 generator sequence.
- Stage slot 3 is consumed; a future stage needs a stream-layout change (ADR 0006).
- Not re-running the sphere endpoints isotropic.
- No Geant4 C++ change and no new reference data.

## Open Questions

None.

## Notes

The box rows may come out INCONCLUSIVE or FAIL; the verdict is recorded as is, no criterion changes.
