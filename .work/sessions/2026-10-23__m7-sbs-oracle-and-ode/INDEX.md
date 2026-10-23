# Session Summary: m7-sbs-oracle-and-ode

**Session ID:** 2026-10-23__m7-sbs-oracle-and-ode
**Created:** 2026-10-23T14:19:05.995Z
**Finished:** 2026-10-23T22:03:44.098Z
**Status:** Complete

## Grill

- Resume: [01-grill/resume.md](01-grill/resume.md)

## Plan

- Plan: [02-plan/plan.md](02-plan/plan.md)

## Tickets

- ✅ 01 source-faithful-sbs-kernel — [spec](02-plan/tickets/01-source-faithful-sbs-kernel.md) · [log](03-implement/01-source-faithful-sbs-kernel/commit-log.md)
- ✅ 02 irt-sbs-statistical-oracle — [spec](02-plan/tickets/02-irt-sbs-statistical-oracle.md) · [log](03-implement/02-irt-sbs-statistical-oracle/commit-log.md)
- ✅ 03 v10-lsoda-well-mixed-oracle — [spec](02-plan/tickets/03-v10-lsoda-well-mixed-oracle.md) · [log](03-implement/03-v10-lsoda-well-mixed-oracle/commit-log.md)
- ✅ 04 geant4-two-body-reference-mode — [spec](02-plan/tickets/04-geant4-two-body-reference-mode.md) · [log](03-implement/04-geant4-two-body-reference-mode/commit-log.md)
- ✅ 05 preregister-geant4-comparison — [spec](02-plan/tickets/05-preregister-geant4-comparison.md) · [log](03-implement/05-preregister-geant4-comparison/commit-log.md)
- ✅ 06 document-oracles-and-open-gate — [spec](02-plan/tickets/06-document-oracles-and-open-gate.md) · [log](03-implement/06-document-oracles-and-open-gate/commit-log.md)

## Changelog

- **Bump:** minor
- **Fragment:** [2026-10-23__m7-sbs-oracle-and-ode.md](../../changelog/2026-10-23__m7-sbs-oracle-and-ode.md) (merged into the CHANGELOG at release)

## Branch & PR

- **Branch:** `test/m7-sbs-oracle-and-ode`
- **Base:** `main`
- **Pull request:** https://github.com/rtonneau/geant4-rs/pull/20

## Timeline

