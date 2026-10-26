# Implementation Plan

**Session:** crate-domain-ownership
**Date:** 2026-10-26T12:08:12.759Z
**Estimated effort:** 1–2 developer-days

## Strategy

Start from merged R01 `main`. Capture and check exact physical and coupled physicochemical output before changing ownership. Then move the existing species types to chemistry, update consumers, and compare new captures with the baseline. Stop on an unexpected starting gate or scientific difference.

## Tickets Overview

- **Ticket 1:** Capture reproducible pre-refactor scientific evidence and run the baseline gates.
- **Ticket 2:** Move species ownership to chemistry and prove exact behavior is preserved.

## Sequencing Rationale

The baseline must be recorded before the type move so its output can detect changed ordering, formats, RNG consumption or scientific values. Each ticket produces a tested, committable result.

## Risks & Mitigation

- **Risk:** Snapshot code misses a scientific field or accidentally accepts changed inputs. → **Mitigation:** Record ordered physical and chemistry output with bit-exact floats; test one-bit, ordering, header and input-identity failures.
- **Risk:** Moving the species type changes names, serialization, event handoff or RNG behavior. → **Mitigation:** Move the implementation without algorithm edits, add public-interface tests, compare same-platform captures exactly, and rerun maintained reference gates.
- **Risk:** The detached starting checkout predates R01. → **Mitigation:** Use merged R01 `main` as the session base and preserve the older refactor branch separately.

## Assumptions

- S01 follows the approved design and covers only evidence plus species ownership. S02 handles crate renaming; S03 handles optional chemistry.
- Reference files and generators remain unchanged. No Geant4 generator is run.
- A baseline failure is diagnosed before changing species ownership; comparison tolerances are not relaxed.
