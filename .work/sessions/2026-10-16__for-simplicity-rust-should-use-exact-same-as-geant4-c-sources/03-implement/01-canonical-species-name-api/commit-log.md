# Ticket 01: canonical-species-name-api

**Status:** ✅ Done

## Local Test Result

`cargo test -p dna-core species`: 5 passed, 0 failed; integration binaries ran with all unrelated tests filtered out. Rust emitted only filesystem hard-link fallback warnings from the incremental cache.

## Review Notes

Reviewed the `gpt-6-sol` delegate's diff against every criterion. All 15 byte-for-byte configuration keys are asserted in Geant4 order, strict parsing rejects the former `OH` and `HO2` aliases, the explicitly named legacy parser accepts them, and charged runtime-name parsing is unchanged. No follow-up edits were needed.

## Blockers / Challenges

None.

## Commits

- e385913 refactor: add canonical chemistry name API (ticket 01)

## Time Spent

3m (ticket-start.js to ticket-complete.js)
