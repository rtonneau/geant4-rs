# Implementation Plan

**Session:** g4-crate-namespace-migration
**Date:** 2026-10-26T14:04:07.537Z
**Estimated effort:** 2 days

## Strategy

Prepare a narrowly validated snapshot comparison mode while the workspace still has its old package names. Then rename the five crates in one commit, update active references and current documentation, and verify exact scientific identity against S01's retained captures and the maintained reference gates.

## Tickets Overview

- **Ticket 1:** Validate local package rename differences in the snapshot checker while keeping its strict mode unchanged.
- **Ticket 2:** Rename all five crates together, update active references, and run the full migration gates.

## Sequencing Rationale

The checker must be able to distinguish planned package identity changes from scientific output changes before the rename. The second ticket commits the coherent package set before its final candidate captures so each capture revision resolves to the corresponding lockfile.

## Risks & Mitigation

- **Risk:** A broad rename changes serialized IDs, data paths or reference evidence. → **Mitigation:** Change only active package/import/path references; compare exact snapshots and audit intentional old-name occurrences.
- **Risk:** Cargo.lock changes third-party versions while updating local package names. → **Mitigation:** Validate committed lockfiles at the captured revisions and reject any unrelated package change.
- **Risk:** A green workspace test hides a missing reference comparison. → **Mitigation:** Run the named M2, M3 and M6 selectors and confirm their selected counts.

## Assumptions

- S01 is merged and its two baseline JSON files remain in the retained evidence directory.
- This session does not add optional chemistry or biology; those are separate work.
- No Geant4 generator or reference fixture rewrite is needed.
