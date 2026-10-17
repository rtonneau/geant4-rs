# Ticket 03: verdict-and-source-proven-fix

**Status:** ✅ Done

## Local Test Result

`cargo test -p dna-sim --test physchem_reference m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact --nocapture`: exit 0, 1 passed, 35.46 s (was 16 failing checks), criteria unchanged. Rerun by the supervisor: cargo fmt --check exit 0, clippy -D warnings exit 0, cargo test --workspace exit 0 with 270 passed, 16 ignored. Logs in .scratch/tests/2026-10-17__m6-residual-rust-audit/ (03-*.log).

## Review Notes

Subagent (opus, xhigh). Verdict: electron-hole recombination scheduling (D7 auto-ionisation e_aq not a candidate in the creating pass, D9 retest of failed ions in the next zero-time pass, D10 Geant4 processing order), traced to Geant4 .cc lines and confirmed quantitatively on 6 seeds with a scratch prototype (failing checks 11-16 -> 0, RMS z 7 -> about 1, 1 keV recombinations +0.70 per event vs 0.72 fitted). Only production change: PhysChemEngine::convert_history in crates/dna-chemistry/src/engine.rs. Re-baselined crates/dna-chemistry/tests/history.rs (3 tests adapted, 2 added; the behavioural ones fail on the old engine). docs/reproducibility.md updated for the changed draw order. validation/reference and physchem_reference.rs untouched. Caveats for ticket 4: one Geant4 seed, no covariance, and assumed match of Rust event order with Geant4 tracking order; docs/09 Q21/R14, docs/04 D.2.1 and the RR-001 Rust means are now stale.

## Blockers / Challenges

None

## Commits

- 5df0aa1 fix: schedule electron-hole recombination in Geant4 zero-time passes (ticket 03)

## Time Spent

30m (ticket-start.js to ticket-complete.js)
