# Session: serial-irt-history-engine

**Date:** 2026-10-28T23:20:55.681Z
**Status:** Grill phase complete

## Problem Statement

R05 needs a complete, correlated and deterministic serial chemistry history, preserving identity, event timing and spatial correlations while explicitly exposing incomplete histories and scientific limitations.

## Context & Constraints

- **Current behavior:** Physicochemistry produces a 1 ps snapshot; compiled reaction networks, molecule identity, pair IRT and unary primitives exist, but there is no complete history engine.
- **Pain point:** Independent pair primitives cannot represent competing reactions, product histories, stale events or time-dependent counts.
- **Dependencies:** R03 remains required. The approved G1-F source-fidelity gate replaces the original combined R04/G1 implementation prerequisite. G1-F passed on 2026-11-05 (RR-005, Geant4 11.4.3, 200k pairs; closure commit `7071bd3`). Implementation is now unblocked. Follow R05 in docs/reviews/proposed-roadmap.md and preserve existing evidence and physics-only builds.
- **Tech stack:** Rust, g4-chemistry and g4-sim; caller-owned per-history chemistry RNG; Geant4 11.4.1 option3 IRT as the source reference.

## Success Metrics

- R05 fixtures cover empty and nonreacting inputs, contact closure, competition, A+A, unary versus pair, ties, stale events, repeated products, lineage, accounting and explicit resource exhaustion; unsupported networks fail before a history starts.
- Count observations are right-continuous and consume no RNG or movement; changing the observation grid leaves the final state and RNG stream bitwise unchanged. Pre-IRT and post-contact 1 ps counts are distinctly labeled. Same-platform 1/2/8-thread and batch variants match.
- Preregistered clustered and dilute comparisons of indexed versus all-pairs and cutoff multipliers 1/2/4 satisfy the R05 99% interval criteria including rare counts; unresolved bias or precision blocks production acceptance. Report representative memory, event and candidate counts.

## Architecture & Approach

Implement serial history internals in g4-chemistry in vertical slices: pairs, competing triplets, products/unary, then full-network histories. Accept a supported compiled network, immutable input, options and caller RNG; option3 is the production validation target and custom networks support controlled tests. Validate the common 1 ps input and preserve its immutable pre-IRT snapshot. Use monotonic molecule identities and generations and a deterministic event queue; invalidate events involving consumed or changed participants while preserving unaffected events. Canonical candidate ordering and insertion ordinal resolve exact competing-event ties, with the first scheduled event winning. Follow source-ordered product creation and Geant4 lazy partner-time and conditional placement semantics.

Accept ordered observation times from 1 ps through a caller-selected final horizon no later than 1 us. Include events exactly at each observation time, including the final horizon; document the inclusive final boundary as a deliberate difference from Geant4's strict end-time scheduling. Process initial contacts to closure before post-contact counts. Counts, status and aggregate diagnostics are returned without materializing positions; defer position diagnostics. Detailed lineage traces are opt-in and bounded; trace or other resource exhaustion returns a typed incomplete-history error with diagnostics and no successful count result.

Use Geant4's finite-horizon cutoff formula at a candidate default multiplier of 1, with configurable and reported multiplier and horizon; compare 1/2/4 and an all-pairs oracle under the roadmap's preregistered criteria before production acceptance. Use a small test-only wrapper for thread and batch checks; R06 owns the production physics-to-chemistry pipeline.

## Assumptions & Trade-offs

G1-F is the satisfied implementation prerequisite. The original combined G1 was superseded, not passed; independent physical-law limitations remain open. Cutoffs do not imply mathematically zero omitted probability. Preserve explicit reservoirs and the named reaction-53 exception, source provenance and numerical errors. Exclude intra-history parallelism, arbitrary boundaries, track scoring, CLI, event-heap persistence, IRT-syn and V10 closure. A well-mixed oracle is not V10 spatial validation.

## Open Questions

The G1-F implementation prerequisite is resolved. Independent Brownian/physical-law limitations remain open and are not reclassified by source agreement. R05 production acceptance still requires its own cutoff convergence and G2 evidence.

## Notes

Use the acceptance limits, detailed scope and exclusions in docs/reviews/proposed-roadmap.md R05; preserve the distinction between local G2 checks and the later R07 full-track Geant4 count gate.

## Prerequisite amendment (2026-11-05)

Approved scope: update prerequisite records only; implementation has not started.
RR-005 passed all 120 included endpoints at 200k; 24 exact PDC contact endpoints
remain excluded. Analytic, biased-sampler and contact regressions passed separately.
Evidence: data `51c22b6`, provenance `0b96afe`, closure `7071bd3`; see
`docs/validation/g1-f-source-fidelity.md` and `docs/08-roadmap.md`.
The original OPEN dependency above was replaced by the approved G1-F decision;
historical G1 evidence and limitations remain preserved in those documents.

RR-005 targets 11.4.3. This amendment does not migrate all R05 source semantics
from their planned 11.4.1 baseline. Verify each relevant .cc implementation
and record its version and any differences before implementing that behavior.
