# Ticket 01: optional-chemistry-simulation

**Status:** ✅ Done

## Local Test Result

Independent inline review reran all three ticket commands with `CARGO_TARGET_DIR=.scratch/tests/2026-10-26__physics-only-simulation-build/cargo-target`: `cargo test -p g4-sim --no-default-features --all-targets --locked`, `cargo test -p g4-sim --test chemistry_feature --locked`, and `cargo test --workspace --locked --quiet`. All exited 0. The no-default run selected 35 physical tests and 7 intentionally ignored cases; `physics_only` passed. `chemistry_feature` passed (1 test). The workspace suite exited 0; the subagent's complete log reports 399 passed and 19 ignored. The subagent also ran format, workspace Clippy, no-default Clippy, and exact default/pre-S01 and default/no-default physics snapshot comparisons; all passed. Logs and captures are in `.scratch/tests/2026-10-26__physics-only-simulation-build/evidence/`.

## Review Notes

Reviewed the manifest feature and required-feature targets, public `run` re-exports, cfg-gated modules/errors, physical and chemistry consumer tests, and snapshot example. The coupled implementation was moved to a private child module without changes to its algorithm. `git diff --check` passed. `cargo tree -p g4-sim --no-default-features --edges normal,build --locked` showed no `g4-chemistry` entry. The default physics and physchem snapshots matched the immutable pre-S01 evidence copies, and the default/no-default physics snapshots matched exactly. No changes were needed after review.

## Blockers / Challenges

The original ignored `target/crate-split-baseline` capture was absent. The preserved immutable copies under `.scratch/tests/2026-10-26__crate-domain-ownership/evidence/` were used for the same exact comparisons; the initial missing-path diagnostic and successful comparison logs are retained. No other blocker.

## Commits

- 5387056 feat: make g4-sim chemistry optional (ticket 01)

## Time Spent

11m (ticket-start.js to ticket-complete.js)
