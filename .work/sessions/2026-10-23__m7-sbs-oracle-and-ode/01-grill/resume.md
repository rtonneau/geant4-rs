# Session: m7-sbs-oracle-and-ode

**Date:** 2026-10-23T14:19:05.995Z
**Status:** Grill phase complete

## Problem Statement

M7 has an implemented isolated two-body independent-reaction-time sampler, but it lacks the two independent validation paths required by the roadmap: a source-faithful Geant4 step-by-step (SBS) oracle for a totally diffusion-controlled pair and a well-mixed stochastic-kinetics comparison against an ODE solution. It also lacks the additive Geant4 two-body reference mode and pre-registered ignored comparison needed to open the external-data gate without an agent running Geant4.

## Context & Constraints

- **Current behavior:** `dna-chemistry` provides free Brownian diffusion and an isolated A+A/A+B IRT sampler with V9 analytic validation. It has no SBS module, no test-only well-mixed SSA/ODE harness, and no Geant4 two-body reference mode.
- **Pain point:** The IRT sampler is currently validated analytically but not against an independently implemented spatial stepping model, a bulk kinetic limit, or Geant4-generated two-body survival data.
- **Dependencies:** Preserve the option3 IRT model and existing public production interfaces. Follow Geant4 11.4.1 executable source semantics. Never run the new C++ mode; compile-check only. Do not overwrite existing generator modes or reference data. Pre-register all statistical criteria before external data exists. End with the Geant4 gate `awaiting data` for the later `m7-sbs-close-gate` session.
- **Tech stack:** Rust 2024; existing `dna-core` deterministic RNG and `dna-chemistry` diffusion/IRT/reaction-table APIs; `rsoda` as a dev-only public Git dependency pinned to commit `ae76eee69d6d494be9253eaee819c77ee962c934`; Geant4 11.4.1 C++ in the additive `validation/geant4/g4rs_ref` executable.

## Success Metrics

- A validation-only Rust SBS oracle reproduces the Geant4 11.4.1 adaptive encounter stepper and Smoluchowski reaction semantics for option3 H+H reaction ID 1 at `r0 = 0.2 nm`, diffusing both molecules independently and covering inclusive contact plus observation-capped Brownian-bridge behavior.
- IRT and SBS survival/reaction CDFs for 100,000 independently seeded pairs agree at `tau = [0.25, 0.5, 1, 2, 4, 16]` within the pre-registered combined three-sigma binomial bounds, with the family definition and fixed seeds stated in the test header.
- A test-only exact Gillespie SSA for option3 reaction ID 2, `e_aq + H -> OHm + H2`, starts with 10,000 molecules of each reactant at 1 micromolar, runs 512 independent histories to 360 microseconds, conserves molecules in every history, and its ensemble means agree with the pinned-`rsoda` LSODA solution within 1%.
- A new compile-checked but never agent-run Geant4 H+H two-body mode accepts output path, seed, and sample count; emits survival counts for the same six observation points; and is documented in the README, BUILD_NOTES, and one new RUN_REQUESTS entry.
- An ignored Rust comparison consumes the future Geant4 CSV plus provenance using tolerances fixed before data exists, and documentation records the gate as `awaiting data` without introducing a production scheduler, production SBS engine, or production ODE API.

## Architecture & Approach

Add a validation-only SBS implementation under `dna-chemistry` that independently transcribes the executable behavior of `G4DNAMoleculeEncounterStepper` and `G4DNASmoluchowskiReactionModel`: for identical H reactants use the Geant4 encounter constant `32 * D_H`, schedule the adaptive step from separation minus reaction radius, diffuse both particle positions through the existing `diffuse` kernel, treat `r <= R` as contact, and apply the Brownian bridge only when a step is capped at an observation boundary. Document the stale Geant4 source comment whose stated constant disagrees with the executable expression. Keep this out of the production chemistry API/scheduler boundary.

Create a pre-registered integration test that compares this SBS oracle with the production IRT sampler for reaction ID 1 at the existing six V9 dimensionless times. Use independent deterministic streams, 100,000 samples for each method, survival counts that include all unreached reactions, and a combined-binomial three-sigma absolute bound at each point.

Create a separate test-only well-mixed kinetics harness. Its exact Gillespie SSA enables only option3 reaction ID 2, treats its products as terminal within this closed one-reaction system, and checks conservation per history. Solve the matching mass-action equations with `rsoda`/LSODA from the public repository at the exact pinned revision, then compare the 512-history ensemble means at 360 microseconds with a 1% relative criterion. The LSODA and SSA helpers remain test-only.

Extend `g4rs_ref` additively with a two-body H+H mode using reaction ID 1, `r0 = 0.2 nm`, the six shared observation times, and explicit output/seed/sample-count CLI arguments. Emit a compact survival-count CSV with total, reacted, and surviving counts plus the fields needed for provenance validation. Compile-check against the configured Geant4 11.4.1 installation but do not run. Add a single user-run request and an ignored Rust consumer whose schema, family, and tolerances are fixed before the CSV exists.

## Assumptions & Trade-offs

- The SBS oracle deliberately follows Geant4's adaptive source semantics rather than using a fixed-step convergence oracle. Independent implementation and tests still provide a separate Rust code path, but the scientific algorithm is source-faithful by design.
- Geant4's inclusive SBS contact boundary differs from the production IRT sampler's equality handling at a measure-zero boundary. Preserve each model's semantics and test/document the distinction rather than forcing artificial equality.
- Use two calls to the existing diffusion primitive instead of a single relative-coordinate walk. The distributions are equivalent, while the selected design better matches the source structure and validates the shipped diffusion code.
- A pinned Git dev-dependency is reproducible and portable today; a local sibling path is forbidden and `rsoda` is not yet published on crates.io. The dependency stays outside the production graph.
- V10 uses a test-only exact SSA because the many-species spatial scheduler belongs to a later M7 session. Reaction ID 2 supplies a validated physical rate instead of inventing one, but products are terminal within this deliberately closed one-reaction test.
- No agent-generated Geant4 CSV or provenance is committed. No existing reference file, generator mode, tolerance, or public production API is replaced.

## Open Questions

None. The external Geant4 comparison intentionally remains open until the project lead runs the registered command and commits the new data and provenance for `m7-sbs-close-gate`.

## Notes

The canonical domain term **Step-by-step (SBS) oracle** was added to `.work/GLOSSARY.md`. Geant4 11.4.1 executable code uses `8 * (D_A + D_B + 2 * sqrt(D_A * D_B))`, which reduces to `32 * D_H` for H+H; a nearby source comment incorrectly says 16 times the parenthesized term, so the executable expression is normative. The new C++ mode follows the established additive, compile-only, user-run request lifecycle in `validation/geant4/RUN_REQUESTS.md`.
