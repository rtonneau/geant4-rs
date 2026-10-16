# Session: m6-comparison-report

**Date:** 2026-10-16T18:35:24.469Z
**Status:** Grill phase complete

## Problem Statement

The ignored M6 Geant4 reference gate stops at its first failed assertion, currently `1000 eV H2O2: z = 111.6`. It therefore hides the remaining count and spatial disagreements. Add a non-asserting report that prints the complete Rust-versus-Geant4 comparison without weakening the pre-registered gate.

## Context & Constraints

- **Current behavior:** `cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture` runs the M6 reference gate for 1, 10, 100, and 900 keV, but the first failed assertion prevents inspection of later species and energies.
- **Pain point:** The project cannot tell whether the M6 discrepancy is limited to a few species or is broad across counts and RMS radii.
- **Dependencies:** Reuse `validation/reference/physchem.csv`, `provenance_m6.toml`, the existing Rust simulation setup, history counts, and master seed. Keep the asserting gate and every tolerance unchanged. Do not modify C++, regenerate reference data, or touch any file under `validation/reference/`.
- **Tech stack:** Rust workspace tests in `dna-sim`, the existing `PhysChemSummary` scorer, Geant4 11.4.1 CSV reference data, and Markdown project documentation.

## Success Metrics

- A second ignored, non-asserting test prints one fixed-width table for each of 1, 10, 100, and 900 keV, with all 15 canonical species in every table.
- Each row shows Geant4 mean +/- SE, Rust mean +/- SE, signed combined-error z-score, signed relative difference, and both RMS radii; undefined metrics print `N/A`.
- The existing asserting test and its thresholds remain unchanged, `docs/08-roadmap.md` records the observed M6 failure, `docs/09-open-questions-and-risks.md` closes Q24 with a pointer to `validation/geant4/BUILD_NOTES.md` section 5, and no reference file changes.

## Architecture & Approach

Add the report beside the existing gate in `crates/dna-sim/tests/physchem_reference.rs` so both paths share the CSV loader and simulation setup. Build a complete energy-by-species view from `Species::ALL`; a species absent from the Geant4 CSV has zero observations. Compute signed `z = (Rust - Geant4) / combined SE` and signed relative difference `(Rust - Geant4) / Geant4`, using `N/A` when the combined standard error or denominator is zero. Print canonical chemistry names and group rows into one fixed-width terminal table per energy. The report never asserts scientific agreement.

## Assumptions & Trade-offs

Missing Geant4 CSV rows represent zero observed instances, not unknown data. The diagnostic uses the same expensive production-sized simulations as the gate to keep values directly comparable. Fixed-width human-readable output is preferred over a new machine-readable artifact because this task is diagnostic and must not create or replace validation evidence. No glossary term or ADR is needed.

## Open Questions

None.

## Notes

The committed reference data use Geant4 11.4.1 with 20,000, 2,000, 200, and 20 events at 1, 10, 100, and 900 keV. Q24's generator crash is already fixed by the changes documented in `validation/geant4/BUILD_NOTES.md` section 5; this session records that closure but makes no C++ change.
