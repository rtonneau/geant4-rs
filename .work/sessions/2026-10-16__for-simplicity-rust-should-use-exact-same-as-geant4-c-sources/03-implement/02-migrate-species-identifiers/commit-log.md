# Ticket 02: migrate-species-identifiers

**Status:** ✅ Done

## Local Test Result

`cargo test -q -p dna-core; cargo test -q -p dna-chemistry; cargo test -q -p dna-sim` completed with exit code 0. All executed tests passed; only the repository's expected ignored tests and Windows incremental-cache hard-link fallback warnings were reported. The delegate also ran `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` successfully.

## Review Notes

Reviewed the `gpt-6.1-sol` delegate's 18-file diff. Rust variants and all call sites use chemistry casing, canonical TOML names are parsed strictly, CSV/scoring output uses `chemistry_name()`, legacy aliases remain isolated to the compatibility parser, and the old generic tag APIs and variants are absent. The direct-key C++ crosscheck change planned for Ticket 3 was necessarily brought forward so this independently committed migration remains testable; Ticket 3 retains extraction and documentation provenance work. Reference CSVs and physical values were untouched. No follow-up edits were needed.

## Blockers / Challenges

The hand-value script needs `python -X utf8` on this Windows host when its Unicode JSON is piped; its output matched the updated fixture exactly under UTF-8 mode.

## Commits

- eaf0d1c refactor: migrate canonical chemistry identifiers (ticket 02)

## Time Spent

7m (ticket-start.js to ticket-complete.js)
