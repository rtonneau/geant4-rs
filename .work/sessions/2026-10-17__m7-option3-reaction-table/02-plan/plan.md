# Implementation Plan

**Session:** m7-option3-reaction-table
**Date:** 2026-10-17T18:23:56.125Z
**Estimated effort:** 2 days

## Strategy

Establish the source-of-truth data first, then add the Rust domain model and validation around that fixed input. Add an independent evaluator and end-to-end cross-checks only after both representations exist. Finish by documenting the audited semantics, provenance and deferred kinetics, then run the full repository gates.

## Tickets Overview

- **Ticket 1:** Extract all 72 option3 reactions from Geant4 11.4.1 and commit the exact TOML and C++ fixture.
- **Ticket 2:** Add the immutable Rust reaction table, lookup API, strict validation and fingerprint coverage.
- **Ticket 3:** Add the independent Python evaluation and complete Rust/C++ cross-checks.
- **Ticket 4:** Document the reaction table and run formatting, linting and workspace tests.

## Sequencing Rationale

Ticket 1 fixes the authoritative input and stable IDs before Rust types depend on them. Ticket 2 implements the public boundary and validation against that data. Ticket 3 checks both layers with an independent calculation. Ticket 4 documents the final tested behavior and performs the broad repository verification.

## Risks & Mitigation

- **Risk:** A permissive parser silently changes a Geant4 literal or accepts an invalid network. → **Mitigation:** Compare all 72 records with a textual extraction and reject every malformed field or imbalance except the exact reaction-53 signature.
- **Risk:** Bulk bookkeeping configurations pollute the mobile species model. → **Mitigation:** Represent them through `ReactionParticipant` without extending `Species`.
- **Risk:** Model-dependent Geant4 reaction types are mistaken for fixed source data. → **Mitigation:** Store source class and kinetic intent only; defer runtime type and derived kinetic quantities.
- **Risk:** The local unpacked Geant4 tree lacks commit metadata. → **Mitigation:** Record release v11.4.1 plus exact relative paths and line ranges, without inventing a commit.

## Assumptions

- `DEV_DIR` resolves the configured Geant4 v11.4.1 source tree and remains available for extractor verification.
- Executable C++ literals are authoritative even when adjacent comments disagree.
- The work is independent of M6 closure and does not implement diffusion, scheduling or reaction execution.
- Python standard library, existing approved Rust dependencies and current table-loader conventions are sufficient.
