# Ticket 03: escape-summary

**Status:** ✅ Done

## Local Test Result

`cargo test -p g4-sim`: 17 test binaries ok, 0 FAILED, 0 SKIPPED. New tests/escape.rs: 3 passed.

## Review Notes

Base 392c4bc. HistoryResult now carries `exits: Vec<ExitEvent>` (forwarded through the Observers combinator). Scorer gets an opt-in `with_escape_scoring()`, which run.rs `scorer_for` turns on when the config has a volume. `EscapeSummary` holds: primary_escape_fraction (FractionStat, binomial SE), escaped_tracks_per_history, escaped_energy_fraction, and spectra primary/secondaries/all on spectrum_edges_ev. A dedicated `EscapeSpectrum` with an `n_tracks` field avoids misusing `n_secondaries`. The primary is defined by an exit event of generation 0. ExitEvent derives PartialEq (needed by HistoryResult).

## Blockers / Challenges

None

## Commits

- 0c30a79 feat(g4-sim): escape summary (fraction, tracks, energy, spectra) (ticket 03)

## Time Spent

3m (ticket-start.js to ticket-complete.js)
