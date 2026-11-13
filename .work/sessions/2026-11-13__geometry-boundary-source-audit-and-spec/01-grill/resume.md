# Session: geometry boundary source audit and spec

**Date:** 2026-11-13T16:16:22.428Z
**Status:** Grill phase complete

## Problem Statement

M2–M8 assume an infinite liquid-water medium. A finite volume is a planned extension, but we have neither a source-verified account of how Geant4 handles geometry boundaries for DNA transport and chemistry nor a chosen geometry representation. This session produces the source audit and spec (documents only) and bumps the project's Geant4 reference version from 11.4.1 to 11.4.3.

## Context & Constraints

- **Current behavior:** `docs/04` states "Infinite medium: no geometry boundary in M2-M8; a finite box is a later extension." Chemistry diffusion and IRT are documented as unbounded. AGENTS.md pins Geant4 v11.4.1.
- **Pain point:** The brief assumed the remaining interaction length resets in the next volume. The 11.4.3 source shows it is carried over and rescaled. The fate of displaced species at the boundary was undocumented.
- **Dependencies:** Cite only the Geant4 11.4.3 source. Never overwrite or delete the existing RR-001…RR-008 reference data. Run requests are logged, never run, by agent sessions.
- **Tech stack:** Markdown docs (`docs/04`, `docs/07`, `docs/08`, `docs/09`), the ADR via the domain-doc tool, `validation/geant4/RUN_REQUESTS.md`, AGENTS.md.

## Success Metrics

- `docs/04` has a boundary section citing 11.4.3 path:line for each of these: world-exit kill in `G4Transportation`; the interaction length carried over (not reset) in `G4VEmProcess`; solvation and dissociation displacements clamped to 0.8 × safety; Brownian transport killing a molecule outside water.
- `docs/09` lists the residual questions (boundary handling in IRT, which has no navigator; the fate of out-of-volume dissociation products), each tied to a logged run request.
- An ADR records `enum Volume { Unbounded, Box{half:[f64;3]}, Sphere{r} }`, centred on the origin, an enum rather than a trait.
- `docs/07` holds fixed acceptance criteria (escape fraction and energy deposited inside, against a new run request with tolerances defined in advance). `docs/08` has a finite-volume milestone.
- AGENTS.md and the docs name 11.4.3 as the reference. RR-001…008 are flagged as 11.4.1 data. A note diffs the DNA sources between 11.4.1 and 11.4.3.
- `cargo fmt --check`, `clippy` and `test` still pass (no code change).

## Architecture & Approach

Source audit first, then the spec. Settled facts are recorded as findings with source path:line. Only questions the source cannot settle become open questions, answered by a future Geant4 reference run (generator written later). Geometry is a small closed enum, since the set of shapes is known and a trait would add dispatch with no practical use.

## Assumptions & Trade-offs

Documents only: no engine code and no generator code. The old RR data stays valid, flagged as 11.4.1. Re-runs are left to the user's later decision. The box is axis-aligned and centred on the origin, so no rotations or offsets.

## Open Questions

Boundary behaviour in IRT (no navigator involved); whether a dissociation product placed outside its volume is killed at its next step under IRT scheduling. Both go to the run request.

## Notes

The working tree carries uncommitted RR-008 follow-up work from another session. This session's commits must not include those files.
