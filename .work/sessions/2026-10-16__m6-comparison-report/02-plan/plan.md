# Implementation Plan

**Session:** m6-comparison-report
**Date:** 2026-10-16T18:40:44.458Z
**Estimated effort:** 3 hours

## Strategy

Record the already-observed reference failure and generator status before adding the diagnostic, as required by the gate's pre-registration note. Then add the ignored report beside the asserting test, reusing its inputs and simulation setup while keeping the scientific acceptance criteria untouched.

## Tickets Overview

- **Ticket 1:** Record the M6 failure in the roadmap and close Q24 with its documented generator fix.
- **Ticket 2:** Add the full non-asserting M6 species comparison report.

## Sequencing Rationale

The documentation ticket comes first because `physchem_reference.rs` requires a failure observed after inspecting the CSV to be recorded in docs/08 before diagnostic changes proceed. The report then builds on that recorded state and remains independently reviewable.

## Risks & Mitigation

- **Risk:** Diagnostic work could accidentally weaken the scientific gate. -> **Mitigation:** Add a separate ignored test and leave the existing assertion thresholds unchanged.
- **Risk:** Missing CSV rows could hide Rust-only species. -> **Mitigation:** Iterate over all 15 `Species::ALL` entries and treat absent Geant4 rows as zero observations.
- **Risk:** The report could be mistaken for new validation evidence. -> **Mitigation:** Keep it as terminal-only, non-asserting output and do not modify `validation/reference/`.

## Assumptions

- A missing Geant4 CSV row means that species had zero observations at that energy.
- The committed M6 reference CSV and provenance file are complete and remain read-only.
- Production-sized history counts are acceptable for the ignored diagnostic because they make its results directly comparable to the gate.
