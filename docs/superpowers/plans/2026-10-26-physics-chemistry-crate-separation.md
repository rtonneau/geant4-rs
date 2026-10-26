# Physics/Chemistry Crate Separation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task, within the approved GPS session/ticket workflow. Steps use checkbox syntax. Do not run a second independent branch/commit/session workflow alongside GPS.

**Goal:** Separate scientific ownership and allow physics-only use without compiling chemistry, while preserving every existing scientific behavior and leaving biology for a later requirement-driven implementation.

**Architecture:** Retain five crates, renamed `g4-core`, `g4-data`, `g4-physics`, `g4-chemistry` and `g4-sim`. Chemistry owns species; core retains small shared event/math contracts. Physics and chemistry do not depend on each other; simulation enables chemistry through a default-on optional feature.

**Tech Stack:** Existing Rust workspace, pinned toolchain, Cargo features, Rayon, serde/serde_json and existing dependencies; standard-library Python for a small dependency/snapshot checker. No new third-party dependencies, no runtime FFI.

**Spec:** [2026-10-26-physics-chemistry-crate-separation-design.md](../specs/2026-10-26-physics-chemistry-crate-separation-design.md). Read it with this plan.

**Status:** Proposal only, 2026-10-26. No seeds registered/executed, no code changes, no active GPS migration. Baseline inspected: `1bdcb2d391b5ac88b8802ac360ef1ac2bc9934b6`. Exact names, source-compatibility break and optional orchestration feature require approval of this plan.

## Global Constraints

- Preserve physical formulas, constants, data bytes, model windows, RNG draw order, scientific output and existing statistical tolerances.
- No Geant4 execution. Consume existing stored reference data only; never blanket-run all ignored tests.
- Use the pinned Rust toolchain and approved dependencies; do not change compiler policy in this refactor.
- Crates remain `publish = false`; do not create facade/shim/bio crates or reserve registry names.
- Source/import paths may break as specified; serialized schema IDs, CSV headers, exact species names, data locations and environment variables must not change.
- No core-to-chemistry or physics-to-chemistry dependency, including a compatibility re-export that creates a cycle.
- Keep shared `PhysChemEvent`/`PhysChemSite` and Meesungnoen penetration implementation in core; document their water-specific purpose rather than claiming a fully generic core.
- No scheduler, G(t), proton, biology, CLI, verified-input redesign or batching/performance change in these seeds.
- Preserve historical GPS files, review reports, reference fixtures/provenance and old run commands. Update current guides and executable paths only.
- Resolve `DEV_DIR`/`G4_ROOT` from the environment; stop if unset. Never write machine-specific roots into code/global instructions.

## Review Focus

1. Moving solvation code or recording events can change draw order or apply displacement twice: exact handoff/RNG snapshots in Task 1 and Tasks 2/4/5 regression checks.
2. Renaming package identities can accidentally rename scientific schemas or data paths: serialized-contract comparison in Tasks 1/3/5.
3. Default features/dev-dependencies can mask a supposed chemistry-free build: fresh package-scoped graph/build checks in Tasks 4/5.
4. Moved species can change ordering, Unicode names, parser behavior or public type identity: exact type/name/CSV tests in Task 2.
5. Conditional compilation can hide all tests or leave a dangling error/diagnostic type: explicit test-target inventory and default/no-default consumers in Tasks 4/5.

## Roadmap integration and execution order

Recommended sequence relative to [the strategic proposal](../../reviews/proposed-roadmap.md):

```text
R01 validation-evidence-and-ci
  -> S01 crate-domain-ownership
  -> S02 g4-crate-namespace-migration
  -> S03 physics-only-simulation-build
  -> R02 verified-inputs-and-provenance
  -> R03 option3-reaction-execution -> remaining scientific roadmap
```

R01 is a recommended prerequisite, not an extra separation seed or work silently executed here. It establishes trusted current gates; S02 updates their package names. If R02 is already implemented when this plan runs, preserve its interfaces/invariants and execute the separation afterward. Do not restart or undo verified-input work. The existing selected process-count session is not a prerequisite; resolve its retirement separately through the previously proposed, approved GPS migration.

There are **exactly three new GPS seeds** for this change. S01/S02/S03 below are stable review IDs, not replacements for R01–R12. No separate planning-only, per-crate rename, biology-placeholder or closure seed is required. Each session exits compilable and tested. Rough effort: three focused sessions, approximately 3–6 developer-days including migration/regression work; a planning estimate, not a guarantee. This is larger than a rename-only change because S03 adds a supported physics-only orchestration build.

