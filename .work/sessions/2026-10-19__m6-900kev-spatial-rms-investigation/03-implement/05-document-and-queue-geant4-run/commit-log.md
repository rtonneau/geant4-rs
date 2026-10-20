# Ticket 05: document-and-queue-geant4-run

**Status:** ✅ Done

## Local Test Result

`cargo test -p dna-sim --test physchem_reference m6_900kev_spatial_diagnostic -- --ignored --exact --nocapture` compiled the consumer and exited 101 as preregistered with: `RR-002 awaiting data: spatial_geant4_900kev_40.csv and provenance_m6_spatial_900kev_40.toml; project lead must run Geant4 and commit both complete new files`.

`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` all passed. The workspace run includes 21 dna-chemistry unit tests and 18 spatial-diagnostic tests; expected slow/reference tests remain ignored.

## Review Notes

Documented the exact RR-002 mode, five-boundary schema, identity/moment fields, 18 radial bins, runtime/completeness checks, provenance fields, 20,000-replicate hierarchical bootstrap, primary/simultaneous reporting, and precision-only 40/80/160 rule. `BUILD_NOTES.md` remains below 300 lines (204). Added RR-002 without changing RR-001 and recorded the awaiting-data/non-closure state in both M6 documents.

The ignored consumer validates provenance and embedded completion/stage comments, strictly parses all seed/history/category blocks, generates the independent Rust sample only after committed Geant4 evidence exists, and reports the primary interval, simultaneous boundary intervals, first detected divergence, precision decision, and secondary boundary/category summaries.

To satisfy the required workspace gate on Windows, made the option3 fingerprint canonical over LF and normalized CRLF in two fixture-edit helpers. This preserves the pinned hash and every chemistry value while removing checkout-line-ending dependence.

## Blockers / Challenges

The first workspace run reproduced three pre-existing CRLF-sensitive dna-chemistry failures. The portable normalization described above resolved all three. The Geant4 data/provenance are intentionally absent and remain a project-lead run request; no generator or reference-data write was performed.

## Commits

- 5420d04 docs: queue M6 spatial diagnostic run (ticket 05)

## Time Spent

6m (ticket-start.js to ticket-complete.js)
