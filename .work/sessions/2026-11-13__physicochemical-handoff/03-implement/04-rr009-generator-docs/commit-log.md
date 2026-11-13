# Ticket 04: rr009-generator-docs

**Status:** ✅ Done

## Local Test Result

`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`: all clean, 0 failures (new ignored consumer compiles). Generator: `cmake --build <scratch>/04-fv-build-1141 --target g4rs_ref` against the 11.4.1 install, and `<scratch>/04-fv-build` against 11.4.3 (G4_ROOT): both `Linking CXX executable g4rs_ref.exe`, exit 0. Not run (agent rule).

## Review Notes

Base 0d3cb0e, clean tree. Inline, reviewed inline.
- New mode `finite_volume <out_prefix> <seed> <sphere|box> <dims_nm> <E:events> [end_ps]`; world = finite solid; isotropic primaries; per-history CSV (edep, escaped at fWorldBoundary, primary_escaped, escaped_tracks) and per-history species CSV at 1 ps and at end of chemistry (count/outside/not_inside). Exclusive file creation. Existing modes untouched.
- RUN_REQUESTS RR-009: status awaiting run; replaces the pre-registered working command shape (fixed world per process) before any run; commands, seeds, outputs, provenance, consumer.
- README section, BUILD_NOTES section 14 (kept at 299 lines), docs/08 M12 progress.
- Consumer `g4-sim/tests/rr009_finite_volume.rs` (ignored): D1 on Geant4 rows; S1/S2 Bonferroni z=4.098 per docs/07; box endpoints NOT RUN (Rust source not isotropic).
- Geant4 S3 'shortened' is not observable; README states S3 from Geant4 is outside counts only.

## Blockers / Challenges

First compile: missing forward declaration of WriteProvenance (fixed). docs/08 contains a non-UTF-8 byte; edited byte-wise.

## Commits

- 8ca8673 feat(validation): g4rs_ref finite_volume mode for RR-009, consumer and docs (ticket 04)

## Time Spent

10m (ticket-start.js to ticket-complete.js)
