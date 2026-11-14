# Ticket 03: rr009-box-endpoints

**Status:** ✅ Done

## Local Test Result

`cargo test --release -p g4-sim --test rr009_finite_volume -- --ignored --nocapture`: 1 passed in 45 s; 24 endpoints evaluated, box 6/6 PASS (1 keV S1 -0.000050+-0.000047, S2 -0.000200+-0.000141; 10 keV S1 0.001567+-0.001299, S2 -0.000400+-0.000424; 100 keV S1 0.000040+-0.000033, S2 0+-0); sphere values identical to the recorded 2026-11-13 table; open endpoints 0. Log `.scratch/tests/2026-11-14__m12-isotropic-source-box-endpoints/03-rr009.log`. fmt/clippy clean.

## Review Notes

Base 217e5dcacc1cf35d7f25570a084c85d5ea56931c. Inline. Box = G4Box half-lengths (1000,1000,100) nm per g4rs_ref.cc:829 and provenance_rr009.toml command line; Volume::box_ takes half-lengths. Z_FAMILY, MARGIN, ACCEPTED_INCONCLUSIVE, seeds unchanged. Verdict recorded in docs/07, docs/08, gates.json. docs/gps-backlog-m12-geometry.md is untracked pre-existing user work, left untouched.

## Blockers / Challenges

docs/07 and docs/08 are CRLF and docs/08 holds a non-UTF-8 byte; edited byte-preserving.

## Commits

- 7d8c69c validation(rr009): run box endpoints with isotropic source, gate closed (ticket 03)

## Time Spent

3m (ticket-start.js to ticket-complete.js)
