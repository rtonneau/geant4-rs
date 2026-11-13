# Ticket 01: scheduler-diagnostic

**Status:** In Progress

## Local Test Result

`python validation/geant4/rr008_seed_audit.py` reports 12288 unique prospective pairs and zero overlap with 289 distinct historical pairs. The new executable compiled and linked against Geant4 11.4.1 using MSVC and existing Qt interface stubs. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` exited 0 (some reference tests ignored). Boundary spot checks: 258 inconclusive, 259 and 438 pass, 439 inconclusive.

## Review Notes

Base 490b6e8579ac36dade58297ad47f14a344312bfd. Existing modified and untracked project files predate this ticket; ticket work includes rr008_seed_audit.py, rr008_followup_consumer.py, SynSchedulerFollowup.cc and additions to CMakeLists.txt. Independent review found missing provenance, exit-status and smoke validation; incorrect coverage precedence and complete-data handling; and producer exceptions suppressing structural evidence. Corrections started, but provenance and smoke remain unimplemented and a fresh review is required. Consumer deliberately cannot declare PASS until evidence validation is implemented.

## Blockers / Challenges

The 11.4.1 install was found under DEV_DIR and compile-checked. Consumer evidence validation, tests, documentation, exact commands and request registration remain incomplete. No generator was run.

Correction pass and fresh independent review: consumer now checks declared provenance, hashes, manifest contents and smoke/production records; three synthetic tests pass. Reviewer found remaining gaps: malformed row with a footer is classified FAIL rather than INCONCLUSIVE for incomplete schema; no executable historical Ranecu reconstruction or independently reproducible bound artifact; evidence and overall verdict lack end-to-end tests. Reviewer also flagged zero coverage as frequency FAIL; the approved contract separately says missing setting coverage is INCONCLUSIVE, so its precedence must be resolved without changing either criterion. Documentation, resource-enforcing commands and request registration are not finished. No generator was run. This was the one correction pass; remaining findings are unresolved.

**Blocked (2026-11-09T11:57:34.346Z):** Independent re-review found unresolved evidence validation and reconstruction gaps after the permitted correction pass; request is not ready.

## Commits

Filled by ticket-complete.js.

## Time Spent

Filled by ticket-complete.js.
