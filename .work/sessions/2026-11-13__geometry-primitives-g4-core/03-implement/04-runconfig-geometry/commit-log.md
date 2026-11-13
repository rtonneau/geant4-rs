# Ticket 04: runconfig-geometry

**Status:** ✅ Done

## Local Test Result

Ran `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` (preceded by `cargo fmt --all`, which changed nothing outside config.rs): exit 0. Output: .scratch/tests/2026-11-13__geometry-primitives-g4-core/evidence/04-verify.txt. New tests in config::tests (4) pass.

## Review Notes

Base commit: e3f8db53d37aed8b46097af49b4d55f8fbc4032c.
- `RunConfig.geometry: Option<Volume>`, default None in `new()`. Output unchanged: `#[serde(default, skip_serializing_if = "Option::is_none")]`; RunOutput serialises `config`, so a test checks JSON and TOML omit the key for None.
- `validate()` returns InvalidConfig for `Some(_)` (test). Transport does not read the field.
- RunConfig was Serialize only; added Deserialize (all fields plain types) for the round-trip. Tests: box/tubs/sphere TOML and JSON round-trips; invalid box half-length, unknown shape and rmin>rmax rejected at deserialisation.
- docs/04 D.5: added a module overview paragraph before D.5.1 (units, solids, centre/no rotation, Inside 3-state with tolerances, distance_to_out None for Outside, sources, limitations).

## Blockers / Challenges

None

## Commits

- a903f27 feat(g4-sim): optional RunConfig geometry, rejected until transport honours it (ticket 04)

## Time Spent

5m (ticket-start.js to ticket-complete.js)

## Parent acceptance notes

- No independent reviewer: the change is localized and low-risk (one optional field that validate() rejects, plus tests and docs), so the parent reviewed the diff directly.
- One parent fix: the validate() message had a run of embedded spaces from a broken line continuation; collapsed to one space.
- Parent re-verification: fmt --check clean; clippy --workspace --all-targets -D warnings clean; cargo test --workspace 679 passed, 0 failed.
- Accepted: all criteria met; `None` is skipped when serialising, so existing outputs are unchanged.
