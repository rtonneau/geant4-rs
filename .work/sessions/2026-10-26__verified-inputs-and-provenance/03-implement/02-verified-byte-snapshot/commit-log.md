# Ticket 02: verified-byte-snapshot

**Status:** ✅ Done

## Local Test Result

I reran `cargo test -p g4-core -p g4-data -p g4-physics --locked` with `CARGO_TARGET_DIR` in the session scratch directory. It exited 0: 210 tests passed, one pre-existing long test was ignored, and doc tests passed. The implementer also ran formatting, workspace Clippy and workspace tests successfully; logs are retained under the session scratch `evidence/` directory.

## Review Notes

The ticket was implemented by `gpt-6-astra` at xhigh effort. I reviewed the snapshot and model constructors: the manifest reads each listed file once, checks the captured bytes and parses catalogued tables from them. `VerifiedDataSet` and `VerifiedProcessTable` own read-only data and identity without a source root. Tests reject missing, corrupt, wrong-manifest and unlisted inputs; a frozen snapshot survives later file changes; relocated inputs retain identity. Verified and unverified option2 models match cross sections, final states and RNG draw counts. I made no follow-up code changes.

## Blockers / Challenges

Cargo reported that the scratch filesystem could not hard-link incremental cache files and copied them instead; the test run completed successfully.

## Commits

- 58b80e3 feat: bind option2 models to verified data snapshot (ticket 02)

## Time Spent

14m (ticket-start.js to ticket-complete.js)
