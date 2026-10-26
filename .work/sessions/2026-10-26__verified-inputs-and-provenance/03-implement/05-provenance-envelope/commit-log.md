# Ticket 05: provenance-envelope

**Status:** ✅ Done

## Local Test Result

Ran `cargo test -p g4-core --locked` with `CARGO_TARGET_DIR` in the session scratch directory. All 53 tests passed (48 unit, 3 committed-data, 2 thread-independence); doc tests passed.

## Review Notes

Reviewed the full diff. The build script captures the actual Cargo target, compiler version, Git revision and dirty state, and SHA-256 of workspace `Cargo.lock`; unavailable evidence is represented with `null`. The envelope has structured units, model and numerical policy IDs, verified/unverified input variants, and chemistry semantic/source identities. Logical data paths only are serialized. Tests cover keys, unavailable evidence, and policy/lockfile identity changes. No changes were needed after review.

## Blockers / Challenges

None.

## Commits

- 94ca1d0 feat: record reproducibility provenance (ticket 05)

## Time Spent

8m (ticket-start.js to ticket-complete.js)
