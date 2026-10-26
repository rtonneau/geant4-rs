# Ticket 04: chemistry-semantic-identity

**Status:** ✅ Done

## Local Test Result

Ran `cargo test -p g4-chemistry --locked` with `CARGO_TARGET_DIR` in the session scratch directory. The chemistry suite passed; one existing Geant4-dependent test remains ignored.

## Review Notes

Reviewed the complete diff in `engine.rs` and `params.rs`. The engine hashes validated stored species, channels, and parameters in deterministic order; source-file identity remains separate for embedded option3. Mutation changes the semantic identity, and invalid custom values are rejected. New tests cover ordering, mutation, validation, and unchanged default RNG behavior. No changes were needed after review.

## Blockers / Challenges

None. Cargo reported nonfatal hard-link warnings while copying build artifacts into the scratch target directory.

## Commits

- c923773 feat: derive chemistry semantic identity (ticket 04)

## Time Spent

10m (ticket-start.js to ticket-complete.js)