## File and interface map

| Current path / symbol | Target | Action |
|---|---|---|
| `crates/dna-core/src/species.rs` | S01: `crates/dna-chemistry/src/species_types.rs`; S02: `crates/g4-chemistry/src/species_types.rs` | Move implementation/tests; private module, root re-export of `Species`, `SpeciesInstance`, `SPECIES_CSV_HEADER` |
| `dna-core/src/physchem.rs`, `penetration.rs` | Same modules under `g4-core` | Keep implementation and semantics; correct ownership docs |
| `dna-physics/src/physchem.rs` | `g4-physics/src/physchem.rs` | Preserve handoff re-exports; update crate prefix only |
| Five `crates/dna-*` directories/manifests | Corresponding `crates/g4-*` | One atomic package/directory/import migration |
| `Cargo.toml`, `Cargo.lock` | Same paths | Workspace/member/path/profile/lock package identities only; no unrelated version updates |
| `dna-sim/src/run.rs` | `g4-sim/src/run.rs` plus `g4-sim/src/run/chemistry.rs` | Keep physics/common orchestration in `run.rs`; private cfg-gated chemistry implementation with public re-exports from `run` |
| `dna-sim/src/{lib,error,physchem_scoring,spatial_diagnostic}.rs` | Corresponding `g4-sim` files | Feature-gate chemistry modules/imports/error variants; retain default exports |
| `crates/dna-sim/examples/crate_split_snapshot.rs` | Later `crates/g4-sim/examples/crate_split_snapshot.rs` | New test-support snapshot producer, both build profiles |
| `scripts/check_crate_split.py` | Same path | New standard-library checker: dependency graph and exact snapshot comparison |
| `scripts/test_check_crate_split.py` | New file | Standard-library unit tests for comparison and graph validation |
| Current tests/examples and scripts referencing moved fixtures | Corresponding new paths/imports | Update executable references without regenerating expected scientific data |
| `.github/workflows/ci.yml` | Same path | Add explicit physics-only package check; retain full default validation |
| `docs/crate-separation.md` | New current guide | Architecture, import migration, feature matrix, no-biology scope and verification record |

Target type imports: `g4_chemistry::{Species, SpeciesInstance, SPECIES_CSV_HEADER}`. `g4_chemistry::species::SpeciesTable` retains its property-table role. Do not create duplicate species enums or conversion wrappers.

## GPS seed specifications

### S01 — `crate-domain-ownership`

**Title:** Put chemical species under chemistry ownership and freeze scientific contracts.

**Strength:** Strong. **Priority / complexity:** P1 / M. **Prerequisites:** approved design/plan; recommended R01 completed on the starting revision. **Traceability:** audit F8 and core-domain leakage; separation request. **Files:** core species/lib, chemistry lib/species/import consumers, simulation consumers, snapshot example/checker and current architecture guide.

**Problem:** Chemical species are currently in `dna-core`, although physics needs event records and shared thermalisation math, not the reaction-network species enum. A move without exact baseline evidence could silently alter ordering, formats or random streams.

**Scope / solution:** Execute Tasks 1–2 using the existing package names. Freeze pre-change scientific snapshots, move species and their tests to chemistry, update imports, and document why the event/penetration contracts stay shared. Preserve type definitions and all runtime behavior.

**Exclusions:** Package renaming, Cargo feature changes, biology, generic transport events, scientific bug fixes or reference regeneration.

**Deliverables:** Root chemistry species exports, no core-owned chemical species, baseline/checker, ownership/import guidance.

**Acceptance:** Old scientific snapshots match exactly on the same platform/toolchain; all 15 canonical names/order/parsers and CSV remain identical; physics and chemistry have no dependency on each other; workspace and maintained M2/M3/M6 gates pass or a pre-existing baseline failure is explicitly resolved before proceeding. No unobserved M6 pass may be assumed from the prior audit.

### S02 — `g4-crate-namespace-migration`

**Title:** Rename the workspace coherently without changing its scientific formats.

**Strength:** Strong. **Priority / complexity:** P1 / M. **Prerequisites:** S01 accepted. **Traceability:** naming/ownership separation request; prepares strategic R02/R03 paths. **Files:** five crate directories, workspace/crate manifests, lockfile, imports, current scripts/CI/examples/guides.

**Problem:** `dna-*` names obscure the intended separation of physics, chemistry and future biology; a blanket text replacement would corrupt scientific schema names and historical evidence.

