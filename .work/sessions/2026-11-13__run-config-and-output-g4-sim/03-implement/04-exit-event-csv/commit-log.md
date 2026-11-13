# Ticket 04: exit-event-csv

**Status:** ✅ Done

## Local Test Result

`cargo test -p g4-sim`: exit 0, 17 binaries ok, 0 FAILED/SKIPPED; tests/escape.rs 4 passed. `cargo check -p g4-sim --no-default-features --all-targets` ok.

## Review Notes

Base 3fbb565. Added `EXIT_CSV_HEADER`, `exit_csv_row`, the `EscapeSink` trait, `CsvEscapeWriter<W>`, and a `RunSinks` struct with `run_summary_with_sinks`/`run_with_sinks`. The old `*_with_sink` functions delegate to them, so the API stays compatible. Rows go out in the sequential fold, so they are in history order; byte-identical for 1/2/8 threads (tested). Chemistry runs get no escape sink (out of scope; their summary still has the escape block).

## Blockers / Challenges

A heredoc escape put literal newlines into the test strings; fixed.

## Commits

- da8878a feat(g4-sim): exit-event CSV sink (EscapeSink, CsvEscapeWriter) (ticket 04)

## Time Spent

2m (ticket-start.js to ticket-complete.js)
