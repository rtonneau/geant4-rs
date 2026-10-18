# Session: m7-option3-reaction-table

**Date:** 2026-10-17T18:11:13.589Z
**Status:** Grill phase complete

## Problem Statement

Milestone M7 needs a validated, reproducible Rust data representation of the complete Geant4 11.4.1 `G4EmDNAChemistry_option3` reaction table. The repository currently has option3 species, dissociation-channel and physicochemical parameter tables, but no chemical reaction table for later IRT/SBS work.

## Context & Constraints

- **Current behavior:** `dna-chemistry` stops at the 1 ps physicochemical boundary. It embeds and validates three option3 chemistry files and has no reaction-network loader or reaction lookup API.
- **Pain point:** Later M7 diffusion and scheduling work cannot consume a source-audited reaction network, and the C++ table contains comment/literal discrepancies, bulk bookkeeping configurations and one malformed-looking stoichiometric entry that make an unchecked transcription unsafe.
- **Dependencies:** The work is independent of M6 closure. Geant4 v11.4.1 executable literals are authoritative; the C++ reference remains read-only. Preserve the existing 15-member mobile `Species` type and the project rule that physical constants and reaction parameters are never guessed.
- **Tech stack:** Rust 2024, `serde`, `toml`, existing `dna-core` units/species and `dna-chemistry` validation patterns; Python standard library for source extraction and independent evaluation.

## Success Metrics

- All 72 executable option3 entries load from `data/chemistry/reactions_option3.toml`, retain stable source-order IDs, exact participants/products/rates and source-line provenance, and match an independent textual extraction of Geant4 11.4.1.
- Every entry passes atom and charge validation with only implicit bulk-water completion, except the single exact-ID/exact-stoichiometry Geant4 anomaly authorized in Rust; malformed tags, IDs, rates, units, duplicates and unauthorized imbalances produce `ChemistryError::Invalid` or `Parse`.
- The immutable `ReactionTable` supports iteration, stable-ID lookup and unordered mobile-pair lookup; the chemistry-data SHA-256 includes the new file, independent Python checks pass, and the named scientific/reproducibility documentation is updated.

## Architecture & Approach

Add `data/chemistry/reactions_option3.toml` and a focused `crates/dna-chemistry/src/reactions.rs` module following the existing table-loader pattern. A reaction participant is either one of the 15 mobile species or one of the four non-mobile bulk participants (`H2O(B)`, `H3Op(B)`, `OHm(B)`, `NoneM`); bulk participants do not extend `Species`. Each record stores its source-order ID, exact ordered reactants and products, Geant4 source class/mechanistic intent, explicit kinetic order and unit, observed-rate coefficient and source line. Mobile pair lookup is unordered, while source order and product multiplicity/order remain intact for provenance.

Validation requires contiguous unique IDs 1 through 72, canonical participant names, finite positive observed rates, the correct rate field/unit for the kinetic order, no unordered duplicate mobile pair, and atom/charge balance after allowing an integral implicit-water residual on either side. The known `O3m + NoneM -> H3Op + O2m` source anomaly is accepted only when both its stable ID and full signature match a code-level allowlist. No generic data flag can disable conservation checks.

Add a textual C++ extractor beside `extract_physchem_from_cpp.py`, a committed JSON fixture, and an independent Python evaluator for count, identity, rate/order semantics and conservation. Rust integration tests compare the TOML table with the extraction and hand evaluation. Extend `option3_data_sha256()` in the documented order. Update the chemistry data README, C++ inventory (including B.7.6-B.7.7 context), the D.2 stage boundary and D.3 reaction description, data requirements, roadmap and reproducibility documentation.

## Assumptions & Trade-offs

Executable C++ literals win over nearby comments; discrepancies are recorded rather than silently corrected. Numeric coefficients and explicit rate units are stored once in TOML and compared as parsed decimal values to the extractor rather than duplicating raw C++ expressions. `k_act`, `k_diff`, reaction/effective radii, probability and model-specific runtime reaction type are derived later and are outside this milestone. Source chemical class and kinetic intent are preserved without pretending Geant4's model-dependent integer type is unconditional. No scheduler, diffusion implementation, reaction execution, C++ generator or Geant4 run is included.

## Open Questions

None. The source-literal policy, participant boundary, conservation rule, anomaly authorization, rate representation and public API boundary were approved during the grill.

## Notes

The authoritative local source is the Geant4 v11.4.1 release tree at `source/physics_lists/constructors/electromagnetic/src/G4EmDNAChemistry_option3.cc`, chiefly `ConstructReactionTable` lines 96-666. The unpacked source tree has no Git metadata, so provenance must name release v11.4.1 and exact relative paths/line ranges without inventing a commit. Five rate comments disagree with executable literals, and the Type-VI comment for the known anomalous entry also disagrees with its executable products. The settled terminology and source-literal decision are recorded in `.work/GLOSSARY.md` and `.work/adr/0002-geant4-reaction-literals.md`.