**Scope / solution:** Execute Task 3. Rename all five packages/directories/import roots in one compilable change; update live commands and scripts. Keep old schema identifiers/data directories/environment names; document one deliberate source-level breaking change.

**Exclusions:** Compatibility shim crates, publishing, symbol-wide cosmetic renames, numerical/model changes, rewriting historical GPS/reference provenance.

**Deliverables:** Five `g4-*` packages, functioning scripts/examples/current docs, dependency graph and exact pre/post evidence.

**Acceptance:** Cargo resolves exactly the five renamed workspace members; no old package remains in active Rust imports/manifests; no dependency versions change unexpectedly; standalone physics/chemistry checks and full tests pass; snapshot scientific payloads and current reference verdicts are unchanged. Old `dna-sim` schema strings and `data/dna` paths remain intentionally present.

### S03 — `physics-only-simulation-build`

**Title:** Deliver a chemistry-free simulation build and lock in the separation.

**Strength:** Strong. **Priority / complexity:** P1 / M–L. **Prerequisites:** S02 accepted. **Traceability:** optional-chemistry orchestration proposal in this design; complements, does not implement, strategic R06/R08.

**Problem:** Even though the physics crate is independent, simulation currently unconditionally links chemistry through `run.rs`, scoring, diagnostics and `SimError`. A package rename alone does not let callers use parallel physics orchestration without chemistry.

**Scope / solution:** Execute Tasks 4–5. Add default-on `chemistry` feature to `g4-sim`; isolate coupled run implementation while keeping default public paths; gate chemistry tests/errors/modules; verify raw event export remains available in a physics-only build. Add CI/dependency guards and complete the migration guide.

**Exclusions:** New CLI, no-op chemistry stubs, split biological crate, exposing unstable scheduler interfaces, changing batching/RNG or adding simulation modes.

**Deliverables:** Both supported feature profiles, package-scoped consumer tests, graph enforcement, final verification report and command/import guide.

**Acceptance:** Resolved no-default simulation graph contains no `g4-chemistry`; a separate no-default consumer compiles and runs physics plus handoff CSV; default coupled physics/physchem and spatial diagnostics retain their behavior; both build profiles give identical physical payloads; chemistry tests still run by default; full required regression commands pass. Future biology is documented but no placeholder implementation is created.

## Task 1 — Capture reproducible pre-refactor evidence (S01)

**Files:** Create `crates/dna-sim/examples/crate_split_snapshot.rs`, `scripts/check_crate_split.py`, `docs/crate-separation.md`. Reuse test setup/bit extraction patterns in `crates/dna-sim/tests/determinism.rs` and `physchem_a.rs`; do not change their existing expected values.

**Interfaces:** Snapshot example accepts exactly `--physics` or `--physchem`, prints one JSON object to stdout and exits nonzero on missing input/error. Checker accepts `--compare <baseline.json> <candidate.json>`; exits 0 only for exact compatible snapshots, otherwise 1 with the first differing field. Later Task 5 adds `--check-graph`.

- [ ] Implement a deterministic test-support capture with schema `geant4-rs/crate-split-snapshot/v1`, seed `20261026`, history IDs 3–10 (eight histories), primary energies 100 eV, 1 keV and 10 keV, default Geant4 elastic rotation, full secondaries and recorded handoff events. Use one thread and batch size 7 for the baseline.
- [ ] Record every physical history outcome/ledger/secondary/radial/event field, coupled species list/stats for `--physchem`, existing summary JSON/schema and raw CSV headers/rows. Represent all floating-point scientific values as their `to_bits()` hexadecimal strings in the comparison payload; preserve ordering and signed zero. Use the existing production entry points, not a second implementation of physics.
- [ ] Record toolchain/target, input hashes and capture revision separately. The comparison requires matching toolchain/target/data/config. Before capture, enumerate the actual metadata field paths allowed to change: capture/implementation revision, dirty state, package label, and lockfile hash if recorded by earlier work. Validate each changed value against the candidate checkout; a lockfile-hash difference is allowed only with unchanged third-party dependency versions/checksums and the planned local-package rename. Data/model/chemistry hashes are never allowlisted. Do not exclude an entire provenance or summary object to make a comparison pass.
- [ ] Test the checker with a small synthetic equal pair (exit 0), a one-bit scientific difference (exit 1), changed species order/header (exit 1), changed input hash/toolchain (incompatible, exit 1), and only the allowlisted revision change (exit 0). Use standard-library `unittest` in `scripts/test_check_crate_split.py`; add no dependencies.
- [ ] Run `python -m unittest discover -s scripts -p test_check_crate_split.py`. Run the example in each profile and save captures through the runner to a **new** ignored directory `target/crate-split-baseline/<revision>-<unique-run-id>/`. Never overwrite a baseline or reference fixture. Record capture commands, revision and hashes in the guide. Provide `--capture <new-directory> --package dna-sim` in the checker for this purpose; it runs the two example profiles, validates success and creates the directory exclusively.
- [ ] Run the full baseline gate commands below. Stop implementation on an unexpected failing baseline rather than mixing scientific repair into this seed. Record successful command outputs and the baseline location before changing ownership. Check that new capture instrumentation does not change existing results.
- [ ] Commit this test/documentation slice through GPS. This capture establishes regression identity, not independent scientific validation or permission to regenerate Geant4 fixtures.

