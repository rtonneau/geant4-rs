# Ticket 02: runconfig-isotropic-direction

**Status:** ✅ Done

## Local Test Result

fmt --check clean; clippy --workspace --all-targets -D warnings clean; `cargo test --workspace`: 76 test binaries ok, 0 failed (log `.scratch/tests/2026-11-14__m12-isotropic-source-box-endpoints/02-test.log`). Existing determinism tests pass unmodified.

## Review Notes

Base 2f3bcf436727b2cd135e78d9b4db9ea78359480f. Inline, no independent reviewer: changes RunConfig (public, persisted config) but serialisation of existing configs is pinned by tests; parent inspected the diff. PrimaryDirection via try_from/into repr (array | "isotropic"); unknown keyword rejected. primary() now takes RngStreams and is fallible; only Source stream for isotropic. validate rejects zero/non-finite fixed direction. Spatial diagnostics keep +z guard. Run schema not bumped (fixed output unchanged); documented in docs/reproducibility.md.

## Blockers / Challenges

None.

## Commits

- 3ed1e72 feat(g4-sim): isotropic primary direction option in RunConfig (ticket 02)

## Time Spent

4m (ticket-start.js to ticket-complete.js)
