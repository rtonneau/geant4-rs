# Implementation Plan

**Session:** m6-diagnose-and-fix
**Date:** 2026-10-16T19:55:49.104Z
**Estimated effort:** 4-6 days

## Strategy

First establish what the committed Geant4 snapshot actually contains. The presence of `H2O2`, `HO2°`, `O^-`, and `O2`, which the option1 dissociation channels cannot emit directly, makes snapshot timing the first gate: Rust must not be changed to match a reference that already includes chemical reactions. Once the semantics are proven from Geant4 11.4.1 source, audit the two independently testable Rust boundaries: physicochemical conversion, then physical-cascade and scoring behavior. Apply only source-backed fixes with focused tests. Finally rerun the unchanged gate and either close M6 or prepare the smallest new diagnostic mode and end `awaiting data`.

## Tickets Overview

- **Ticket 1:** Establish whether the Geant4 snapshot precedes or follows zero-time reactions and pin the distinction in diagnostics and documentation.
- **Ticket 2:** Audit dissociation, emission timing, and recombination ordering; correct only verified physicochemical-conversion defects.
- **Ticket 3:** Audit initial-cascade equivalence and count/RMS scoring semantics; correct only verified transport or scoring defects.
- **Ticket 4:** Rerun the fixed gate, document every deviation, and either close M6 or prepare a new diagnostic-data request.

## Sequencing Rationale

Snapshot semantics determine whether the aggregate reference is a valid M6 oracle, so Ticket 1 precedes all model changes. Tickets 2 and 3 then isolate the chemistry and transport/scoring boundaries while building on that interpretation. Ticket 4 consumes their evidence and is the only ticket allowed to declare closure or create an `awaiting data` boundary.

## Risks & Mitigation

- **Risk:** Rust is altered to reproduce chemical-stage products outside M6. → **Mitigation:** prove the Geant4 callback and reaction order before authorizing any model change.
- **Risk:** Statistical noise is mistaken for a model defect. → **Mitigation:** use the full four-energy pattern, deterministic ablations, source inspection, and the unchanged pre-registered gate.
- **Risk:** Aggregate counts cannot distinguish ordering or timing hypotheses. → **Mitigation:** add a new diagnostic mode and consuming test with criteria fixed before the user runs it; never alter the existing mode or reference data.
- **Risk:** A broad fix changes RNG or validated M2/M3 behavior. → **Mitigation:** make focused changes, retain history-local streams, and run the full workspace verification before completion.

## Assumptions

- The committed reference and provenance are evidence, but the description “before diffusion/reactions” remains a hypothesis until checked against scheduler source and observed products.
- The existing `physchem` mode, reference CSV, provenance, and pre-registered tolerances remain unchanged.
- Geant4 generator code may be written and compile-checked but never run by an agent session.
- The 900 keV stack overflow is investigated only if it recurs, in which case a stack trace is required.
