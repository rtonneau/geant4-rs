# Ticket 01: validate-rename-snapshots

**Status:** ✅ Done

## Local Test Result

`python -m unittest discover -s scripts -p test_check_crate_split.py`: 18 tests passed.
`python scripts/check_crate_split.py --compare .scratch/tests/2026-10-26__crate-domain-ownership/evidence/baseline-physics.json .scratch/tests/2026-10-26__crate-domain-ownership/evidence/ticket-02-physics.json`: passed with exact scientific, input and toolchain identity.
`git diff --check`: exited 0. The delegate also reported successful formatting, workspace Clippy and workspace tests.

## Review Notes

Reviewed both script diffs. Strict comparison still permits only a validated revision change. Rename mode requires the exact `dna-sim` to `g4-sim` pair, verifies each captured lock hash against the committed LF bytes or the exact Windows CRLF checkout form, and compares parsed lockfiles after only the five local package-name substitutions. Scientific payloads, data hashes and toolchain remain exact. Mutation tests cover third-party versions/sources/checksums, local versions/dependencies, package names, scientific fields and metadata. Inline review removed dependency-list sorting so an unrelated reordering is rejected, and added a regression case for it.

## Blockers / Challenges

The archived Windows snapshot hashed a CRLF checkout of `Cargo.lock` while Git stores LF bytes; both exact forms are checked. No remaining blocker.

## Commits

- bf4946c test: validate crate rename snapshots (ticket 01)

## Time Spent

16m (ticket-start.js to ticket-complete.js)