| When | Phase | Event | Details | Files |
|---|---|---|---|---|
| 2026-10-23 16:19 | grill | session_started |  | [01-grill/resume.md](01-grill/resume.md) |
| 2026-10-23 21:59 | plan-not-started | grill_written |  | [01-grill/resume.md](01-grill/resume.md) |
| 2026-10-23 21:59 | plan | plan_started |  | [02-plan/plan.md](02-plan/plan.md) |
| 2026-10-23 22:02 | plan | branch_created | branch: test/m7-sbs-oracle-and-ode, base: main |  |
| 2026-10-23 22:02 | ship | plan_written | tickets: 6 | [02-plan/plan.md](02-plan/plan.md), [02-plan/tickets/01-source-faithful-sbs-kernel.md](02-plan/tickets/01-source-faithful-sbs-kernel.md), [02-plan/tickets/02-irt-sbs-statistical-oracle.md](02-plan/tickets/02-irt-sbs-statistical-oracle.md), [02-plan/tickets/03-v10-lsoda-well-mixed-oracle.md](02-plan/tickets/03-v10-lsoda-well-mixed-oracle.md), [02-plan/tickets/04-geant4-two-body-reference-mode.md](02-plan/tickets/04-geant4-two-body-reference-mode.md), [02-plan/tickets/05-preregister-geant4-comparison.md](02-plan/tickets/05-preregister-geant4-comparison.md), [02-plan/tickets/06-document-oracles-and-open-gate.md](02-plan/tickets/06-document-oracles-and-open-gate.md) |
| 2026-10-23 22:03 | ship | ticket_started | ticket: 01-source-faithful-sbs-kernel | [02-plan/tickets/01-source-faithful-sbs-kernel.md](02-plan/tickets/01-source-faithful-sbs-kernel.md), [03-implement/01-source-faithful-sbs-kernel/commit-log.md](03-implement/01-source-faithful-sbs-kernel/commit-log.md) |
| 2026-10-23 22:10 | ship | ticket_done | ticket: 01-source-faithful-sbs-kernel, commit: bd3b2b1 | [03-implement/01-source-faithful-sbs-kernel/commit-log.md](03-implement/01-source-faithful-sbs-kernel/commit-log.md) |
| 2026-10-23 22:10 | ship | ticket_started | ticket: 02-irt-sbs-statistical-oracle | [02-plan/tickets/02-irt-sbs-statistical-oracle.md](02-plan/tickets/02-irt-sbs-statistical-oracle.md), [03-implement/02-irt-sbs-statistical-oracle/commit-log.md](03-implement/02-irt-sbs-statistical-oracle/commit-log.md) |
| 2026-10-23 22:20 | ship | ticket_blocked | ticket: 02-irt-sbs-statistical-oracle, reason: Pre-registered IRT/SBS comparison fails at tau=0.25: CDF residual 0.001360 exceeds the fixed combined 3-sigma bound 0.000632; the other five points pass. | [03-implement/02-irt-sbs-statistical-oracle/commit-log.md](03-implement/02-irt-sbs-statistical-oracle/commit-log.md) |
| 2026-10-23 22:29 | ship | ticket_blocked | ticket: 02-irt-sbs-statistical-oracle, reason: Equality bridge rule is implemented and its deterministic regression passes, but tau=0.25 observation time is one ULP above the adaptive step, so the unchanged 100,000-pair comparison still fails: residual 0.001360 > bound 0.000632. | [03-implement/02-irt-sbs-statistical-oracle/commit-log.md](03-implement/02-irt-sbs-statistical-oracle/commit-log.md) |
| 2026-10-23 22:40 | ship | handoff_saved |  | [HANDOFF.md](HANDOFF.md) |
| 2026-10-23 22:43 | ship | ticket_started | ticket: 04-geant4-two-body-reference-mode | [02-plan/tickets/04-geant4-two-body-reference-mode.md](02-plan/tickets/04-geant4-two-body-reference-mode.md), [03-implement/04-geant4-two-body-reference-mode/commit-log.md](03-implement/04-geant4-two-body-reference-mode/commit-log.md) |
| 2026-10-23 22:57 | ship | ticket_done | ticket: 04-geant4-two-body-reference-mode, commit: efd367c | [03-implement/04-geant4-two-body-reference-mode/commit-log.md](03-implement/04-geant4-two-body-reference-mode/commit-log.md) |
| 2026-10-23 23:10 | ship | ticket_started | ticket: 05-preregister-geant4-comparison | [02-plan/tickets/05-preregister-geant4-comparison.md](02-plan/tickets/05-preregister-geant4-comparison.md), [03-implement/05-preregister-geant4-comparison/commit-log.md](03-implement/05-preregister-geant4-comparison/commit-log.md) |
| 2026-10-23 23:15 | ship | ticket_done | ticket: 05-preregister-geant4-comparison, commit: 53643e9 | [03-implement/05-preregister-geant4-comparison/commit-log.md](03-implement/05-preregister-geant4-comparison/commit-log.md) |
| 2026-10-23 23:17 | ship | ticket_blocked | ticket: 02-irt-sbs-statistical-oracle, reason: Geant4 11.4.1 confirms the executable SBS curve does not agree with IRT; all-six-pass acceptance conflicts with source fidelity and needs a specification decision | [03-implement/02-irt-sbs-statistical-oracle/commit-log.md](03-implement/02-irt-sbs-statistical-oracle/commit-log.md) |
| 2026-10-23 23:20 | ship | ticket_done | ticket: 02-irt-sbs-statistical-oracle, commit: 1efe887 | [03-implement/02-irt-sbs-statistical-oracle/commit-log.md](03-implement/02-irt-sbs-statistical-oracle/commit-log.md) |
| 2026-10-23 23:21 | ship | ticket_started | ticket: 03-v10-lsoda-well-mixed-oracle | [02-plan/tickets/03-v10-lsoda-well-mixed-oracle.md](02-plan/tickets/03-v10-lsoda-well-mixed-oracle.md), [03-implement/03-v10-lsoda-well-mixed-oracle/commit-log.md](03-implement/03-v10-lsoda-well-mixed-oracle/commit-log.md) |
| 2026-10-23 23:23 | ship | ticket_done | ticket: 03-v10-lsoda-well-mixed-oracle, commit: 1b0171b | [03-implement/03-v10-lsoda-well-mixed-oracle/commit-log.md](03-implement/03-v10-lsoda-well-mixed-oracle/commit-log.md) |
| 2026-10-23 23:32 | ship | ticket_started | ticket: 06-document-oracles-and-open-gate | [02-plan/tickets/06-document-oracles-and-open-gate.md](02-plan/tickets/06-document-oracles-and-open-gate.md), [03-implement/06-document-oracles-and-open-gate/commit-log.md](03-implement/06-document-oracles-and-open-gate/commit-log.md) |
| 2026-10-23 23:37 | finish-pending | ticket_done | ticket: 06-document-oracles-and-open-gate, commit: 991c02f | [03-implement/06-document-oracles-and-open-gate/commit-log.md](03-implement/06-document-oracles-and-open-gate/commit-log.md) |
| 2026-10-24 00:03 | finish-pending | changelog_written | bump: minor |  |
| 2026-10-24 00:03 | finish-pending | pr_opened | url: https://github.com/rtonneau/geant4-rs/pull/20 |  |
| 2026-10-24 00:03 | finished | session_finished |  | [INDEX.md](INDEX.md) |

## Next

Start a new feature with /gps start <next-feature>
