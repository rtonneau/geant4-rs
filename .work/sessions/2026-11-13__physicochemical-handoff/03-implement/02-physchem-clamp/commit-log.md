# Ticket 02: physchem-clamp

**Status:** ✅ Done

## Local Test Result

`cargo test -p g4-chemistry --test boundary_clamp`: 2 passed. `cargo test -p g4-physics --test boundary`: 10 passed (incl. new solvation clamp test, data present). `cargo test --workspace`: 0 failures. clippy -D warnings and fmt clean.

## Review Notes

Base ab82b54 (after ticket 01), clean tree. Inline; reviewed inline against Geant4 11.4.1 G4DNAOneStepThermalizationModel.hpp:224-239 and G4DNAMolecularDissociation.cc:203-257.
- New `g4_core::placement::clamp_displacement` with both rules (thermalisation: clamp iff safety <= |d|; dissociation: always min(0.8 s, |d|) along d/(|d|+1e-30)); CLHEP-style component division.
- Transport: solvated e_aq clamped with a volume; `HistoryOutcome.solvations_shortened`, `solvated_not_inside`.
- Engine: `with_volume`, `PhysChemStats.placement` (per-species shortened / not_inside); run layer binds RunConfig.volume to the engine.
- D4: None path unchanged (existing bit-identity tests and huge-volume equality pass; new no-volume == default test). No RNG draw (stream position checked).
- D2: no species Outside in tests; products not strictly Inside are kept and counted.
- A first chemistry test that inferred origins heuristically failed; replaced with single-event conversions where the origin is exact.

## Blockers / Challenges

The IRT history engine is not wired into the g4-sim run layer, so 'outside at end of chemistry' cannot be scored from a run (ticket 03).

## Commits

- 1835553 feat(physchem): Geant4 0.8 x safety clamp for solvated electrons and dissociation products (ticket 02)

## Time Spent

11m (ticket-start.js to ticket-complete.js)
