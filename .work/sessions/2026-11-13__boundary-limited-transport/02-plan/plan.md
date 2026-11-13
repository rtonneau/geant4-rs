# Implementation Plan

**Session:** boundary-limited transport
**Date:** 2026-11-13T20:32:42.122Z
**Estimated effort:** 1 day

## Strategy

Restore the deleted GEO-0 spec first, so the implementation cites it. Then change the g4-physics transport loop behind an `Option<&Volume>` parameter with every existing caller passing None. Last, wire g4-sim and add the end-to-end and thread tests.

## Tickets Overview

- **Ticket 1:** Restore the GEO-0 spec (docs and ADR 0005) deleted by the geometry-primitives merge, updated to the shipped Volume.
- **Ticket 2:** Boundary-limited steps, escape, `Termination::Escaped`, `ledger.escaped` and outside-primary rejection in g4-physics, with tests.
- **Ticket 3:** g4-sim accepts and passes geometry, scores Escaped, and tests thread determinism; docs updated.

## Sequencing Rationale

The spec comes first because the code and docs reference it. Physics must exist before g4-sim can pass a volume through.

## Risks & Mitigation

- **Risk:** Unbounded output changes subtly → **Mitigation:** the None branch is structurally untouched; a bitwise regression test compares outcomes and the event streams.
- **Risk:** Restoring the docs overwrites the newer D.5 primitives text → **Mitigation:** merge by hand and keep both; D.5 audit becomes D.6 if needed.

## Assumptions

- Water-only world: the boundary always kills, so the interaction-length carry-over does not apply.