## Task 2 — Move chemistry-only species (S01)

**Files:** Move `crates/dna-core/src/species.rs` to `crates/dna-chemistry/src/species_types.rs`; modify both `src/lib.rs` files, chemistry species/channels/engine/displacer/molecule/reaction imports and simulation physchem scoring/spatial/test imports. Create `crates/dna-chemistry/tests/species_interface.rs`. Search all source/tests/examples for `dna_core::species` to complete the finite consumer set.

**Interfaces:** Public `dna_chemistry::{Species, SpeciesInstance, SPECIES_CSV_HEADER}`; unchanged `Species` methods/variants and `SpeciesInstance` fields. `dna_chemistry::species::SpeciesTable` unchanged. `dna_core::physchem`/`penetration` unchanged. No `dna_core::species` re-export because core must not depend on chemistry.

- [ ] Add `species_interface` tests importing the proposed root exports; initially they fail to compile. Assert `Species::ALL.len() == 15`, exact names in current order (`H3Op`, `°OH`, `OHm`, `e_aq`, `H`, `H2`, `H2O2`, `HO2°`, `HO2m`, `Oxy`, `Om`, `O2`, `O2m`, `O3`, `O3m`), strict/legacy parser behavior and `SPECIES_CSV_HEADER == "species,x_nm,y_nm,z_nm,time_ps"`.
- [ ] Move the implementation and its unit tests without algorithm changes. Make `species_types` private, root-re-export the two types and CSV constant, and fix their unit/vector imports to `dna_core`. Update all consumers. Do not duplicate enum definitions or disable tests to avoid imports.
- [ ] Assert an `e_aq` instance at `(1.0,-2.5,0.25)` nm and 1 ps still emits `e_aq,1.0,-2.5,0.25,1.0`. Exercise `SpeciesInstance` directly through the chemistry root interface.
- [ ] Run `cargo test -p dna-chemistry --test species_interface --locked`, then workspace/default and named reference gates. Capture new snapshots and compare against Task 1: exact scientific identity required.
- [ ] Check dependency trees for `dna-core`, `dna-physics` and `dna-chemistry`. Core must not depend on either stage; neither stage depends on the other. Update core/module docs to acknowledge shared water contracts instead of claiming core contains no physics.
- [ ] Commit and close S01 only after its acceptance checks. No package renaming yet.

## Task 3 — Atomic package/directory/import rename (S02)

**Files:** Rename `crates/dna-{core,data,physics,chemistry,sim}` to corresponding `crates/g4-*`. Update root/crate Cargo manifests, lockfile identities, Rust imports/rustdoc, current CI/gate commands, examples, and executable fixture references in `scripts/hand_reactions.py`, `scripts/hand_physchem.py` and extractor instructions. Update current README/docs entry points and `docs/crate-separation.md`.

**Interfaces:** The same public symbols under `g4_core`, `g4_data`, `g4_physics`, `g4_chemistry`, `g4_sim`. Existing `DnaError` keeps its name; no broad type rename. Serialized formats and data paths remain unchanged.

