# Ticket 03: neutral-brownian-oracle

**Status:** ✅ Done

## Local Test Result

`cargo test --release -p g4-chemistry --test pair_brownian neutral -- --nocapture` passed: 2 tests, 0 failures. The scientific output for all six neutral cases remains INCONCLUSIVE at the default 10,000-pair, three-level runtime budget; the raw output is retained in the session scratch evidence directory.

## Review Notes

Reviewed the test-only radial Brownian stepper, independent random draws, absorbing/radiation handling, frozen sample/refinement schedules and per-endpoint output. The code does not call production diffusion, SBS or IRT samplers. It never labels survivors at tau=100 as eventual nonreactors, and no analytic tail is substituted for the independent oracle. Convergence and precision limits are explicit, with INCONCLUSIVE status when unresolved. No inline code correction was needed.

## Blockers / Challenges

The initial 10,000-pair run cannot resolve most finite-time endpoints to <0.005 and cannot establish eventual no-reaction mass. The full precision/refinement budget is available through explicit test settings but has not been run. This keeps the Brownian G1 verdict open; a passing Rust test only confirms the experiment and status checks execute.

## Commits

- 99f46c4 test: add neutral Brownian pair oracle (ticket 03)

## Time Spent

10m (ticket-start.js to ticket-complete.js)
