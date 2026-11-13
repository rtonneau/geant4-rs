# Ticket 01: volume-config-provenance-schema

**Status:** ✅ Done

## Local Test Result

`cargo test --workspace`: 67 test binaries, all `test result: ok`, 0 failures (log in .scratch/tests/.../01-test.log).

## Review Notes

Base e4388921cde34603ba30d4f50834deb1be8bec62, clean tree. Inline. `geometry` renamed to `volume` in the config, history and tests. Added `#[serde(deny_unknown_fields)]` on RunConfig: without it the legacy `[geometry]` key was silently ignored (infinite water), which test `legacy_geometry_key_is_rejected` caught. Provenance gets `volume` + `with_volume`, applied in output_with_sink/physchem_output. The schema strings are bumped in src and tests.

## Blockers / Challenges

The grill assumed deny_unknown_fields was already on; it wasn't, so I added it (needed to meet the 'rejected' criterion).

## Commits

- e20563c feat(g4-sim): RunConfig.volume, volume in provenance, schema run/v5 (ticket 01)

## Time Spent

5m (ticket-start.js to ticket-complete.js)