- [ ] Add a checker assertion for the exact five desired workspace package names. On the pre-rename tree it must report the old names, not success.
- [ ] Rename directories and package names together, update path dependencies, workspace members and `[profile.dev.package.*]` entries. Update Rust import roots and live source links. Regenerate the lockfile only as needed for local package identities; inspect the diff to reject unrelated dependency upgrades.
- [ ] Update live scripts that read moved chemistry test fixtures. Do not run extraction/fixture-generation scripts into existing fixture paths. Do not mass-replace `dna` in schema strings, model names, G4EMLOW paths, environment variables, historic reports, session metadata or reference provenance.
- [ ] Run `cargo metadata --format-version 1 --no-deps --locked` and verify the five members; `cargo check -p g4-physics --all-targets --locked` and `cargo check -p g4-chemistry --all-targets --locked` must pass.
- [ ] Run the complete renamed baseline gates and snapshot comparison. `g4_sim::run` output still contains `geant4-rs/dna-sim/run/v2` and `/v3`, and original CSV/data identity. Update checker capture invocations to `--package g4-sim`; do not overwrite the old baseline.
- [ ] Publish a precise import migration table in the guide, including old `dna_core::species::*` -> `g4_chemistry::*`. Record intentional historical/schema occurrences so searches do not prompt accidental cleanup. Commit the coherent rename and close S02.

## Task 4 — Optional chemistry in simulation (S03)

**Files:** Modify `crates/g4-sim/Cargo.toml`, `src/lib.rs`, `src/error.rs`, `src/run.rs`, snapshot example and chemistry-only test targets. Create `src/run/chemistry.rs`, `tests/physics_only.rs`, `tests/chemistry_feature.rs`. Keep `src/history.rs` physics-facing; its raw handoff records do not require chemistry.

**Interfaces:**

```toml
[features]
default = ["chemistry"]
chemistry = ["dep:g4-chemistry"]

[dependencies]
g4-chemistry = { path = "../g4-chemistry", optional = true }
```

Keep existing `g4_sim::run::{run, run_detailed, run_summary, run_summary_with_sink, run_with_sink, RunOutput, PhysChemSink, CsvPhysChemWriter}` available in both profiles. Keep `run_physchem_summary`, `run_physchem_summary_with_diagnostics`, `run_with_physchem` and `PhysChemRunOutput` at their existing `run` paths only when `chemistry` is enabled. Their existing parameter/return types do not change except crate ownership/imports.

- [ ] Add `physics_only` tests using `RunConfig::new(1000.0, 8)`, seed `20261026`, `record_physchem = true`, one thread/batch 7. Assert eight ordered histories, finite conserved ledgers under existing limits, nonempty raw handoff events, and unchanged handoff CSV header. Missing committed data must fail this required test, not return early.
- [ ] Add `chemistry_feature` (required feature `chemistry`) exercising root species exports and coupled `run_physchem_summary`; assert canonical ordering and unchanged 1 ps output. Check exported symbols by compiling real calls, not string searches alone.
- [ ] Move only coupled functions, `PhysChemRunOutput` and its impl into private cfg-gated `run::chemistry`; re-export them from `run`. Keep CSV handoff sink and physics functions in `run.rs`. Existing `Edges`, `ids`, `with_pool` and `run_batch` remain private shared helpers accessible to the child module. Do not create a new public orchestration trait or change the batching algorithm.
- [ ] Gate `physchem_scoring`, `spatial_diagnostic` and the associated `SimError::Chemistry` / `SpatialDiagnostic` imports/variants. Keep physical `PhysChemEvent`/`PhysChemSite` recording available without the feature. Do not introduce a dev-dependency that unconditionally brings chemistry back.
- [ ] Configure required-feature test targets for `physchem_a`, `physchem_b`, `physchem_audit`, `physchem_reference`, `spatial_diagnostic` and `chemistry_feature`; inspect any newly added tests from R01 for the same need. Keep `determinism`, `physical_sanity`, `geant4_reference`, `slowing_down_reference` and physical examples active without chemistry. Any mixed test file must split tests, not disable the whole physical family.
- [ ] Make snapshot `--physics` compile/run in both profiles. `--physchem` without the feature returns a clear nonzero unsupported-feature error, never empty or successful output. Preserve default-feature snapshots exactly.
- [ ] Run `cargo test -p g4-sim --no-default-features --all-targets --locked`, `cargo test -p g4-sim --test chemistry_feature --locked`, and default workspace checks. Explicitly confirm physics-only tests actually ran and default chemistry tests still run. Commit this tested implementation slice.

## Task 5 — Enforce independence and close the migration (S03)

**Files:** Extend `scripts/check_crate_split.py` and its unittest; update `.github/workflows/ci.yml`, `docs/crate-separation.md`, current architecture/reproducibility guides and supported command lists. Do not edit historic review findings to pretend they described the new tree.

**Interfaces:** Checker `--check-graph` consumes Cargo's resolved normal/build dependency graph for each selected package/profile and exits 1 on forbidden edges. Never infer a dependency merely from its presence in workspace metadata; walk only nodes reachable from the selected root. Add `--capture <new-directory> --package g4-sim --no-default-features` for physics-only capture; it must not attempt the unsupported physchem profile.

