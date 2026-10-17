# Session: m6-n1-thermalisation-audit

**Date:** 2026-10-17T12:53:01.699Z
**Status:** Grill phase complete

## Problem Statement

Audit the unresolved M6 N1 thermalisation difference: Geant4 11.4.1 applies a navigator-based `0.8 * safety` limit to a solvated-electron displacement when that displacement reaches a geometry boundary, while Rust has no navigator or clip. Establish whether this difference applies to the project's infinite-water model or the committed M6 reference, whether it can explain the remaining 900 keV RMS-radius excess, and record the evidence without changing scientifically correct behavior merely to match Geant4.

## Context & Constraints

- **Current behavior:** Rust samples the Geant4 option2 `Meesungnoen2002` thermalisation displacement without geometry clipping. Geant4's live chemistry-on code computes the same penetration and shortens it to `0.8 * safety` only when the sampled displacement reaches a boundary. The M6 reference generator uses a 1 m water sphere, primaries start at its centre, and relevant displacements are nanometre-scale. The stage-matched M6 count/RMS gate currently passes, but the seven 900 keV RMS rows remain approximately 12-13% high and within the pre-registered 20% criterion.
- **Pain point:** N1 is still recorded as not audited, leaving ambiguity about whether an omitted Geant4 behavior is a model defect or a possible explanation for the high-energy RMS residual.
- **Dependencies:** Preserve the infinite-water model, existing reference data, generator modes, fixed validation tolerances, deterministic random streams, and the separation between M6 and later chemistry. Never modify the read-only Geant4 repository or run Geant4. Make no production-model change unless the audit reveals a separate source-proven defect.
- **Tech stack:** Rust workspace (`dna-core`, `dna-physics`, `dna-chemistry`, `dna-sim`), Geant4 11.4.1 C++ reference sources, and the existing `g4rs_ref` validation generator and M6 tests.

## Success Metrics

- Source-backed documentation gives exact Geant4 and Rust paths/lines for penetration sampling, navigator safety clipping, the reference geometry, and the infinite-water assumption, and reaches an explicit N1 verdict.
- Focused tests or existing invariant tests establish that thermalisation produces the expected one `e_aq` per solvation and that boundary clipping is inapplicable to the current infinite-water/reference geometry; the separate N2 solvation-count audit is not expanded in scope.
- Documentation states whether N1 can explain the 900 keV RMS excess, keeps that residual honest if unresolved, and leaves formal M6 closure to the project lead.
- No existing reference file or generator mode is changed, and no new Geant4 generator or run request is added unless existing source and test evidence proves insufficient.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` complete successfully, or any inability to run them is reported accurately.

## Architecture & Approach

Perform a bounded source-and-test audit. Trace Geant4's option2 solvation path from penetration sampling through `SampleSecondaries`, navigator `ComputeStep`, the `safety <= displacement` condition, and the `0.8 * safety` replacement. Compare it to Rust's `Solvation::sample` and the handoff into physicochemical events. Establish the geometric scale from the committed generator and the project's infinite-medium contract. Add only focused regression or invariant coverage that materially preserves the conclusion; reuse existing statistical sampling tests rather than duplicating them. Record N1 as not applicable to infinite water if the evidence holds, and explicitly separate that verdict from N2 and from the unexplained 900 keV RMS residual.

## Assumptions & Trade-offs

The current simulation domain is infinite liquid water and the Geant4 validation world is sufficiently large that no transported track or thermalisation displacement reaches its boundary. A finite-geometry API and navigator abstraction are out of scope. N2 is out of scope except for checking the one-solvation-to-one-`e_aq` invariant along the audited path. A new Geant4 diagnostic is avoided because static source and geometry evidence should decide N1; it is introduced only if that evidence becomes insufficient. Passing the existing M6 gate is not treated as project-lead approval to close M6.

## Open Questions

- The physical or statistical cause of the 900 keV RMS excess remains open if N1 is excluded.
- Formal M6 closure remains a project-lead decision.

## Notes

The initial grill inspection corrected an ambiguity in the existing prose: the `0.8 * safety` block is active in Geant4 11.4.1, not commented out. Its condition is geometry-dependent. In the committed reference, the world is a 1 m sphere and the source is at the centre, whereas thermalisation displacements are nanometre-scale; therefore the branch should be unreachable for this validation workload. No glossary addition or ADR is warranted.
