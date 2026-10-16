# Ticket 05: register-seeds

**Status:** ✅ Done

## Local Test Result

scout-merge.js --from docs/gps-backlog.md seeds.json: Seeded 13 idea(s). status.js lists all 13 slugs under 'Scouted ideas not started'.

## Review Notes

Entries built programmatically from the 13 backlog headings (slug = heading); problem = goal + requires (+ RUN_REQUESTS rule where C++ is involved). The only committed file is the archived scout report, the record of the registration; seeds.json stays in scratch.

## Blockers / Challenges

Ticket 04's ticket-complete failed once (docs/next.md already staged as deleted, pathspec error); committed by hand and recorded with --commit. One EPERM on ticket-start (transient lock), rerun worked.

## Commits

- a522780 docs: note seed registration in the gps backlog (ticket 05)

## Time Spent

1m (ticket-start.js to ticket-complete.js)
