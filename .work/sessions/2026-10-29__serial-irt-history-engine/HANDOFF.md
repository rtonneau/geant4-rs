# Handoff: serial-irt-history-engine

**Session:** 2026-10-29__serial-irt-history-engine
**Saved:** 2026-11-05T21:43:57.025Z
**Current phase:** ship
**Active ticket:** 01-pair-history

## Where I Stopped

Updated the grill, plan and seven tickets to record G1-F PASS. This task changes prerequisite records only; all seven implementation tickets remain pending.

## Reasoning So Far

The approved G1-F replacement gate passed RR-005 at 200k on Geant4 11.4.3 (closure 7071bd3). Original G1 physical limitations remain open; R05 still needs its own G2 convergence evidence.

## Next Step

When implementation is requested, resume GPS ship with Ticket 01: pair history and event storage. Read its acceptance criteria and inspect the relevant Geant4 .cc sources first.

## Open Questions

No outstanding user decision blocks the prerequisite update. R05 retains the planned 11.4.1 source baseline; verify version differences per behavior rather than inferring a scheduler-wide migration from RR-005.

## Settled Decisions (do not re-litigate)

G1-F is satisfied: 120 included endpoints passed, 24 exact PDC contacts excluded; analytic and contact regressions passed. R05/G2, R06 integration and R07 acceptance are separate. Ticket criteria and phase remain unchanged.

## Uncommitted Work

- **Git status (project):** clean
- **Git status (session dir):**  M .work/sessions/2026-10-29__serial-irt-history-engine/.session-config.json,  M .work/sessions/2026-10-29__serial-irt-history-engine/01-grill/resume.md,  M .work/sessions/2026-10-29__serial-irt-history-engine/02-plan/plan.md,  M .work/sessions/2026-10-29__serial-irt-history-engine/02-plan/tickets/01-pair-history.md,  M .work/sessions/2026-10-29__serial-irt-history-engine/02-plan/tickets/02-competing-events.md,  M .work/sessions/2026-10-29__serial-irt-history-engine/02-plan/tickets/03-products-and-unary.md,  M .work/sessions/2026-10-29__serial-irt-history-engine/02-plan/tickets/04-observation-and-limits.md,  M .work/sessions/2026-10-29__serial-irt-history-engine/02-plan/tickets/05-spatial-index.md,  M .work/sessions/2026-10-29__serial-irt-history-engine/02-plan/tickets/06-network-and-reproducibility.md,  M .work/sessions/2026-10-29__serial-irt-history-engine/02-plan/tickets/07-convergence-and-g2.md
- **Why not committed:** Snapshot taken before the prerequisite-record commit. The pre-existing auto_started event is preserved; GPS appended handoff_saved.

## Machine State (auto-filled)

- **Ticket queue:** 01-pair-history: pending, 02-competing-events: pending, 03-products-and-unary: pending, 04-observation-and-limits: pending, 05-spatial-index: pending, 06-network-and-reproducibility: pending, 07-convergence-and-g2: pending
- **Recent commits:** 7071bd3 Close G1-F with passing RR-005 200k precision evidence, 0b96afe Record RR-005 200k provenance, 51c22b6 Record RR-005 complete 200k precision expansion, 51cd56b Record RR-005 precision-inconclusive result and fix native oxygen name, b7febf0 Record RR-005 run provenance and checksum, eba9b3c Record RR-005 Geant4 11.4.3 complete pair matrix, 4e5ea99 Update authorized RR-005 reference contract to Geant4 11.4.3, d3684e2 feat(validation): add prospective G1-F Geant4 fidelity gate, 82e12eb Add validation scripts and evidence for Brownian and Coulomb PDE analyses, 8d28821 docs: record RR-004 contact smoke failure
