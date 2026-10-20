# Ticket 03: instrument-rust-spatial-boundaries

**Status:** ✅ Done

## Local Test Result

`cargo test -p dna-core -p dna-physics -p dna-chemistry -p dna-sim spatial_diagnostic`

Passed: 18 focused tests; 0 failed. The 900 keV production diagnostic was discovered and intentionally ignored by the normal command.

The delegated implementation pass also ran `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`; both passed.

## Review Notes

Implemented by the resumed `gpt-6-astra` Ticket 03 subagent and reviewed inline against all acceptance criteria. The review confirmed that the five boundaries share the existing transport and physicochemical execution (no replay), physical track lineage and deterministic ordinals are retained, non-chemistry interactions are excluded, and the ignored five-seed x 40-history producer uses create-new scratch output. Independent inline verification passed without requiring code changes.

`crates/dna-sim/src/history.rs` and `crates/dna-sim/src/error.rs` are necessary companion changes: they expose the existing scored history to the observational receiver and propagate diagnostic sink failures without duplicating transport.

## Blockers / Challenges

The first delegated turn stopped at the account usage limit; the same subagent resumed from the preserved working tree without regenerating the implementation. A full workspace test run encountered three pre-existing CRLF-sensitive chemistry fixture/fingerprint failures. The chemistry TOMLs and affected tests are unchanged by this ticket; normalizing the checkout bytes from CRLF to LF reproduces the pinned expected SHA-256 exactly. This does not affect the focused Ticket 03 verification.

## Commits

- 9d24b0b feat: instrument Rust spatial boundaries (ticket 03)

## Time Spent

10h 45m (ticket-start.js to ticket-complete.js)
