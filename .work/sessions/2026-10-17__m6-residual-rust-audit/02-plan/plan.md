# Implementation Plan

**Session:** m6-residual-rust-audit
**Date:** 2026-10-17T10:04:32.828Z
**Estimated effort:** 1-2 days

## Strategy

Build the conservation-relation and multi-seed harness first, then compare the Rust channel tables with the Geant4 11.4.1 sources, then decide the stage family and make a fix only if a Geant4 .cc line proves a defect, then record the result in the docs.

## Tickets Overview

- **Ticket 1:** Ignored audit test with pre-registered relations and a 5-seed Rust rerun.
- **Ticket 2:** Source comparison table of Rust vs Geant4 channel, fraction and recombination code.
- **Ticket 3:** Stage verdict and a source-proven fix, if any, with the gate rerun under unchanged criteria.
- **Ticket 4:** Record the result in docs/08, docs/09 and the backlog, and run the full checks.

## Sequencing Rationale

The relations (1) steer the source re-read (2). Both feed the verdict and any fix (3). Documentation (4) comes last so it records what was found.

## Risks & Mitigation

- **Risk:** The Geant4 CSV cannot decide, because per-event covariance is unavailable. → **Mitigation:** Use a conservative bound, state the limitation, and fall back to "undecided" and the counts-request seed.
- **Risk:** 900 keV multi-seed runs are slow. → **Mitigation:** Fall back to 2 seeds there and say so.
- **Risk:** A fix changes Rust baselines. → **Mitigation:** Re-baseline with documented justification; never touch Geant4 reference files.

## Assumptions

- Events are independent for variance estimates.
- Agent sessions never run Geant4.
- No tolerance is loosened.
