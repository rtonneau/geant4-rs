# Ticket 02: source-channel-comparison

**Status:** ✅ Done

## Local Test Result

`git diff --stat`: only docs/08-roadmap.md changed by this ticket (+73 lines, no deletions); 38 table rows (A1-A8, B1-B10, C1-C8, D1-D10, N1-N2) each cite a Rust and a Geant4 file:line (script check in .scratch/tests/2026-10-17__m6-residual-rust-audit/02-verify.log). No cargo run: docs only.

## Review Notes

Subagent (opus, high) wrote the draft section; Geant4 checkout read-only. Spot-checked D7 against crates/dna-chemistry/src/engine.rs:331-410: auto-ionisation e_aq from earlier events are inserted into the grid, as the table states. D9 is inferred from source, not traced at run time (stated in the table). Verdict material for ticket 03: A, B, C identical; recombination D7 (differs, raises Rust count), D9 (differs, raises Geant4 count), D10 undecidable; N1 (0.8 x safety clip on e_aq displacement) not audited. The tables fit about 0.72 fewer recombinations per event in Rust at 1 keV, looser at 10 and 100 keV.

## Blockers / Challenges

None

## Commits

- bfe2a73 docs: add M6 residual source-comparison draft (ticket 02)

## Time Spent

11m (ticket-start.js to ticket-complete.js)
