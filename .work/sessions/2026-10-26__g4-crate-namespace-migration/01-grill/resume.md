# Session: g4-crate-namespace-migration

**Date:** 2026-10-26T13:58:13.017Z
**Status:** Grill phase complete

## Problem Statement

Rename the five existing `dna-*` Rust crates to `g4-*` so their package names express the intended separation of shared infrastructure, physics, chemistry, and simulation. The rename must leave scientific behavior and persisted formats unchanged.

## Context & Constraints

- **Current behavior:** S01 is merged. The workspace still contains `dna-core`, `dna-data`, `dna-physics`, `dna-chemistry`, and `dna-sim`; chemical species now belong to `dna-chemistry`. S01 recorded passing exact snapshots and the maintained M2, M3, and M6 gates.
- **Pain point:** The `dna-*` package names obscure the physics/chemistry split. A broad text replacement could change scientific schema IDs, data paths, reference evidence, or current runnable paths incorrectly.
- **Dependencies:** S01 is accepted. Preserve physical formulas, constants, RNG order, output fields and CSV, species names, `data/dna`, `G4RS_DATA_DIR`, reference fixtures/provenance, historical GPS files, and third-party dependency versions/checksums. Do not run a Geant4 generator or overwrite reference data.
- **Tech stack:** Existing Rust 2024 workspace with the pinned Rust 1.97 toolchain, Cargo manifests and lockfile, current Python standard-library snapshot checker, GitHub CI, existing scripts/examples/docs. No new third-party dependencies.

## Success Metrics

- Cargo resolves exactly `g4-core`, `g4-data`, `g4-physics`, `g4-chemistry`, and `g4-sim` as workspace members, with no active `dna-*` package or `dna_*` Rust import.
- Standalone `g4-physics` and `g4-chemistry` all-target checks, formatting, workspace Clippy/tests, and the maintained M2/M3/M6 reference selectors pass. The selected tests must actually execute.
- Pre/post physics and physicochemical snapshot scientific payloads compare exactly, including floating-point bits, CSV, species order, and input data hashes. Only validated local package identity, lockfile hash, and revision metadata differences may be allowed; third-party versions/checksums stay unchanged.
- Current scripts, CI commands, examples, and guides work under the new paths. Existing `geant4-rs/dna-sim/run/v2` and `/v3` schema IDs, CSV headers, `data/dna` paths, and historical evidence remain intact.

## Architecture & Approach

Rename all five crate directories and package identities in one coherent, compilable migration: `dna-core` to `g4-core`, `dna-data` to `g4-data`, `dna-physics` to `g4-physics`, `dna-chemistry` to `g4-chemistry`, and `dna-sim` to `g4-sim`. Change workspace members, path dependencies, dev profile package keys, Cargo.lock local identities, and Rust import roots accordingly. Preserve public symbols such as `DnaError`. Update active script fixture paths, CI and current documentation, and publish an import migration table including `dna_core::species::*` to `g4_chemistry::*`. Extend the snapshot comparison only for individually validated rename-related metadata differences, then capture new snapshots and run the full regression gates.

## Assumptions & Trade-offs

This pre-release unpublished workspace accepts one deliberate source/import-path break and creates no compatibility shims. All five crates remain `publish = false`. S03 owns the optional chemistry feature and physics-only simulation build; this session does not add either. No biology crate, numerical changes, new physics, performance work, reference regeneration, or historical rewrite is in scope. The current S01 snapshots are retained under `.scratch/tests/2026-10-26__crate-domain-ownership/evidence/` even though the original `target/crate-split-baseline/` directory is absent.

## Open Questions

None. The exact scope and compatibility policy were approved in this conversation.

## Notes

The implementation source is Task 3 / S02 in `docs/superpowers/plans/2026-10-26-physics-chemistry-crate-separation.md` and its companion design. The current checker treats package identity and Cargo.lock hash as exact inputs; revise it with narrowly defined validation before using it as S02 evidence. Keep the migration on the existing GPS session workflow and do not start S03 automatically.
