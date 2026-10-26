# Ticket 01: validate-table-domains

**Status:** ✅ Done

## Local Test Result

I reran `cargo test -p g4-data -p g4-physics --locked` with `CARGO_TARGET_DIR` in the session scratch directory. It exited 0: 154 tests passed, one pre-existing long statistical test remained ignored, and doc tests passed.

## Review Notes

The ticket was implemented by `gpt-6-astra` at high effort. I reviewed the eight-file diff: catalog and direct constructors enforce exact channel counts; block constructors reject missing value columns; narrow endpoint bracketing returns a typed error; scaled values, sums and queried areas are checked for finite nonnegative results. Regression and property tests cover malformed shapes and overflow. The checked Geant4 11.4.1 source uses five Born levels/shells and nine Sanche levels. I made no follow-up code changes.

## Blockers / Challenges

None.

## Commits

- a14f5ca fix: validate DNA table domains (ticket 01)

## Time Spent

8m (ticket-start.js to ticket-complete.js)
