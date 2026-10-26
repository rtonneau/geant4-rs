# Implementation Plan

**Session:** validation-evidence-and-ci
**Date:** 2026-10-26T09:51:11.671Z
**Estimated effort:** 3-5 developer days

## Strategy

Make the existing M2, M3 and stage-matched M6 evidence readers fail closed before adding a versioned inventory and runner. Then require fixture/data checks in Linux and Windows CI, state the pinned compiler support policy and update current-status docs. Preserve all committed reference bytes, numerical thresholds and historical exclusions.

## Tickets Overview

- **Ticket 1:** Validate M2 reference rows, provenance and comparison inputs with exact grid coverage and negative tests.
- **Ticket 2:** Validate M3 reference rows, provenance and comparison inputs with exact grid coverage and negative tests.
- **Ticket 3:** Close the M6 deterministic zero-error gap and test stage-specific evidence rejection.
- **Ticket 4:** Add the versioned gate inventory, required-data preflight and manual statistical runner.
- **Ticket 5:** Enforce required-data checks in Linux/Windows CI and correct compiler and current-status documentation.

## Sequencing Rationale

Tickets 1-3 make each current scientific reader trustworthy without changing its verdict. Ticket 4 names and invokes only the maintained selections after those readers are strict. Ticket 5 connects the required tier to CI and documents the verified status. Each ticket can be reviewed and committed separately.

## Risks & Mitigation

- **Risk:** Stricter parsing changes a valid reference verdict. → **Mitigation:** Keep existing numerical criteria, run each maintained ignored gate after its reader changes and investigate any new failure.
- **Risk:** A gate silently passes with missing or altered evidence. → **Mitigation:** Test empty, truncated, duplicate, incomplete, invalid-number, invalid-metadata and hash-mismatch cases; make required-data preflight fail before Cargo tests.
- **Risk:** CI or docs overstate support. → **Mitigation:** State pinned Rust 1.97.0, distinguish local Windows evidence from remote CI, and label historical or unavailable-platform results.

## Assumptions

- Existing Geant4 11.4.1 CSV and provenance files remain immutable; no generator is run.
- M2's four, M3's two and the named M6 prechem comparison are the maintained selections. Historical post-zero-time M6 tests and scratch diagnostics remain excluded.
- The full statistical tier is an explicit manual command; the required-data tier runs in CI on both Linux and Windows.
- Python standard-library tooling is acceptable for the inventory runner; no new Rust dependency or general workflow engine is needed.
