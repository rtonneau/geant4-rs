# Ticket 02: escape-observer-hook

**Status:** ✅ Done

## Local Test Result

`cargo test -p g4-physics`: all ok, 0 failed, no SKIPPED (data present). New test `exit_events_lie_on_the_surface_and_match_the_ledger` passes for the sphere, box and tube.

## Review Notes

Base a2a6bc7. Added `ExitEvent {generation, track_id, position, direction, energy}` and a default `on_escape`, called right after the escape is booked, with the position already moved to the exit point. No RNG involvement, and the outcome is identical with and without the observer (asserted).

## Blockers / Challenges

None

## Commits

- f029574 feat(g4-physics): ExitEvent and HistoryObserver::on_escape (ticket 02)

## Time Spent

2m (ticket-start.js to ticket-complete.js)
