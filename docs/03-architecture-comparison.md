# C. Architecture comparison

Current repo state: one placeholder crate (`geant4-rs`, edition 2024, `src/lib.rs` with an `add` stub). Nothing to preserve beyond the name/edition.

## Options
**A. Single crate, module tree.** `core`, `data`, `physics`, `transport`, `physchem`, `chemistry`, `scoring`, `sim`, `config`, `bin/cli`.
**B. Small layered workspace (4 library crates + CLI).**
- `dna-core`: units, vectors, RNG streams, particle kinds/states, material, errors.
- `dna-physics`: cross-section data loading/interpolation, per-model sampling, process tables, *and* the track-structure transport loop (they are tightly coupled; splitting `transport` out is deferred until a second user of the models exists).
- `dna-chemistry`: species, reaction network, physicochemical stage, SBS/IRT behind a `ChemistryEngine` trait.
- `dna-sim`: config (serde/toml), history orchestration (rayon), scoring/statistics, output metadata.
- `dna-cli`: clap binary only.
**C. Data-oriented ECS/SoA engine.** Particles/species as SoA arrays, systems stepping all tracks per process; generic over dimensions.

| Criterion | A | B | C |
|---|---|---|---|
| Modularity / enforced boundaries | modules only; cycles possible | crate DAG enforces physics !-> chemistry | high but unfamiliar |
| Extensibility (new particle/chem algorithm) | good | good (new model = new impl + registry entry) | requires system rewiring |
| Performance | fine | fine (same codegen; LTO off-crate cost negligible) | best for chemistry neighbour loops, unproven for transport (branchy, low batch width) |
| Rust ergonomics / compile time | best early | good; incremental builds per crate | poor, lots of indexing boilerplate |
| Testability | integration tests need whole crate | stage crates tested alone; physics tests do not link chemistry | harder to unit-test |
| Complexity | lowest | moderate | highest |
| Scientific traceability | ok | ok; each model file carries its C++ source tag | diluted |

**Implemented so far:** `dna-core` (M0), `dna-data` (M1), `dna-physics` and `dna-sim` (M2, extended in M3). `dna-physics` holds models (M3: Sanche, Melton, solvation), `ProcessTable`, the cascade transport loop with its LIFO secondary stack, and the physicochemical record `PhysChemEvent` (`physchem.rs`; M3 defines it in `dna-physics`, not in `dna-core`, because only transport produces it, and `dna-sim` hands it to the future chemistry crate, so the DAG rule below still holds: chemistry will depend on `dna-physics` for this one plain-data type only, or the type moves to `dna-core` when M6 starts; decision for M6); no rayon, no JSON. `dna-sim` holds `RunConfig`, the per-history driver and its observers, the scorers (`Scorer` consumes `HistoryResult`s only), the physchem sink/CSV writer and the JSON output. `dna-chemistry` and `dna-cli` do not exist yet.

**M6 part A:** `dna-chemistry` (depends on `dna-core` only) holds the physicochemical stage. The record types that cross the stage boundary moved to `dna-core` as announced above: `PhysChemEvent`/`PhysChemKind` (`physchem.rs`), `Species`/`SpeciesInstance` (`species.rs`); so did the helpers both stage crates need (`sampling.rs`: uniform and Gaussian variates, `penetration.rs`: the Meesungnoen2002 law shared by solvation in `dna-physics` and by the auto-ionisation displacer in `dna-chemistry`, `testing.rs`: `ScriptedRng`). `dna-physics` re-exports them, so M2/M3 paths are unchanged. `dna-sim` depends on `dna-chemistry` only as a dev-dependency (the end-to-end test `physchem_a.rs`) until session B wires the stage into the runs.

**M6 part B:** `dna-sim` owns orchestration/scoring: physics produces `Vec<PhysChemEvent>`, `PhysChemEngine` consumes it on `Stage::PhysChem`, and `PhysChemScorer` consumes `HistoryChemistry`. The reduction is fixed-order after each parallel physics batch. Neither physics nor chemistry depends on `dna-sim`; `PhysChemEvent` remains in `dna-core`.

**M7 isolated two-body slice:** `dna-chemistry` adds free diffusion (`diffusion`), stable molecule identity and pair validation (`molecule`), validated runtime parameters (`reaction_parameters`), and an independent reaction-time kernel (`irt`), supported by private special functions. These modules consume `dna-core` species, units, vectors and caller-owned RNGs. The production boundary remains kernel-level: no event queue, competing-pair resampling, reaction products/site placement, or many-species scheduler exists. The option3 source reaction table is validated data; supported mobile bimolecular reactions can have parameters derived, while the sampler handles TDC and neutral PDC. Charged PDC, bulk participants and first-order entries are rejected by the sampler/parameter API as applicable. See ADR 0003 for choosing option3 IRT and explicit outcomes/errors over IRT-syn and overloaded sentinels.

Two independent validation paths remain outside that production boundary. `sbs` is a validation-only transcription of the Geant4 11.4.1 adaptive TDC encounter stepper, not a selectable chemistry engine. `tests/kinetics_ode.rs` contains an exact Gillespie SSA and an `rsoda` LSODA mass-action solve; both are test-only and expose no production ODE API. The additive `g4rs_ref two_body_h_h` mode and its ignored Rust consumer form the external L3 gate, which is still **awaiting data** under RR-003.

## Recommendation: **B**, with these rules
1. Dependency DAG: `core <- physics`, `core <- chemistry`, `{physics, chemistry} <- sim <- cli`. `physics` and `chemistry` never depend on each other; they exchange only `dna-core` types (`PhysChemEvent`, `SpeciesInstance`) via `sim`.
2. Closed sets are enums (`ParticleKind`, `ChargeState`, `ProcessKind`, `ReactionKind`); open sets (cross-section/final-state models, chemistry engines) are traits used as `Box<dyn ...>` **at configuration time only**; per-step hot paths use enum dispatch over a per-(particle, material) `ProcessTable` built once, to avoid dyn calls in the inner loop.
3. A new particle = new `ParticleKind` variant + a `ProcessTable` constructor registering its models; the transport loop is untouched.
4. A new chemistry algorithm = another `impl ChemistryEngine`; transport and scoring untouched. Scoring consumes `PhysicsEvent`s and `ChemistrySnapshot`s through observer traits, never model internals.
5. Each model source file starts with a `//! Geant4 reference:` block (path, class, commit, dataset file, energy range) and a unit test that asserts the documented range.
6. Workspace split is a refactor-later option; if M0 shows friction, collapsing B into A costs one afternoon (modules map 1:1).

Units: **hand-rolled `f64` newtypes** (no `uom`): few quantities, need zero-cost, simple `serde`, and explicit eV/nm/ps. Revisit if >10 dimensions appear. Vectors: own tiny `Vec3` (3 f64), no `nalgebra` (not approved; no need).
