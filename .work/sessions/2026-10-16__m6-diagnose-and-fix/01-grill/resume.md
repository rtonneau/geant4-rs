# Session: m6-diagnose-and-fix

**Date:** 2026-10-16T19:53:49.341Z
**Status:** Grill phase complete

## Problem Statement

Rust M6 disagrees with the Geant4 11.4.1 initial physicochemical-species reference at 1 ps. The first asserting failure is `1000 eV H2O2: z = 111.6`; the completed comparison report exposes the full count and RMS differences. Diagnose their causes, fix verified Rust model defects, and document every intentional or unresolved deviation with evidence.

## Context & Constraints

- **Current behavior:** M0-M3 and the M6 physicochemical engine are implemented. `validation/reference/physchem.csv` and `provenance_m6.toml` contain the Geant4 reference, and `m6_initial_species_comparison_report` prints all four energy tables. The pre-registered asserting gate remains open.
- **Pain point:** The current species-count and spatial discrepancies are not yet attributed to channel branching, species timing, recombination ordering, scoring semantics, known model differences, or the physical cascade.
- **Dependencies:** `m6-comparison-report` is merged. Preserve its asserting thresholds and all files in `validation/reference/`. Verify physics against Geant4 11.4.1 source. Agent sessions may add and compile-check a new diagnostic generator mode but must never run it.
- **Tech stack:** Rust workspace crates `dna-core`, `dna-data`, `dna-physics`, `dna-chemistry`, and `dna-sim`; Geant4 11.4.1 C++ source and the repository's validation generator; Cargo tests and documentation in `docs/`.

## Success Metrics

- Each discrepancy is traced through the dissociation channels, 1 ps emission timing, recombination and event ordering, count-time and RMS definitions, Q12/Q13/Q16/Q17, and cascade settings using source-backed evidence.
- Verified Rust defects are fixed with focused tests; no pre-registered tolerance is changed and no reference file is regenerated or overwritten.
- The M6 gate passes, or every remaining failure is documented explicitly in `docs/08-roadmap.md` and `docs/09-open-questions-and-risks.md`. If new Geant4 data are required, the session ends as `awaiting data` with a compile-checked diagnostic mode and a logged run request.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` pass before completion.

## Architecture & Approach

Work one hypothesis at a time. Establish the full Rust-versus-Geant4 discrepancy pattern from the existing report, then trace each contributing stage from physical events through physicochemical conversion and scoring. Compare Rust behavior with the exact Geant4 11.4.1 implementation before changing code. Add small diagnostic or regression tests around each confirmed cause, apply the minimum correction, and rerun the report and gate after each independently committable change. Keep scientific diagnosis, implementation fixes, and final validation documentation separable in the plan.

If the existing reference cannot distinguish competing hypotheses, add a new diagnostic mode without changing the existing `physchem` mode. Compile-check it, document its arguments and schema, and add a request to `validation/geant4/RUN_REQUESTS.md`; do not run it during the agent session.

## Assumptions & Trade-offs

The committed Geant4 reference and provenance are treated as evidence unless source inspection identifies a generator defect. Regenerating `physchem.csv` is outside scope and requires a separate user request. Scientific correctness takes priority over matching Geant4 when a difference is intentional, but each such difference must be named and evidenced. Statistical tolerances remain fixed. The investigation may touch transport, chemistry, scoring, tests, and documentation only where the evidence shows they contribute to M6.

The unexplained 900 keV Geant4 stack overflow is not addressed unless it recurs; recurrence requires a captured stack trace rather than another stack-size increase.

## Open Questions

Whether the existing aggregate reference is sufficient to isolate every cause. If it is not, the plan must stop at an evidence-backed `awaiting data` boundary after preparing the required diagnostic generator and comparison test.

## Notes

The required hypothesis set is: option1 dissociation-channel branching, including `H2O2`, `H2`, `O`, and `OH^-`; species born at exactly 1 ps versus later decays; Q21/R14 recombination and ordering; the first scheduler step with `dt > 0`; RMS distance from the primary origin over all alive molecules; Q12, Q13, Q16, and Q17; and equivalence of the initial cascade with all secondaries transported to solvation and fluorescence/Auger disabled.

No new glossary term or ADR is warranted by the approved design.
