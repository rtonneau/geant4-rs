# Ticket 01: source-stream-isotropic-sampler

**Status:** ✅ Done

## Local Test Result

`cargo test -p g4-core`: 177 + 3 + 2 passed, 0 failed (log `.scratch/tests/2026-11-14__m12-isotropic-source-box-endpoints/01-test.log`). `cargo clippy --workspace --all-targets -D warnings` clean.

## Review Notes

Base fc7218089b08250125ac95a5dae7d186deda5f0b; pre-existing untracked: scout report and docs/gps-backlog-m12-geometry.md (not ticket-owned). Inline, no reviewer (localized, low-risk). Stage::Source = 3 with slot test against raw ChaCha stream (id<<2)|3; disjointness test extended to Source. isotropic_direction in g4_core::sampling: two uniforms (ScriptedRng test), KS on cos theta and phi at 1.95/sqrt(N), unit norm 1e-12. Geant4 11.4.3 G4RandomDirection.hh read: Marsaglia rejection; distribution, not sequence, reproduced (rustdoc).

## Blockers / Challenges

None.

## Commits

- 8526244 feat(g4-core): Source RNG stage and isotropic direction sampler (ticket 01)

## Time Spent

1m (ticket-start.js to ticket-complete.js)
