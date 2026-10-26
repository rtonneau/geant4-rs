# Session: crate-domain-ownership

**Date:** 2026-10-26T11:51:40.464Z
**Status:** Grill phase complete

## Problem Statement

Chemical species are owned by `dna-core` even though physics needs only shared physicochemical event records and penetration math. Move species ownership to `dna-chemistry` without changing scientific behavior, using exact pre-change evidence to detect drift.

## Context & Constraints

- **Current behavior:** `dna-core::species` defines the 15 mobile chemistry species and species instances; chemistry and simulation consume them. Core also owns shared event and penetration contracts. The five-crate workspace still uses `dna-*` package names.
- **Pain point:** Core exposes chemistry-only domain types, obscuring stage ownership. A type move can accidentally change species order, serialization, event handoff or RNG behavior.
- **Dependencies:** Use the approved S01 scope in `docs/superpowers/plans/2026-10-26-physics-chemistry-crate-separation.md` and its companion design. Start implementation from the merged R01 baseline on `main`, then capture immutable pre-refactor snapshots. Preserve reference data and provenance. Do not run a Geant4 generator.
- **Tech stack:** Existing Rust workspace and pinned toolchain, Cargo tests, current Geant4 reference fixtures, and a standard-library Python snapshot checker. No new third-party dependency.

## Success Metrics

- A reproducible pre-refactor capture and checker compare exact scientific floating-point bits, ordered physical and physicochemical output, input identity and runtime formats; synthetic checker tests detect a one-bit scientific change and incompatible inputs.
- `dna_chemistry::{Species, SpeciesInstance, SPECIES_CSV_HEADER}` exposes the unchanged types and values; all 15 canonical names, order, parsers and CSV behavior remain identical, and `dna-core` no longer owns or exports chemical species.
- Core, physics and chemistry retain the intended dependency direction; existing workspace tests and maintained M2/M3/M6 reference gates run, and post-move snapshots match the baseline exactly on the same platform/toolchain. Any unexpected baseline or comparison failure stops the move for diagnosis.

## Architecture & Approach

First add the deterministic capture example, checker, checker tests and a current separation guide; capture physical and coupled physicochemical output before changing ownership. Then move the existing `dna-core/src/species.rs` implementation and tests to private `dna-chemistry/src/species_types.rs`, re-export its public types and CSV header at the chemistry crate root, update the finite consumer set, and add interface tests. Keep `physchem` event records and Meesungnoen penetration in core as shared water-specific contracts. Compare fresh captures and run the named validation gates before accepting the move.

## Assumptions & Trade-offs

S01 changes Rust import paths for chemical species, with no compatibility re-export from core because that would reverse the dependency. It does not rename packages, add Cargo features or biology, change algorithms or data, regenerate reference files, or perform scientific fixes. The subsequent S02/S03 sessions own the namespace migration and physics-only simulation build. Exact snapshot equality is a regression check, not independent physical validation.

## Open Questions

None for the approved S01 design. A failing starting gate or incompatible snapshot is an investigation blocker, not a reason to relax acceptance criteria.

## Notes

The user approved the existing S01 scope on 2026-10-26. `main` contains merged R01 work; the current detached checkout predates that merge, so planning and implementation must use the actual approved starting revision rather than the proposal document's inspection hash.
