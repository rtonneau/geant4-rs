# Ticket 01: capture-pre-refactor-evidence

**Status:** ✅ Done

## Local Test Result

The gpt-6-astra/high implementer ran the ticket's full verification matrix and saved logs under `.scratch/tests/2026-10-26__crate-domain-ownership/evidence/`. Inline review reran `python -m unittest discover -s scripts -p test_check_crate_split.py` (10 passed), `cargo fmt --all -- --check` (0), `cargo clippy --workspace --all-targets --locked -- -D warnings` (0), and `cargo test --workspace --locked` (0; log `parent-workspace.log`). The named ignored selectors also passed: M2 4/4 (`parent-m2.log`), M3 2/2 (`parent-m3.log`), and M6 1/1 (`parent-m6.log`). Both independent repeat captures passed `--compare` against the baseline for physics and physchem with no difference.

## Review Notes

Reviewed the new example, checker, mutation tests and guide against ticket 01. The example records all fields of `HistoryResult`, `HistoryOutcome` and `PhysChemEvent`, plus ordered secondary records and coupled species/stats, with floating-point bits preserved before JSON serialization. The checker compares input and toolchain identities and permits only individually validated revision metadata. Baseline files live in a new ignored `target/crate-split-baseline/` directory, with copies in session evidence. The inline review made no source changes.

## Blockers / Challenges

None. The invalid-revision mutation test intentionally emits Git's “Needed a single revision” diagnostic while the test itself passes.

## Commits

- 48c9ffa test: capture pre-refactor scientific evidence (ticket 01)

## Time Spent

21m (ticket-start.js to ticket-complete.js)
