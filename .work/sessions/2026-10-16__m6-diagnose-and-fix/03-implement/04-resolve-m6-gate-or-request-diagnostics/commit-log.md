# Ticket 04: resolve-m6-gate-or-request-diagnostics

**Status:** ✅ Done

## Local Test Result

`cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` exited 0: 268 passed, 13 ignored, no lint warnings. `CARGO_TARGET_DIR` and every log/build artifact are under `.scratch/tests/2026-10-16__m6-diagnose-and-fix/`. Full output: `ticket04-workspace.log`.

The new C++ mode compiled and linked with installed Geant4 11.4.1, MSVC 19.50.35729, Ninja and RelWithDebInfo. The initial sandbox build compiled the object but could not execute the vcpkg PowerShell post-link dependency copy; the approved host build completed the link (exit 0, `ticket04-geant4-build-retry.log`). No Geant4 executable was run.

Separately ran the exact ignored tests in `physchem_reference`:

- `m6_initial_species_counts_and_rms_match_geant4`: exit 101, unchanged `1000 H2O2: z=111.55722825815035` (6.64 s, `ticket04-original-gate.log`).
- `m6_prechem_counts_and_rms_match_geant4`: exit 101, expected `RR-001 awaiting data: physchem_prechem.csv` (0.06 s, `ticket04-awaiting-data.log`). It was also run before generator implementation and failed for the missing new data. No data are fabricated to make this future gate pass.
- `m6_reference_contains_products_requiring_zero_time_chemistry`: exit 0, 1 passed (`ticket04-semantics.log`).

`git diff --check` passed; `git diff --exit-code -- validation/reference` passed (historical CSVs/provenance untouched). `BUILD_NOTES.md` remains below 300 lines.

## Review Notes

Read the ticket, preceding 20 commits, named source/docs, and earlier audit evidence. Rechecked the read-only Geant4 11.4.1 scheduler, model processor, IRT stepper, step processor, user callback, recombination and solvation code. No additional Rust physics correction was established. The final decision records an evidence-backed stage mismatch and unchanged z <= 4.5, 15% non-small-count and 20% RMS criteria; M6 remains awaiting data and gates dependent M7 work.

Added a classified M6 difference register in docs/09 with source paths and observed effects: snapshot mismatch; unresolved ordering/candidate caching and residual shared-species bias; intentional RNG/transport sequence, infinite-medium geometry, killed-track representation, scoring separation and domain policies. Retained shared Q12/Q13/Q17 settings are explicitly distinguished from differences.

Added only the separately named `physchem_prechem` mode. It uses the unchanged option2/option3 processes, reaction table and IRT selection. It captures in the post-zero-step callback when no H2O state remains, after daughters merge and killed tracks are removed, before the next time selection initializes IRT. Guards fail on chemical reactions, positive-time advancement, missing snapshots, wrong track status/time and incomplete event counts. Exclusive output creation prevents overwriting evidence. Existing mode arguments, physics settings and output formats remain unchanged.

Added the ignored RR-001 consumer with strict provenance, complete four-energy statistics, seven-species/28-row uniqueness and count-moment checks. Factored the original assertions into a shared helper without changing their logic or tolerances. Documentation records new filenames, schema, exact command, pre-registered criteria, build/run instructions and runtime-unverified status. RUN_REQUESTS.md logs RR-001 as awaiting run. No new reference data were created, replaced or deleted. Review corrected a documentation source path and strengthened required provenance fields to nonempty strings plus an explicit dirty-state boolean.

## Blockers / Challenges

No implementation blocker. M6 intentionally awaits the project lead's RR-001 run; this is the ticket's required terminal outcome because the committed aggregate post-reaction snapshot cannot recover pre-reaction populations or positions. Runtime behavior of the compile-checked new hook remains unverified until that run; guards must not be removed to accommodate a failure. The initial sandbox post-link execution issue was resolved through the approved host build. Unrelated pre-existing `.gitignore` and `.VSCodeCounter/` work is preserved and excluded from this ticket.

## Commits

- c6bea6e test: add pre-IRT M6 reference diagnostic (ticket 04)

## Time Spent

8h 44m (ticket-start.js to ticket-complete.js)
