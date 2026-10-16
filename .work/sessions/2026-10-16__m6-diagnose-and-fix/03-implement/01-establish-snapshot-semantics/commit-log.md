# Ticket 01: establish-snapshot-semantics

**Status:** ✅ Done

## Local Test Result

`cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture` was run before and after the change, with `CARGO_TARGET_DIR=.scratch/tests/2026-10-16__m6-diagnose-and-fix/cargo`.

Final result: **2 passed, 1 failed**, exit 101, in 47.12 s. The non-asserting `m6_initial_species_comparison_report` completed all four energies and 60 rows; the new `m6_reference_contains_products_requiring_zero_time_chemistry` diagnostic passed. The unchanged `m6_initial_species_counts_and_rms_match_geant4` failed at `1000 H2O2: z=111.55722825815035`, exactly the baseline failure. This is the explicitly permitted outcome in this ticket's Verification Step, not a closed M6 gate.

Required repository checks: `cargo fmt --all -- --check` passed; `cargo clippy --workspace --all-targets -- -D warnings` passed; `cargo test --workspace` passed (**264 passed, 0 failed, 11 ignored**). `git diff --check` passed for the three ticket files. Build and run logs remain in the session scratch directory (`ticket01-baseline.log`, `ticket01-verification.log`, `ticket01-workspace.log`).

## Review Notes

Read Geant4 11.4.1 scheduler, IT model processor, molecular dissociation, option1 dissociation channels and option3 reaction table directly, plus the IRT sampler/encounter stepper needed to establish zero-time contact reactions. The first positive-step pre-callback follows preceding zero-time collisions, so the committed reference is after zero-time chemical reactions at 1 ps. The appearance of reaction-only products supports that interpretation; the source call order establishes it independently.

The diagnostic checks the complete seven-species dissociation-product union, including H2Ovib after electron-hole recombination, and distinguishes it from reference-only H2O2, HO2°, HO2m, Om, O2, O2m and O3. It also requires the four principal reaction-only products at every reference energy.

`docs/08-roadmap.md` records exact source paths/line numbers and all 60 mean/SE/z/relative-difference/RMS rows; each table row was checked against the final rerun. `docs/09-open-questions-and-risks.md` records Q25/R17 and corrects the interpretation of the already-closed lifecycle issue. All four acceptance criteria are satisfied. The original gate body was compared against HEAD and is unchanged after newline normalization; no tolerance, reference/provenance file, generator mode or simulation behavior changed. The mismatch does not establish that every remaining count difference has the same cause; M6 stays open.

## Blockers / Challenges

None. The known scientific gate failure is retained as required evidence. Git's ownership check was handled with the existing per-command safe-directory convention; unrelated `.gitignore` and `.VSCodeCounter` changes were preserved.

## Commits

- 8a281fd test: establish M6 snapshot semantics (ticket 01)

## Time Spent

9m (ticket-start.js to ticket-complete.js)
