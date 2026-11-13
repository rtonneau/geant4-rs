# Ticket 03: finite-volume-spec

**Status:** ✅ Done

## Local Test Result

`cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`: exit 0; 71 suites, 577 passed, 0 failed, 27 ignored (output in .scratch/tests/.../03-cargo.txt; the working tree also contained the other session's uncommitted Rust code).

## Review Notes

Base 0ac6fa0. ADR 0005 (.work/adr, the project's ADR location, not docs/adr as the plan said) records the Volume enum. docs/09 has G-B1/G-B2 plus risk R-B1. docs/07 holds fixed criteria D1-D4 (exact) and S1/S2 (24 endpoints, Bonferroni alpha 0.001, +-0.01 band, S3 diagnostic only). RUN_REQUESTS logs RR-009, awaiting generator (RR-006..008 are taken by the uncommitted work). docs/08 has M12, and M11 is updated. docs/07 and RUN_REQUESTS.md carry the other session's uncommitted hunks, so the index was built as HEAD + this ticket's appended text (hash-object/update-index) and the commit holds only this ticket's text. Self-review fixed the endpoint count (36 → 24). Before committing, I also found that RR-005 data is 11.4.3 and corrected the ticket 01 text in follow-up commit 0ac6fa0.

## Blockers / Challenges

Pre-modified shared files (see Review Notes). When the other session commits, its edits to the ends of docs/07 and RUN_REQUESTS.md will need a trivial merge with these appended sections.

## Commits

- 48bbb97 docs: specify finite water volume, boundary acceptance and RR-009 (ticket 03)

## Time Spent

4m (ticket-start.js to ticket-complete.js)
