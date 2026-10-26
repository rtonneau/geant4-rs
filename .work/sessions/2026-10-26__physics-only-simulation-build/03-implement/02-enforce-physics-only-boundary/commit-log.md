# Ticket 02: enforce-physics-only-boundary

**Status:** ✅ Done

## Local Test Result

The subagent ran the full S03 matrix; `evidence/ticket02-matrix.json` records each command and exit code. All exited 0: 24 checker tests, five graph roots/profiles, formatting, workspace and no-default Clippy, 399 workspace tests (19 ignored), 35 no-default simulation tests (7 ignored), standalone physics/chemistry checks, one chemistry-feature test and selected M2/M3/M6 comparisons (4/2/1 passed). Inline review reran the ticket's seven verification commands independently: `python -m unittest discover -s scripts -p test_check_crate_split.py`, `python scripts/check_crate_split.py --check-graph`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked --quiet`, `cargo test -p g4-sim --no-default-features --all-targets --locked --quiet`, and `cargo clippy -p g4-sim --no-default-features --all-targets --locked -- -D warnings`. All exited 0. Cargo output used the session scratch target directory.

## Review Notes

Reviewed reachable normal/build graph traversal, direct/transitive/disconnected fixtures, exclusive capture behavior, package-scoped CI, current guides and the recorded matrix. `git diff --check` passed. Fresh default and no-default physics captures have the same SHA-256 (`9d4540a118055be9e15ecdda0b6e28ffa8c2f93f1f61f9d2a2f5e29249d5f0ce`); default physical and physicochemical captures passed exact scientific comparison against preserved pre-S01 evidence. M2/M3/M6 logs each show the required selected tests passing. No changes were needed after review.

## Blockers / Challenges

The original ignored pre-S01 target capture was absent; the preserved immutable evidence copies were used. Linux CI has not yet run. No other blocker.

## Commits

- 56f2da7 test: enforce physics-only simulation boundary (ticket 02)

## Time Spent

15m (ticket-start.js to ticket-complete.js)
