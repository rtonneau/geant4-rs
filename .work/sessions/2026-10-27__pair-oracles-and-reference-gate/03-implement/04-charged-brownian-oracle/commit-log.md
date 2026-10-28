# Ticket 04: charged-brownian-oracle

**Status:** ✅ Done

## Local Test Result

`cargo test --release -p g4-chemistry --test pair_brownian charged -- --nocapture`: 2 passed, 0 failed (parent rerun). Charged output reports exact physical-versus-Geant4 eventual discrepancies as FAIL and keeps G1 OPEN. Brownian finite-time law and eventual tail remain INCONCLUSIVE. The subagent also ran formatting, workspace Clippy, and workspace tests successfully.

## Review Notes

Reviewed gpt-6-astra (low) implementation against the Coulomb drift, absorbing/radiation boundary, A+A cases, frozen timestep schedule and separate approximation verdict. Checked the 80/160-digit physical anchors and explicit INCONCLUSIVE language. Fixed two corrupted Unicode characters in comments of `analytic.py`; no numerical code changed. The missing finite-time exact Coulomb anchors and unresolved eventual tail are documented limitations and do not grant G1.

## Blockers / Challenges

No implementation blocker. The scientific gate remains open pending finite-time physical anchors and tail resolution; exact physical-versus-approximation differences already exceed the registered effect limit in charged cases.

## Commits

- 7b0daf5 test: add charged Coulomb pair oracle (ticket 04)

## Time Spent

9m (ticket-start.js to ticket-complete.js)
