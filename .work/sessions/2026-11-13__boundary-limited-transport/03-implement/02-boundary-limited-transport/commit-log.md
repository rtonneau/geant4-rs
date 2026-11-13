# Ticket 02: boundary-limited-transport

**Status:** ✅ Done

## Local Test Result

Base 00b9855c4679f6bddf06217dd79b566caa5421c4, clean tree. `cargo test -p g4-physics --release`: all suites ok (boundary.rs 9 passed). `cargo clippy --workspace --all-targets -- -D warnings`: clean. The unbounded digest 0xa352351e0df05129 was computed by the same test body in a worktree at base 00b9855 (old API) and is identical to the new code's value.

## Review Notes

Each acceptance criterion is checked: an explicit `Option<&Volume>` argument; d_out before the draw (0 → no draw); `d_out < s` → escape at the surface, energy to `ledger.escaped`, `n_escaped` counted; `PrimaryOutsideVolume` with no draw; `Termination::Escaped` for the primary only. The debug in-flight ledger includes escapes. A point classified Outside mid-track (rounding) is treated as d_out = 0. g4-sim: `history.rs` passes None for now (ticket 03 wires it), and scoring gets an `escaped` termination count, skipped when zero so the unbounded JSON is unchanged (pulled into this ticket so the crate compiles).

## Blockers / Challenges

The worktree for the base digest hit Windows MAX_PATH; fixed with core.longpaths, and removed afterwards.

## Commits

- 8b58cf0 feat(g4-physics): boundary-limited transport with escaped energy (ticket 02)

## Time Spent

6m (ticket-start.js to ticket-complete.js)
