# Implementation Plan

**Session:** For simplicity, Rust should use exact same as geant4 c++ sources for all species
**Date:** 2026-10-16T07:47:29.939Z
**Estimated effort:** 1 day

## Strategy

Introduce the exact-name API without breaking callers, migrate the workspace and chemistry data to it in one compiling change, then tighten the C++ crosscheck and documentation. This order keeps every ticket independently buildable and separates the public API contract from the mechanical migration and provenance audit.

## Tickets Overview

- **Ticket 1:** Add strict canonical species-name and explicit compatibility parsing APIs.
- **Ticket 2:** Rename chemistry-facing Rust variants and migrate all code, data, fixtures, and output to canonical Geant4 keys.
- **Ticket 3:** Remove alias translation from the C++ crosscheck, update provenance documentation, and run the complete workspace gate.

## Sequencing Rationale

Ticket 1 establishes interfaces that Ticket 2 consumes while temporarily preserving the old API. Ticket 2 performs the breaking migration as a single coherent commit so no committed state mixes naming systems. Ticket 3 independently verifies the result against the C++ source and records the durable naming policy after the implementation is stable.

## Risks & Mitigation

- **Risk:** Unicode degree signs are silently normalized or lost in TOML, fixtures, CSV, or source extraction. → **Mitigation:** Assert byte-for-byte names for all 15 species and compare directly with machine-extracted C++ keys.
- **Risk:** A broad variant rename changes scientific data or reaction behavior. → **Mitigation:** Keep numerical properties and reaction contents fixed, run crate tests after migration, and inspect regenerated fixture diffs.
- **Risk:** Rust naming lints reject chemistry notation. → **Mitigation:** Use the smallest documented lint allowance on the closed enum and keep exact non-identifier symbols in `chemistry_name()`.

## Assumptions

- Geant4 v11.4.1 `G4ChemDissociationChannels_option1` is the naming authority.
- Scope remains the 15 mobile/reactive option3 configurations; four bookkeeping configurations and water electronic states remain excluded.
- Existing committed validation CSV files are evidence and will not be regenerated or modified.
