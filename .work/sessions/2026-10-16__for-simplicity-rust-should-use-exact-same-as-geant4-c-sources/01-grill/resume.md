# Session: For simplicity, Rust should use exact same as geant4 c++ sources for all species

**Date:** 2026-10-16T07:31:23.895Z
**Status:** Grill phase complete

## Problem Statement

Rust currently uses idiomatic or ASCII-normalized aliases for some Geant4-DNA chemistry species (`Oh`, `EAq`, `H2o2`, `Ho2`; external `OH` and `HO2`) instead of consistently preserving the authoritative Geant4 11.4.1 chemistry names. This creates avoidable translation rules and makes it harder to verify species identity against the C++ source.

## Context & Constraints

- **Current behavior:** `Species` contains the 15 mobile/reactive configurations created by `G4ChemDissociationChannels_option1`; its numerical option3 diffusion coefficients, charges, and radii already match the effective C++ runtime configuration. Rust enum variants use ordinary Rust casing, `tag()` emits stable ASCII aliases, and `from_geant4_name()` parses charged `G4Molecule::GetName()` values from validation data.
- **Pain point:** Chemistry spelling and identity are not uniform across the Rust API, data files, serialization, CSV, CLI output, tests, documentation, and Geant4 reference data. In particular, `OH`/`HO2` omit Geant4's degree signs and Rust spellings such as `Oh`/`EAq` obscure the chemistry notation.
- **Dependencies:** Geant4 v11.4.1 is authoritative. Preserve all effective option3 numerical values and the charged runtime-name parser used for reference CSV ingestion. Keep compatibility with old ASCII spellings only through an explicitly named compatibility parser; never emit those aliases.
- **Tech stack:** Rust workspace with TOML chemistry inputs, source-extraction/crosscheck scripts, CSV and JSON-facing simulation output, and the existing cargo test suite.

## Success Metrics

- Every one of the 15 mobile/reactive option3 species has a canonical `chemistry_name()` equal byte-for-byte to its Geant4 `G4MoleculeTable` configuration key, including `°OH`, `e_aq`, and `HO2°`.
- Rust API variants preserve chemistry casing wherever Rust syntax permits (`OH`, `e_aq`, `H2O2`, `HO2`, and analogous names), with narrowly scoped lint allowances where chemistry notation conflicts with Rust naming conventions.
- TOML data, serialization, CSV, CLI output, tests, and documentation emit only canonical chemistry names; exact round-trip and C++ extraction tests cover all 15 names, while legacy ASCII spellings are accepted only by an explicit compatibility parser.

## Architecture & Approach

Retain the closed 15-variant `Species` model, rename variants to chemistry-oriented spellings, and replace the generic `tag()` API with `chemistry_name()`. Treat the returned exact Geant4 configuration key as the canonical identity everywhere outside the enum discriminant. Update embedded TOML and all consumers to use those names directly, retain charged `G4Molecule::GetName()` parsing for validation input, and separate any legacy ASCII parsing from strict canonical parsing. Tighten tests so the extraction crosscheck and round trips compare exact C++ keys without alias translation.

## Assumptions & Trade-offs

The scope is the 15 mobile/reactive option3 species. The four bulk/bookkeeping configurations (`H2O(B)`, `H3Op(B)`, `OHm(B)`, `NoneM`) and transient water electronic states remain excluded until a reaction-engine requirement needs them. Rust cannot use the degree sign in identifiers, so `OH` and `HO2` are the closest legal variants while `chemistry_name()` carries `°OH` and `HO2°`. Ordinary Rust PascalCase is deliberately secondary to chemistry notation for this API. No physical parameters or reference datasets are changed.

## Open Questions

None.

## Notes

The project's glossary defines "chemistry species name" as the byte-for-byte Geant4 configuration key. ADR 0001 records the naming decision and its Rust-syntax boundary.
