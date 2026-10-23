# Session: m7-diffusion-and-two-body-irt

**Date:** 2026-10-23T05:53:38.360Z
**Status:** Grill phase complete

## Problem Statement

`dna-chemistry` has the validated Geant4 11.4.1 option3 species and reaction tables, but it cannot yet diffuse molecules or sample an independent two-body reaction time. Implement source-faithful free Brownian diffusion and an isolated A+B/A+A IRT kernel that can be validated analytically before the later many-species scheduler is built.

## Context & Constraints

- **Current behavior:** M6 produces mobile `SpeciesInstance` records at global time 1 ps, and M7's option3 table preserves reaction literals and intent, but runtime kinetic parameters and chemistry-time sampling are deliberately absent.
- **Pain point:** Later chemistry work has no validated primitive for advancing a molecule or sampling whether and when one isolated pair reacts; mixing Geant4's default option3 IRT with the separate IRT-syn model would also obscure the scientific reference.
- **Dependencies:** M6 must be formally closed before M7 planning or implementation consumes its 1 ps positions. Preserve the existing M6 record/serialization shape, the validated option3 tables, Geant4 11.4.1 provenance, and deterministic per-history `Stage::Chemistry` RNG streams.
- **Tech stack:** Rust workspace crates `dna-core` and `dna-chemistry`; existing `Vec3`, unit types, `SpeciesTable`, `ReactionTable`, ChaCha12 streams, and open-(0,1)/Gaussian sampling helpers; Geant4 11.4.1 `G4DNAIRT` and option3 sources as the normative reference.

## Success Metrics

- Free unbounded 3D diffusion follows the per-axis normal law with variance `2 D dt`; zero time or zero diffusivity returns the unchanged position without consuming RNG draws.
- The isolated sampler covers distinct-molecule A+B and A+A pairs, neutral and charged TDC behavior, neutral PDC behavior, source-faithful contact semantics, explicit infinite-time non-reaction probability, and typed validation/convergence errors.
- Three source-backed option3 regimes each pass six pre-registered analytic curve points with at least 10,000 pairs and every point within 3 sigma: `H + H -> H2` (neutral A+A TDC), `°OH + H -> H2O` (neutral A+B PDC), and `H3Op + OHm -> water` (charged A+B TDC). The 18-point family and thresholds are fixed in test headers before execution.
- A+A symmetry, same-identity rejection, contact/boundary/extreme behavior, and bitwise-equal results for 1, 2, and 8 threads are tested.
- Documentation 03, 04, 08, 09, 10, and `reproducibility.md` records the implementation, validation, scope, and any deliberate deviations from Geant4.

## Architecture & Approach

Add focused diffusion, reaction-parameter, and isolated-IRT modules under `dna-chemistry`. Keep `SpeciesInstance` unchanged and introduce a chemistry-layer molecule wrapper with a stable `MoleculeId`; pair construction rejects the same identity twice while allowing two molecules of the same species. Kernels accept caller-provided RNGs; integration obtains the existing per-history chemistry stream.

Derive runtime parameters from the validated species/reaction tables using the exact option3 rules, including the A+A diffusion/rate factor, physical contact radii, `k_diff`, `k_act`, encounter probability, and charged effective-distance transform. Reject non-finite or physically invalid inputs and reject PDC configurations with `k_obs >= k_diff` instead of allowing a non-positive activation rate.

Treat `G4DNAIRT::GetIndependentReactionTime` as normative and do not mix in the separate IRT-syn diffusion-controlled model. Preserve its strict contact branch and PDC rejection envelope. Represent results as `Immediate`, `After(Time)`, or `NoReaction`; after 10,000 rejected PDC attempts return a typed convergence error rather than Geant4's negative-time sentinel. Return elapsed time relative to the pair's current state.

## Assumptions & Trade-offs

The domain is infinite 3D space. Finite boxes and boundary conditions are deferred. This session stops at diffusion primitives, validated pair parameters, and one independent reaction-time outcome; reaction products, reaction-site placement, competing pairs, chronological queues, resampling, SBS, ODE comparison, C++ generators, and the many-species scheduler are out of scope. A pair with zero total relative diffusivity is invalid for IRT even though zero-diffusivity free diffusion is a valid unchanged operation. Open-(0,1) Rust uniforms exclude Geant4's theoretical endpoint draws without changing the distribution; this deviation is documented.

## Open Questions

None. The project lead confirmed that the Geant4 simulations and comparisons needed to close M6 have been performed; formal M6 session closure remains a prerequisite before this plan is implemented.

## Notes

ADR 0003 records the choice of option3's independent reaction-time model over IRT-syn and the replacement of overloaded sentinels with explicit outcomes/errors. The glossary defines isolated two-body system, independent reaction-time outcome, and molecule identity. The charged TDC validation is explicitly a Geant4 effective-distance fidelity test rather than a claim of a general exact electrostatic solution; PDC analytic curves use non-overlapping initial separations, with contact handled by separate boundary tests.
