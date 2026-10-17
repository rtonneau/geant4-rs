# Ticket 02: audit-and-correct-physchem-conversion

**Status:** ✅ Done

## Local Test Result

Passed both verification commands: `cargo test -p dna-chemistry` (42 passed) and `cargo test -p dna-sim --test physchem_reference m6_initial_species_comparison_report -- --ignored --nocapture` (1 passed; all 60 rows for 1/10/100/900 keV, unchanged values; 33.42 s). Also passed `m6_recombination_ordering_ablation -- --ignored --nocapture` (1 passed, six modes at all four energies; 208.19 s), `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` (266 passed, 12 ignored). Cargo target and saved command outputs are under `.scratch/tests/2026-10-16__m6-diagnose-and-fix/` (`ticket02-chemistry.log`, `ticket02-comparison.log`, `ticket02-ordering.log`, `ticket02-workspace.log`). The original ignored agreement gate is unchanged; the successful report still shows its known discrepancies, including 1 keV H2O2 |z| = 111.557.

## Review Notes

- Read recent commits and every named implementation/test/documentation file; re-read Geant4 11.4.1 `G4ChemDissociationChannels_option1::ConstructDissociationChannels`, chemistry-manager hand-off, molecular-dissociation process, configuration lifetime initialization, water definition, and electron-hole recombination/IT step selection. All 23 channels match: physical state/vacancy, exact probability, ordered products, zero decay delay, released energy, and displacement case. The exhaustive audit and source paths are in docs/02 B.7.8; no physical source values or default conversion behavior changed.
- Added a fixture-backed test that executes every channel through event dispatch and checks products/energy/1 ps timing; existing extraction and chi-square cross-checks remain intact. Added an independently hand-constructed competing-hole case (one versus two recombinations from reversed water order). Forward and reversed histories are bitwise reproducible on 1/2/8 threads.
- Added the full-cascade ordering diagnostic in `physchem_reference.rs` with event/reverse/shuffle/excitation-first/ionisation-first/off modes, fixed electron insertion order, separate permutation seed, and restarted history-local PhysChem streams. Quantified recombination fractions, paired differences with SE, and all seven alive-species means. At 10 keV, the five enabled orders span 20.6775-21.7900%; off raises mean e_aq from 400.0000 to 502.3895. Complete results and rerun method are in docs/04 D.2.1 and linked from Q21/R14.
- No conversion defect was confirmed, so the ticket's source-backed-audit alternative applies; no production fix or failing-then-passing correction was needed. Corrected the Q20 documentation assertion that H2Ovib relaxation deposits excitation energy: it actually retains zero released energy, which Rust already implements correctly. Documented that Rust's immediate nearest-alive search is not proven equivalent to the C++ GPIL-cached-candidate scheduler; reversing events is a sensitivity experiment, not a scheduler port.
- Reran comparison unchanged. Preserved separation of physical, physicochemical and chemical stages, existing reference files/modes, tolerances, and open M6 gate. No Geant4 generator run, C++ reference write, new dependency or M7 reaction was introduced. Diff reviewed for scope and whitespace.

## Blockers / Challenges

No task blocker. Git required a command-scoped safe.directory setting because the sandbox account differs from the repository owner. Workspace compilation printed two non-fatal Windows incremental-cache finalization notes (Access is denied); cargo completed with exit 0 and all 266 tests passed. The scientific scheduler-equivalence question remains open as explicitly documented in Q21/R14; it does not prevent the source audit or ordering measurements.

## Commits

- 859030c test: audit M6 conversion and recombination ordering (ticket 02)

## Time Spent

13m (ticket-start.js to ticket-complete.js)
