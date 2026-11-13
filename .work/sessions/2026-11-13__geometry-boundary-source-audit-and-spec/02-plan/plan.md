# Implementation Plan

**Session:** geometry boundary source audit and spec
**Date:** 2026-11-13T16:25:29.075Z
**Estimated effort:** 3 hours

## Strategy

Bump the reference version first, so every later citation is 11.4.3. Then write the source audit (facts). Then the spec built on it: geometry ADR, residual questions, acceptance criteria with a run request, roadmap milestone.

## Tickets Overview

- **Ticket 1:** Bump the project reference to Geant4 11.4.3, flag RR-001…008 as 11.4.1 data, and write the 11.4.1→11.4.3 DNA source diff note.
- **Ticket 2:** Add the boundary source audit to docs/04, citing 11.4.3 path:line.
- **Ticket 3:** Spec: Volume enum ADR, residual open questions in docs/09, fixed acceptance criteria in docs/07, a run request logged in RUN_REQUESTS.md, and the finite-volume milestone in docs/08.

## Sequencing Rationale

Citations depend on the version decision, and the spec depends on the audit's findings.

## Risks & Mitigation

- **Risk:** Committing the unrelated uncommitted RR-008 follow-up files → **Mitigation:** stage only named paths per ticket. Never `git add -A`.
- **Risk:** Editing existing reference data → **Mitigation:** add flags only in docs and README text. Never touch CSV or provenance files.

## Assumptions

- The 11.4.1 and 11.4.3 sources are both available locally, and the diff note is derived from them.
- No Rust code changes, so the cargo checks only confirm nothing broke.
