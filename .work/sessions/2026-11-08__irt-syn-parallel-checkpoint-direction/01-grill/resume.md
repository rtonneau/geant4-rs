# Session: irt-syn-parallel-checkpoint-direction

**Date:** 2026-11-08T21:20:03.518Z
**Status:** Grill phase complete

## Problem Statement

Redirect the first release toward IRT-syn-only production chemistry with default spatial parallelism and restartable calculations, retaining scientifically validated electron transport.

## Context & Constraints

- **Current behavior:** Electron transport and physicochemical conversion exist. Chemistry includes option3 pair kernels and a serial competing-event history scheduler; spatial parallel IRT-syn and production restart are absent. G2 acceptance remains unresolved.
- **Pain point:** Independent histories do not represent interacting tracks in one chemical volume. Existing option3 scheduler semantics cannot simply be relabeled IRT-syn.
- **Dependencies:** Preserve existing reference evidence and physical implementation. Supersede ADR 0003 explicitly for production chemistry. Reference generators may be authored and compile-checked but runs require user execution through RUN_REQUESTS.md.
- **Tech stack:** Existing Rust workspace, deterministic RNG infrastructure, Rayon/shared-memory workers, existing approved serialization dependencies. No distributed runtime required.

## Success Metrics

- Electron transport and chemistry handoff pass explicitly selected Geant4 comparisons within predefined applicability and statistical criteria.
- Serial and spatially parallel IRT-syn pass model-specific and boundary-focused validation; production chemistry defaults to parallel execution with a one-worker reference mode.
- Instantaneous and timed primary lists populate the same interacting volume correctly. Checkpoint continuation preserves the conditional law and agrees statistically with uninterrupted execution.
- Time-dependent species counts and G-values include explicit energy normalization and provenance.

## Architecture & Approach

One coordinator and shared-memory workers operate on transparent computational subregions of unbounded homogeneous water. The production chemical model is IRT-syn. Input is an explicit electron-primary list with energy, position, direction and arrival time; equal times represent instantaneous irradiation. Track arrival and chemical evolution share a consistent timeline. Molecules from different tracks can react.

Deliver through a source-defined serial IRT-syn baseline, timed injection, synchronized checkpoint/restart, spatial parallelism, and end-to-end validation. Checkpoints preserve molecules, identities, chemistry time, future irradiation, scoring, model/data identities and sufficient stochastic/scheduling state for valid continuation. Bitwise replay is not required. Workers evaluate candidates under coordinator-controlled synchronization and cross-boundary conflict resolution.

## Assumptions & Trade-offs

First release is electron-only and single-machine. Proton transport and mesoscopic chemistry are deferred. Worker boundaries have no physical effect. Existing option3 evidence remains preserved even though IRT-syn becomes the sole production chemistry model. Statistically equivalent restart does not authorize arbitrary resampling of pending events. Parallel speedup must be measured, not assumed.

## Open Questions

Product scope is approved. Engineering investigations must establish exact IRT-syn reference semantics, supported scientific ranges, boundary/search error controls, conditional restart state and performance feasibility before enabling production claims. Project policy names Geant4 11.4.1 while the configured installation is 11.4.3; resolve version provenance explicitly without silently migrating the baseline.

## Notes

Approved by the user on 2026-11-08 for GPS implementation planning, not implementation. Existing supervisor recovery and G2 work must be preserved and reconciled rather than treated as accepted chemistry validation. No simulation code changed or tests executed during this design conversation.
