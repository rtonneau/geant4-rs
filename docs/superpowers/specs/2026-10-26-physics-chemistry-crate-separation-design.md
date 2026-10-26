# Physics/chemistry crate separation design

Date: 2026-10-26. Status: proposed for approval; no implementation or GPS-state changes. Baseline source: `1bdcb2d391b5ac88b8802ac360ef1ac2bc9934b6`; rebase the implementation against the actual approved starting revision. Companion: [implementation plan and all GPS seeds](../plans/2026-10-26-physics-chemistry-crate-separation.md).

## Intent

Make physics usable without the chemistry implementation, make chemistry own its chemical domain, and leave a clean place for later biological models. This is an organizational refactor of the existing native Rust implementation, not a move to FFI or a claim of implementing all Geant4 physics. Current physics models are themselves Geant4-DNA models.

The user's request is a plan for the separation discussed in the preceding architecture review. Exact names, optional orchestration feature, and compatibility policy below are proposed engineering choices, not claims of prior user approval. Approving this design adopts them; writing this document does not authorize implementation.

## Approaches considered

1. **Merge everything into exactly two crates.** Superficially simple packaging, but either duplicates units/RNG/data contracts or makes one scientific stage depend on the other. It also mixes transport orchestration and model data with domain implementations. Not recommended.
2. **Keep the five-crate decomposition; clarify ownership, names and standalone builds.** Recommended. Most isolation already exists, so preserve it and remove actual leakage. Two primary scientific crates remain independently usable; core/data/simulation stay supporting modules.
3. **Add two facade packages over the existing workspace.** Cheap packaging, but alone does not move chemical species out of core or remove the simulation crate's unconditional chemistry dependency. Defer facades until a real downstream packaging need appears.

## Target modules and dependencies

Arrows mean “depends on,” not data flow:

```text
g4-core
g4-data       -> g4-core
g4-physics    -> g4-core, g4-data
g4-chemistry  -> g4-core
g4-sim        -> g4-core, g4-data, g4-physics
              -> g4-chemistry only with feature `chemistry`

future g4-bio -> selected physical records and/or chemical results
                (not created or specified by this refactor)
```

Rename existing package and directory prefixes `dna-` to `g4-`; Rust import prefixes change `dna_` to `g4_`. Package descriptions must continue to identify selected Geant4-DNA liquid-water models and their real energy/domain limits. No publishing or package-name reservation is included; all crates remain `publish = false`.

## Ownership and interfaces

- **`g4-core`:** units, vectors, RNG streams, errors, manifests/provenance, shared numerical helpers, the existing physical-to-physicochemical event records, and the shared Meesungnoen penetration implementation. Core is a small shared scientific foundation, not a promise that every module is material-independent.
- **`g4-data`:** existing physics-table catalogue/loading/interpolation. Chemical TOML loading and reaction data remain chemistry-owned.
- **`g4-physics`:** existing transport/models/secondary accounting and thermalisation handoff. No dependency on the chemistry crate, no chemical species enum, no reaction network. Preserve `physchem` event re-exports; their presence is a handoff contract, not execution of chemical reactions.
- **`g4-chemistry`:** move the complete current `dna-core/src/species.rs` implementation into private `src/species_types.rs`, publicly re-export `Species`, `SpeciesInstance` and `SPECIES_CSV_HEADER` at the crate root. Keep current `species.rs` as the species-property table. All physicochemistry, reaction/diffusion/IRT and chemistry errors/data stay here.
- **`g4-sim`:** configuration, physics-history execution/scoring and optional coupled physchem/diagnostics. Default feature set includes `chemistry`, preserving existing default capabilities. `--no-default-features` supports physics histories, physical scoring and raw `PhysChemEvent` recording/CSV without compiling the chemistry crate.

The shared event and penetration modules deliberately remain in core. They are used by both stages today (`dna-physics/src/transport.rs`, `models/solvation.rs`, `dna-chemistry/src/displacer.rs`). Moving them to chemistry would reverse the desired dependency; copying them risks drift; creating a sixth micro-crate is unnecessary for this bounded refactor. Document them as water-specific shared contracts. A future genuinely generic physics expansion can revisit that ownership on evidence.

