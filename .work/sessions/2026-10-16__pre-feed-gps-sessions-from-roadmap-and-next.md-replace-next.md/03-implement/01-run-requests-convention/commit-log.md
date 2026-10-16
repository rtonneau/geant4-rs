# Ticket 01: run-requests-convention

**Status:** ✅ Done

## Local Test Result

wc -l: 49 lines; grep finds RUN_REQUESTS in AGENTS.md:154 and validation/geant4/README.md:11; git diff --stat -- validation/reference is empty.

## Review Notes

Checked every criterion: rule (new modes only, compile-check, never run), entry format fields, gate flow, empty log table with no-request note, links in both files. No machine paths.

## Blockers / Challenges

None

## Commits

- dca4eb1 docs: add Geant4 run request convention (ticket 01)

## Time Spent

0m (ticket-start.js to ticket-complete.js)
