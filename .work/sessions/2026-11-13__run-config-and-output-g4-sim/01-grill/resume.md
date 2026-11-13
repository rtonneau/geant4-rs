# Session: run, config and output (g4-sim)

**Date:** 2026-11-13T21:15:45.045Z
**Status:** Grill phase complete

## Problem Statement

Finite-volume transport works (#32), but the run layer only reports an escape count and the mean escaped energy. The config key is named `geometry`, a word the glossary says to avoid. The provenance record does not carry the volume, and there is no escape fraction, no escaped-energy spectrum and no exit-position output.

## Context & Constraints

- **Current behavior:** `RunConfig.geometry: Option<Volume>` (TOML `[geometry]`) is passed to transport. The summary has `terminations.escaped` and `energy.escaped_ev`, both omitted when zero. The schemas are `geant4-rs/dna-sim/run/v4` and `run-unverified/v1`. `Provenance` (g4-core) has no geometry. `HistoryObserver` has no escape hook.
- **Pain point:** No way to analyse what leaves the volume (fraction, spectrum, where it exits), and the record cannot be reproduced from provenance alone.
- **Dependencies:** Keep the random stream and the unbounded code path unchanged. Keep the existing `terminations.escaped` and `energy.escaped_ev` fields. Keep deterministic output regardless of thread scheduling. Follow the PhysChemSink/CsvPhysChemWriter pattern.
- **Tech stack:** Rust workspace (g4-core, g4-physics, g4-sim), serde/serde_json/toml, rayon.

## Success Metrics

- `RunConfig.volume` (`[volume]`) replaces `geometry` with no alias; a `[geometry]` key is rejected. The schemas are run/v5 and run-unverified/v2 for both the physics and chemistry runs.
- `Provenance.volume: Option<Volume>` round-trips in the same tagged shape and is skipped when None. Unbounded run output differs from before only in the schema string.
- `HistoryObserver::on_escape` reports an exit event with generation, the exit point on the surface (`inside == Surface`), direction and energy. The summed event energies equal `ledger.escaped`, and the random stream is unchanged.
- `summary.escape` is present only with a volume and holds: the primary escape fraction ± binomial SE, escaped tracks per history (mean ± SE), the escaped energy fraction (mean E_escaped / E_in), and escaped-energy spectra (primary, secondaries, all) on `spectrum_edges_ev`.
- `EscapeSink` + `CsvEscapeWriter` write one row per exit event in history order. The output is byte-identical across thread counts.
- docs/crate-separation.md (the feature and import guide), docs/04 D.5/D.6 and CHANGELOG are updated. fmt, clippy and tests pass.

## Architecture & Approach

1. g4-sim config: rename the field and the TOML key, update tests and docs.
2. g4-core provenance: add an optional `volume` field with a builder (`with_volume`). g4-sim run sets it from the config.
3. g4-physics transport: add an `ExitEvent { generation, position, direction, energy }` and a default no-op `on_escape` hook, called where the escape is booked. The exit point is pre-step position + d_out * direction.
4. g4-sim scoring: a per-history escape accumulator in HistoryResult, merged into an `EscapeSummary` (`Option` in RunSummary, skipped when None).
5. g4-sim run: an `EscapeSink` trait and a `CsvEscapeWriter<W>`, with rows emitted in history order like the physchem sink. Bump the schema strings.

## Assumptions & Trade-offs

- No backward-compatible `geometry` alias: the schema bump marks the break.
- Exit positions go to CSV rather than a JSON histogram, so the JSON stays small and the CSV keeps full detail.
- No user-configurable spectrum edges.
- No Geant4 reference comparison of escape observables in this session (no generator mode exists yet).

## Open Questions

None.

## Notes

New glossary terms: Escape, Exit event. Size: standard.
