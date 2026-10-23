# Ticket 02: irt-sbs-statistical-oracle

**Status:** In Progress

## Local Test Result

<!-- gps:fill The Verification Step command you ran and its output -->

## Review Notes

<!-- gps:fill What you checked against the Acceptance Criteria, and anything you changed after review -->

## Blockers / Challenges

**Blocked (2026-10-23T20:20:28.988Z):** Pre-registered IRT/SBS comparison fails at tau=0.25: CDF residual 0.001360 exceeds the fixed combined 3-sigma bound 0.000632; the other five points pass.

**Blocked (2026-10-23T20:29:25.085Z):** Equality bridge rule is implemented and its deterministic regression passes, but tau=0.25 observation time is one ULP above the adaptive step, so the unchanged 100,000-pair comparison still fails: residual 0.001360 > bound 0.000632.

## Commits

Filled by ticket-complete.js.

## Time Spent

Filled by ticket-complete.js.
