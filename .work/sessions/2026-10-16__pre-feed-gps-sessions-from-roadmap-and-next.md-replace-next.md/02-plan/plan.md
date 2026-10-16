# Implementation Plan

**Session:** Pre-feed gps sessions from roadmap and next.md, replace next.md
**Date:** 2026-10-16T17:16:08.027Z
**Estimated effort:** 3-4 hours

## Strategy

Documentation and gps-state only; no physics or Rust behaviour changes. First fix the convention the seeds rely on (RUN_REQUESTS.md), then write the backlog that uses it, then record the decisions and roadmap pointers, then remove `docs/next.md` once everything it held is carried elsewhere, and finally register the seeds with `scout-merge.js --from`.

## Tickets Overview

- **Ticket 1:** Create `validation/geant4/RUN_REQUESTS.md` (compile-only, never-run convention and request format) and link it from `validation/geant4/README.md` and AGENTS.md.
- **Ticket 2:** Write `docs/gps-backlog.md` with the 13 seeds (goal, preconditions, C++ tasks, tests, pre-registered acceptance), carrying all of `next.md` sections 1 and (ii).
- **Ticket 3:** Record the confirmed decisions in docs/09 and add seed pointers to docs/08 (including the M8 proton deviation and the Q24 closure).
- **Ticket 4:** Delete `docs/next.md` and fix its remaining references.
- **Ticket 5:** Register the seeds with `scout-merge.js --from docs/gps-backlog.md` and verify `/gps status`.

## Sequencing Rationale

Ticket 2 refers to RUN_REQUESTS.md, so ticket 1 comes first. Ticket 4 deletes `next.md`, so it comes after tickets 2 and 3 have absorbed its content. Ticket 5 needs the final backlog text.

## Risks & Mitigation

- **Risk:** Content of `next.md` is lost in the migration. → **Mitigation:** ticket 4 checks section 1 (diagnosis tasks 1-3, the 900 keV stack overflow note), section 2 and the M7 prompt against the backlog and docs/09 before deleting; `git show` keeps the old file.
- **Risk:** A seed's problem text drifts from the backlog section. → **Mitigation:** ticket 5 builds the entries from the backlog sections and the script validates them.
- **Risk:** A doc edit accidentally touches validated reference data. → **Mitigation:** no file under `validation/reference/` is edited; the final check is `git diff --stat`.

## Assumptions

- Seeds have no dependency field, so dependencies are written as "requires <slug>" in each seed's problem text.
- Horizon is M6 closure through M8; M5, M9, M10, M11 stay roadmap lines.
- `.work/` state is only changed through gps scripts.