### Critical handoff invariants

`PhysChemEvent`, `PhysChemKind` and `PhysChemSite` retain their fields, names, units, ordering and serialization. In particular, solvation's event position is already displaced, while the physical energy deposit belongs to the thermalisation origin. Do not apply displacement twice or move its RNG draws to another stage. Preserve shell/level numbering, creation order, history/track IDs, per-stage RNG streams and global 1 ps chemistry initialization.

`Species` keeps all 15 variants, exact Geant4 configuration names, ordering, legacy parsing and serialization. `SpeciesInstance` keeps its fields and behavior. This is a type move, not a new wrapper/conversion type.

## Compatibility policy

This pre-release, unpublished workspace takes one documented **source/import-path break**: old `dna_*` paths and `dna_core::species` no longer exist after migration. Do not add five compatibility shim crates or a core-to-chemistry re-export cycle. Keep public symbol names such as `DnaError` unchanged unless moving ownership requires otherwise; cosmetic type renaming is out of scope.

Runtime compatibility is strict: do not rename JSON schema IDs (including existing `geant4-rs/dna-sim/...` IDs), CSV headers, species strings, environment variables such as `G4RS_DATA_DIR`, data paths or manifest keys. Do not rewrite reference provenance because it mentions an old package/path/command; it describes the historical run. Output revision/build metadata may legitimately change; scientific fields must not.

Under `g4-sim`'s default features, keep the existing `run` module's public function/type paths (with only the crate prefix changed). Chemistry-specific paths are absent under `--no-default-features`; no dummy success implementation or silent skipping. Gate chemistry error variants and diagnostics consistently. Raw physical handoff records remain available in both builds.

## Scope and non-goals

Scope: type ownership, package/directory/import migration, a thin optional-chemistry orchestration seam, build/dependency checks, unchanged-behavior evidence and current documentation.

Non-goals: biology code or placeholders, new physics/models, proton generalization, scheduler/G(t)/CLI implementation, verified-input redesign, statistical gate redesign, performance/batching changes, generic plugin traits, FFI, publishing, unsafe code, extra dependencies, changing Geant4 reference files or modes.

Future biology may need both direct physical interaction records and chemical histories. Do not design it as chemistry-only by assumption, and do not create an empty `g4-bio` crate. Record its future position only.

## Constraints and proof

- Preserve all physical formulas, constants, data bytes, model windows, RNG draw order, scientific output and existing statistical tolerances.
- No Geant4 execution. Consume existing stored reference data only; never blanket-run all ignored tests.
- Use the pinned Rust toolchain and approved dependencies; do not change compiler policy in this refactor. The separate R01 audit seed owns the support-policy correction.
- No active GPS state, seed, roadmap, instruction-file or reference-data edits in this planning task.
- Prove standalone core/physics/chemistry builds; prove the resolved physics-only simulation graph excludes chemistry, including via dev-dependency leakage.
- Compare same-platform pre/post scientific outputs with exact floating-point bits under identical toolchain/input/config/seed; explicitly allowlist only build/revision metadata differences. Existing tests/reference gates must also pass.
- Preserve default-feature coverage; zero tests from feature gating is not evidence that physics tests ran. Add feature-specific consumers and assert the expected tests are selected.

## Roadmap placement

Recommended proposed sequence: R01 `validation-evidence-and-ci` -> S01 `crate-domain-ownership` -> S02 `g4-crate-namespace-migration` -> S03 `physics-only-simulation-build` -> R02 `verified-inputs-and-provenance` -> R03 onward from the strategic roadmap.

R01 is a recommended prerequisite because reliable regression evidence must precede this refactor. It is not silently registered or executed here. The migration seeds must update its current commands/paths as packages move. If R02 has already landed at execution time, preserve that design and apply the same separation afterward; do not undo its verified-input contracts.

Three seeds are sufficient. No separate “design,” “rename each crate,” “create bio placeholder” or “close gate” seed is needed. The acceptance result and import migration guide belong to S03.
