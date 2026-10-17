# Ticket 01: audit-relations-harness

**Status:** ✅ Done

## Local Test Result

`cargo test -p dna-sim --test physchem_audit -- --ignored --nocapture`: exit 0, 3 passed, 165.20 s (run by the subagent and again here; log .scratch/tests/2026-10-17__m6-residual-rust-audit/01-verify.log). R-Q and R-H have integer residual 0 on Geant4 at all four energies and on all 5 Rust seeds; per-channel and run-level budgets pass. Species means still differ as in RR-001 (e.g. 1 keV H3O+ z +27.7, H z -16.1).

## Review Notes

Subagent (sonnet, high) wrote crates/dna-sim/tests/physchem_audit.rs only; no production code or reference file touched. Checked: header fixes relations and 4.5 sigma before results; Geant4 limitation printed; 5 seeds at every energy (900 keV took 78 s, no 2-seed fallback); tables printed. Reran the verification myself. Finding for ticket 3: conservation relations hold on both sides, so by the plan bar they cannot name a stage family; R-W shows Rust O atoms per event 0.9% above Geant4 at 1 keV (z +21), a hint toward the partition. Rust-side relation passes are close to identities of the channel tables, so they are weak evidence.

## Blockers / Challenges

None

## Commits

- b30beb5 test: add M6 residual audit harness with conservation relations and 5-seed Rust rerun (ticket 01)

## Time Spent

15m (ticket-start.js to ticket-complete.js)
