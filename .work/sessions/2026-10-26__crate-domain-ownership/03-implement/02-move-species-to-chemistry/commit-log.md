# Ticket 02: move-species-to-chemistry

**Status:** ✅ Done

## Local Test Result

The gpt-6-astra/high implementer captured the required red interface compile failure (`E0432`), then ran the green interface (3 passed), format, Clippy, workspace suite (397 passed, 19 intentionally ignored), M2 (4 passed), M3 (2 passed), M6 (1 passed), dependency trees and both exact snapshot comparisons. Logs and JSON copies are under `.scratch/tests/2026-10-26__crate-domain-ownership/evidence/ticket-02-*`.

Inline review reran `cargo test -p dna-chemistry --test species_interface --locked` (3 passed), `cargo fmt --all -- --check` (0), `cargo clippy --workspace --all-targets --locked -- -D warnings` (0), and `cargo test --workspace --locked` (0; `parent-02-workspace.log`). The three named ignored selectors passed again: M2 4/4 (`parent-02-m2.log`), M3 2/2 (`parent-02-m3.log`), M6 1/1 (`parent-02-m6.log`). Both profiles passed `scripts/check_crate_split.py --compare` against ticket 01 with exact scientific, input and toolchain identity. `git diff --check` exited 0.

## Review Notes

Reviewed the complete species implementation move, root exports, finite consumer updates, interface tests, dependency trees and exact captures. No active Rust source/test/example reference to `dna_core::species` remains. Core has no chemistry edge, and physics and chemistry do not depend on each other. The implementer changed only imports and module prose in the moved implementation. Inline review corrected the current `README.md` layout bullets so they place species under chemistry; older milestone narratives remain historical.

## Blockers / Challenges

None. Cargo reported filesystem hard-link cache fallbacks during several builds; every required command exited 0.

## Commits

- 6c1ea52 refactor: move species ownership to chemistry (ticket 02)

## Time Spent

17m (ticket-start.js to ticket-complete.js)
