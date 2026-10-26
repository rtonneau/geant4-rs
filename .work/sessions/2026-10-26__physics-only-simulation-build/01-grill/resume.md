# Session: physics-only-simulation-build

**Date:** 2026-10-26T15:57:30.770Z
**Status:** Grill phase complete

## Problem Statement

`g4-sim` unconditionally depends on `g4-chemistry` through orchestration, scoring, diagnostics and errors. A caller cannot build parallel physical transport and raw physicochemical handoff export without compiling chemistry. Complete S03, Tasks 4–5 of the approved physics/chemistry crate separation plan.

## Context & Constraints

- **Current behavior:** S01 species ownership and S02 namespace migration are complete. `g4-sim` has no feature boundary; default runs include coupled physicochemistry and chemistry diagnostics.
- **Pain point:** The current simulation dependency graph prevents a chemistry-free physics consumer, and default-only CI cannot prove feature isolation.
- **Dependencies:** Preserve all default public `run` paths, physical event and CSV handoff contracts, scientific data, serialized schemas, model windows, RNG draws, batching and exact same-platform scientific snapshots. Keep the established M2/M3/M6 reference gates and never run a Geant4 generator.
- **Tech stack:** Existing five-crate Rust workspace, Cargo features and package-scoped checks, existing Python standard-library snapshot checker, and GitHub Actions CI. No new third-party dependency.

## Success Metrics

- `g4-sim` has default-on optional `chemistry`; its resolved no-default dependency graph, including relevant target checks, excludes `g4-chemistry`. Standalone physics and chemistry remain independent.
- A real no-default consumer compiles and runs eight ordered 1 keV physics histories with raw nonempty handoff events and unchanged handoff CSV; default coupled tests and diagnostics still run.
- Same-platform default and no-default physics snapshots match exactly in scientific payload, and default coupled output matches the pre-S01 baseline. Required format, Clippy, workspace, profile, graph-checker and selected reference gates pass with recorded evidence.

## Architecture & Approach

Execute Tasks 4–5 as written in `docs/superpowers/plans/2026-10-26-physics-chemistry-crate-separation.md`. Add `chemistry = ["dep:g4-chemistry"]` as a default feature. Keep physical orchestration, raw `PhysChemEvent` recording and CSV sink in `run.rs`; move coupled functions and `PhysChemRunOutput` into a private cfg-gated child module with existing public re-exports under the default feature. Gate chemistry-only scoring, diagnostics, errors and tests. Keep the physics snapshot available in both profiles and make unsupported no-default `--physchem` fail clearly. Extend the checker to validate reachable resolved dependency graphs and capture the physics-only profile; add package-scoped CI and document the migration and verification evidence.

## Assumptions & Trade-offs

Default feature behavior and public paths remain compatible except for the already completed S02 crate-prefix change. Chemistry-specific APIs are absent without the feature; no stub implementations are introduced. This session does not add physics models, chemistry behavior, a CLI, biology, new dependencies or performance changes. Scientific equality is checked under the same toolchain, platform, input and seed, with only explicitly permitted build/revision metadata differences.

## Open Questions

None. The existing approved Tasks 4–5 settle the feature boundary, interfaces, test profiles and acceptance criteria.

## Notes

The scout seed S03 has S02 as prerequisite, now satisfied on `main` by PR #23. The immutable pre-S01 captures and their input hashes are recorded in `docs/crate-separation.md`. Use the plan's explicit verification matrix and selected ignored reference tests; never treat ignored tests as passed merely because the workspace suite succeeds.
