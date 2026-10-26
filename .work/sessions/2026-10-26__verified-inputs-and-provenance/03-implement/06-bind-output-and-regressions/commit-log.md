# Ticket 06: bind-output-and-regressions

**Status:** ✅ Done

## Local Test Result

Ran `cargo fmt --all -- --check`, locked workspace Clippy with `-D warnings`, locked workspace tests, and locked physics-only check and tests; all exited 0. Ran `python scripts/run_validation_gates.py --tier required` (7-gate evidence preflight passed), then the statistical tier through the scratch-only target-directory wrapper; all four M2, two M3, and one M6 gates passed. Agent logs and baseline comparison are retained under the session scratch `evidence/` directory. The parent reran each required command and all seven statistical gates successfully.

## Review Notes

Reviewed the full source and documentation diff. Production output APIs now require `VerifiedProcessTable` and derive provenance from its frozen identity; provenance is read-only on output. Explicit synthetic test APIs emit an unverified schema and no manifest claim. Coupled output records the supplied engine's semantic hash and optional embedded source hash. New regressions cover relocation, changed bytes, frozen output, default score equivalence and custom chemistry. A scratch comparison against pre-R02 revision `baed05953d3cdf3612bd29025c63d5469993612f` matched 24 detailed histories, summaries, events, species, CSV, and physical/physicochemical RNG counts and next values exactly. Dependency graph inspection and the physics-only profile passed. No source changes were needed after review.

## Blockers / Challenges

The gate runner's `--tier required` checks evidence only; the seven scientific comparisons require `--tier statistical`. Its statistical target directory is hardcoded to an earlier session, so the agent used a scratch-only wrapper to redirect builds. My rerun created a Python cache file in `scripts/__pycache__`; I removed that file before committing. Cargo's incremental-cache hard-link warnings were nonfatal.

## Commits

- eacf9fd feat: bind verified inputs to simulation output (ticket 06)

## Time Spent

26m (ticket-start.js to ticket-complete.js)
