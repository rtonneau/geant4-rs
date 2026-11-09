# Handoff: investigate-irt-syn-rr008-coverage

**Session:** 2026-11-09__investigate-irt-syn-rr008-coverage
**Saved:** 2026-11-09T11:18:32.496Z
**Current phase:** plan
**Active ticket:** none

## Where I Stopped

GPS plan started; no tickets drafted or implementation performed. The approved contract remains in 01-grill/resume.md.

## Reasoning So Far

An independent source review confirmed the finite-horizon bound and thresholds: q=0.5429992724599416, probability 0.17668372302986–0.18099975748665. The half-horizon first-passage tail is 0.0068211080203; activation-delay exception bound is 0.0170244181656 and complement delay is at most 359685.1598 ps. Independent scipy beta calculations confirm PASS counts 259–438; at 258/259 the lower bounds are 0.09995345086/0.10039383969 and at 438/439 upper bounds are 0.24955051559/0.25006463210. These assume independent ideal draws. The source and earlier reconstruction are documented in resume.md.

## Next Step

Draft and save GPS tickets for the approved producer, consumer, reproducible verification and request; implement and compile-check, then conduct independent review. Locate a valid Geant4 11.4.1 installation or document the compile limitation. Register a request only after its exact commands and code are reviewable.

## Open Questions

Configured G4_ROOT is 11.4.3; availability of an 11.4.1 installation has not been established. No new generator run is authorized.

## Settled Decisions (do not re-litigate)

Preserve all existing data, ticket history, caps, acceptance criteria and INCONCLUSIVE verdicts. The new experiment is distinct and cannot resolve the 233 forced endpoints or retroactively pass RR-008. No generator runs in this task.

## Uncommitted Work

- **Git status (project):**  M crates/g4-chemistry/src/lib.rs,  M docs/07-validation-strategy.md,  M validation/gates.json,  M validation/geant4/BUILD_NOTES.md,  M validation/geant4/README.md,  M validation/geant4/RUN_REQUESTS.md,  M validation/geant4/g4rs_ref/CMakeLists.txt,  M validation/geant4/g4rs_ref/g4rs_ref.cc, ?? crates/g4-chemistry/src/syn.rs, ?? crates/g4-chemistry/tests/geant4_syn_pair.rs, ?? crates/g4-chemistry/tests/geant4_syn_paired.rs, ?? crates/g4-chemistry/tests/geant4_syn_scheduler.rs, ?? crates/g4-chemistry/tests/syn_analytic_moments.rs, ?? crates/g4-chemistry/tests/syn_kernels.rs, ?? crates/g4-chemistry/tests/syn_support/mod.rs, ?? validation/geant4/SYN2_FEASIBILITY.md, ?? validation/geant4/g4rs_ref/SynKernels.hh, ?? validation/geant4/g4rs_ref/SynPaired.cc, ?? validation/geant4/g4rs_ref/SynScheduler.cc
- **Git status (session dir):** ?? .work/sessions/2026-11-09__investigate-irt-syn-rr008-coverage/.session-config.json, ?? .work/sessions/2026-11-09__investigate-irt-syn-rr008-coverage/01-grill/resume.md, ?? .work/sessions/2026-11-09__investigate-irt-syn-rr008-coverage/02-plan/plan.md, ?? .work/sessions/2026-11-09__investigate-irt-syn-rr008-coverage/HANDOFF.md
- **Why not committed:** Prior ticket work remains incomplete; no new implementation is complete.

## Machine State (auto-filled)

- **Ticket queue:** no tickets yet
- **Recent commits:** no commits yet
