# Implementation Plan

**Session:** m7-sbs-oracle-and-ode
**Date:** 2026-10-23T19:59:19.692Z
**Estimated effort:** 4–6 days

## Strategy

Implement and validate the source-faithful SBS kernel first, then use it in the pre-registered IRT comparison. Add the independent V10 SSA/LSODA oracle next. Once the Rust-side validation passes, add and compile-check the Geant4 reference mode, pre-register its ignored consumer and user-run contract, then update the scientific and reproducibility documentation and run the full workspace gates. The external Geant4 comparison intentionally remains blocked on user-generated data.

## Tickets Overview

- **Ticket 1:** Add the source-faithful, validation-only SBS kernel and deterministic boundary tests.
- **Ticket 2:** Compare IRT and SBS over the six pre-registered H+H survival points.
- **Ticket 3:** Add the test-only Gillespie/LSODA V10 well-mixed oracle.
- **Ticket 4:** Add and compile-check the Geant4 H+H two-body reference mode without running it.
- **Ticket 5:** Pre-register the ignored Geant4 comparison and external run contract.
- **Ticket 6:** Document both oracles, record the open gate, and run the full verification suite.

## Sequencing Rationale

Ticket 2 consumes Ticket 1's SBS interface. Ticket 3 is independent of SBS but follows it so each statistical oracle has a separate commit. Ticket 5 depends on Ticket 4's finalized CSV schema and CLI. Documentation follows the implemented interfaces, compiled generator, and fixed comparison contract so it describes verified behavior rather than a proposal.

## Risks & Mitigation

- **Risk:** A transcription follows the stale Geant4 encounter-constant comment instead of executable behavior. → **Mitigation:** Pin the `32 * D_H` H+H reduction in deterministic tests and cite the executable expression.
- **Risk:** Statistical tests become flaky or criteria drift after results are seen. → **Mitigation:** Fix seeds, sample counts, observation points, comparison families, and tolerances in test headers before the first run.
- **Risk:** The SBS equality boundary is forced to match IRT rather than Geant4. → **Mitigation:** Preserve inclusive SBS contact, keep IRT semantics unchanged, and test/document the measure-zero distinction.
- **Risk:** The unpublished ODE crate makes builds machine-specific. → **Mitigation:** Use the public HTTPS Git repository pinned to exact commit `ae76eee69d6d494be9253eaee819c77ee962c934` as a dev dependency; never commit a sibling path.
- **Risk:** The agent accidentally produces reference data. → **Mitigation:** Compile-check the additive C++ mode only, never execute it, and leave one explicit open run request for the project lead.

## Assumptions

- The existing option3 reaction table and two-body parameter loader remain normative for reaction IDs 1 and 2.
- `DEV_DIR` and `G4_ROOT` provide the machine-specific roots; no resolved machine path enters source or committed build files.
- The new SBS, SSA, and LSODA helpers are validation-only and do not create a production chemistry scheduler or ODE API.
- Future Geant4 CSV and provenance files will be supplied only through `m7-sbs-close-gate`.
