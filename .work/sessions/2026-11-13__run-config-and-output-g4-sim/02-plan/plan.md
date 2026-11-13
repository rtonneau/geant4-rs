# Implementation Plan

**Session:** run, config and output (g4-sim)
**Date:** 2026-11-13T21:21:33.283Z
**Estimated effort:** 1 day

## Strategy

Start with the config rename, schema bump and provenance (one break, one schema bump). Then add the physics hook, then the scorers, then the CSV sink, and finish with the docs.

## Tickets Overview

- **Ticket 1:** Rename `RunConfig.geometry` to `volume`, add `Provenance.volume`, bump the schemas to run/v5 and run-unverified/v2.
- **Ticket 2:** g4-physics `ExitEvent` and the `HistoryObserver::on_escape` hook.
- **Ticket 3:** g4-sim `summary.escape` scorers (fraction, tracks per history, energy fraction, spectra).
- **Ticket 4:** `EscapeSink` + `CsvEscapeWriter` exit-event CSV, deterministic in history order.
- **Ticket 5:** Docs: crate-separation feature/import guide, docs/04 D.5/D.6, CHANGELOG.

## Sequencing Rationale

The scorers and the CSV both depend on the observer hook. The docs describe the final API.

## Risks & Mitigation

- **Risk:** The hook changes the random stream or the unbounded output → **Mitigation:** the existing huge-volume equality and unbounded serialisation tests, plus a test that the summary is unchanged apart from the schema.
- **Risk:** Exit-point rounding puts the point off the surface → **Mitigation:** test `inside(point) == Surface` for all three shapes.

## Assumptions

- No Geant4 reference comparison of escape observables in this session.
