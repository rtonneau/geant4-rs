# Ticket 04: document-option3-reactions

**Status:** ✅ Done

## Local Test Result

Ran the ticket's exact sequential gate:

`cargo fmt --all -- --check; if ($LASTEXITCODE -eq 0) { cargo clippy --workspace --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test --workspace }`

Final exit code: 0. Formatting and Clippy passed; every non-ignored workspace test and all doc-tests passed.

## Review Notes

- Documented the fourth embedded chemistry file, schema, fixed hash concatenation order and Geant4 11.4.1 release/path provenance.
- Recorded all 72 entries, class/type semantics, four bulk participants, five comment/literal discrepancies and the exact reaction-53 anomaly.
- Kept the 1 ps physicochemical boundary explicit and deferred derived radii, probabilities, diffusion and scheduling to later M7 work.
- Updated the M7 roadmap status and reproducibility fingerprint.
- The full all-targets gate found two Clippy diagnostics in the loader and two ill-typed JSON-array assertions in the cross-check; fixed them without changing reaction behavior.

## Blockers / Challenges

A sync-generated untracked `cpp_crosscheck (# Edit conflict ...).rs` duplicate was auto-discovered as an invalid Rust test target. Its useful typed-array comparison was applied to the canonical test, then the exact duplicate path was validated as inside this worktree and removed.

## Commits

- 2553983 docs: document option3 reaction table (ticket 04)

## Time Spent

6m (ticket-start.js to ticket-complete.js)
