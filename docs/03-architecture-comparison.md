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

## Recommendation: **B**, with these rules
1. Dependency DAG: `core <- physics`, `core <- chemistry`, `{physics, chemistry} <- sim <- cli`. `physics` and `chemistry` never depend on each other; they exchange only `dna-core` types (`PhysChemEvent`, `SpeciesInstance`) via `sim`.
2. Closed sets are enums (`ParticleKind`, `ChargeState`, `ProcessKind`, `ReactionKind`); open sets (cross-section/final-state models, chemistry engines) are traits used as `Box<dyn ...>` **at configuration time only**; per-step hot paths use enum dispatch over a per-(particle, material) `ProcessTable` built once, to avoid dyn calls in the inner loop.
3. A new particle = new `ParticleKind` variant + a `ProcessTable` constructor registering its models; the transport loop is untouched.
4. A new chemistry algorithm = another `impl ChemistryEngine`; transport and scoring untouched. Scoring consumes `PhysicsEvent`s and `ChemistrySnapshot`s through observer traits, never model internals.
5. Each model source file starts with a `//! Geant4 reference:` block (path, class, commit, dataset file, energy range) and a unit test that asserts the documented range.
6. Workspace split is a refactor-later option; if M0 shows friction, collapsing B into A costs one afternoon (modules map 1:1).

Units: **hand-rolled `f64` newtypes** (no `uom`): few quantities, need zero-cost, simple `serde`, and explicit eV/nm/ps. Revisit if >10 dimensions appear. Vectors: own tiny `Vec3` (3 f64), no `nalgebra` (not approved; no need).
