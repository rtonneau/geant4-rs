# Ticket 03: g4-sim-geometry-wiring

**Status:** ✅ Done

## Local Test Result

Base c4e2a90. `cargo fmt --all -- --check`: ok. `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo test --workspace --release`: all suites ok, 0 failed (release used for runtime; same code). determinism.rs: the new bounded 1/2/8-thread test passes.

## Review Notes

validate() now requires the volume to contain the origin (the primary start) instead of rejecting geometry. history.rs passes `config.geometry.as_ref()`. The summary adds `energy.escaped_ev` (Option, omitted without escapes) next to `terminations.escaped`, so unbounded JSON is unchanged. The bits() digest includes n_escaped. docs/04 D.5 and D.6 are updated; chemistry ignores the boundary (documented).

## Blockers / Challenges

None

## Commits

- 1513e88 feat(g4-sim): run finite-volume transport and score escapes (ticket 03)

## Time Spent

4m (ticket-start.js to ticket-complete.js)