- [ ] Write graph-checker fixtures for a valid graph, direct forbidden physics->chemistry edge, transitive forbidden edge and a disconnected chemistry workspace member (must not falsely fail). Add default-sim and no-default-sim cases; unittests must demonstrate detection before wiring CI.
- [ ] Check core has no data/physics/chemistry/sim dependency; physics has no chemistry/sim; chemistry has no data/physics/sim; no-default sim has no chemistry. Run package-scoped checks in separate Cargo invocations so workspace feature unification does not hide mistakes. Inspect dev dependencies and the `--all-targets` build as well.
- [ ] Capture default physics and no-default physics on the same toolchain/platform/input/config and compare exact scientific payloads. Compare default coupled output against the original pre-S01 baseline. Fail on any changed scientific bit, format or data identity; diagnose rather than loosen comparison.
- [ ] Add package-scoped no-default check/test/lint CI beside the existing full default workflow. Add the graph checker and explicit selected-test inventory. `cargo test --workspace --all-features` alone is not proof of optional dependency isolation.
- [ ] Run the full final verification matrix below and record commands, exit codes, selected test counts, input hashes and pre/post revisions in the current guide. If a statistical gate fails, retain the failure and diagnose; do not call the migration accepted based only on compilation.
- [ ] Document `g4-bio` as future-only and note it may consume both physical events and chemical results. Leave `CLAUDE.md`'s `@AGENTS.md` delegation unchanged; no speculative biology API or extra module is needed.
- [ ] Commit the checks/docs and finish S03 through GPS after approval. Do not automatically start R02 or other scientific seeds; report the next approved dependency instead.

## Verification matrix and expected results

Use `dna-sim` for the S01 baseline, `g4-sim` after S02. `--offline` may be added only if required dependencies are already available; offline availability is not an architectural requirement.

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test -p g4-sim --test geant4_reference --locked -- --ignored
cargo test -p g4-sim --test slowing_down_reference --locked -- --ignored
cargo test -p g4-sim --test physchem_reference --locked m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact

cargo check -p g4-physics --all-targets --locked
cargo check -p g4-chemistry --all-targets --locked
cargo test -p g4-sim --no-default-features --all-targets --locked
cargo clippy -p g4-sim --no-default-features --all-targets --locked -- -D warnings
cargo test -p g4-sim --test chemistry_feature --locked
python -m unittest discover -s scripts -p test_check_crate_split.py
python scripts/check_crate_split.py --check-graph
```

Expected: all commands exit 0, maintained reference selection retains its four M2/two M3/one M6 comparisons, and feature-profile tests execute rather than disappear. If R01 changed test names, update the runner's registered selectors explicitly while preserving those scientific comparisons. Snapshot comparison commands run once per named profile against immutable baseline captures; expected exit 0 with no scientific differences. Code/reference behavior is not considered verified until outputs have actually been observed.

Final source check: no code imports `dna_core::species`, no `dna-*` package remains, no forbidden dependency edge, no new biology/shim crate, unchanged reference/data bytes, and only documented changes to runtime provenance identity. Existing data-path strings and historical `dna-sim` schema IDs are correct exceptions to a text search.

## GPS import and run instructions — after approval only

The following are future user commands, not commands executed by this planning task. Use the installed GPS skill and its supported scripts; never hand-edit `.pending-seeds.json`, `.current-session` or session directories.

1. Resolve the currently active obsolete session using the separately approved roadmap migration. Do not switch it implicitly.
2. Import **only S01/S02/S03** from this document, retaining the three grouped scopes:

```text
$gps scout --from docs/superpowers/plans/2026-10-26-physics-chemistry-crate-separation.md only the three seeds S01, S02 and S03; preserve their slugs, prerequisites and acceptance criteria
```

3. After the R01 prerequisite and plan approval, run sequentially:

```text
$gps start crate-domain-ownership
$gps start g4-crate-namespace-migration
$gps start physics-only-simulation-build
```

These are **not a command batch**: complete/review the first session through its normal grill/plan/ship/finish lifecycle before starting the next. Each seed points to this full plan and companion design; substantive choices are already proposed here, so GPS should turn them into tickets, not restart architecture exploration. Any requested change to these choices requires explicit approval.

Acceptance of S03 completes the separation only. Biology, full chemistry, G(t), proton support and the strategic roadmap remain separate work. No seed registration, implementation, commit or PR was performed while writing this plan.
