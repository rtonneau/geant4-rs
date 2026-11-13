# Handoff: irt-syn-parallel-checkpoint-direction

**Session:** 2026-11-08__irt-syn-parallel-checkpoint-direction
**Saved:** 2026-11-09T09:07:26.776Z
**Current phase:** ship
**Active ticket:** 02-syn-kernels

## Where I Stopped

RR008 crash recovery succeeded. Independent fresh review no findings; fresh smoke passed; registered production completed and exact consumer ran. Ticket02 blocked on INCONCLUSIVE coverage, not crash. All children stopped.

## Reasoning So Far

Recoverypass3 fixed model selection reset by physics construction and missing molecular physics tables using supported standalone initialization. Actual installedmodel guards verified. RR008 both replicas nativeexit0,288rows/footer; r1=80reactions208censored,r2=79/209. Channel11+z zero events in both. Consumerexit101 stops at r1gap; fullvalidator checked both. Data2264b14ffe024b2f44accb2c53b29ae40ee93d37 and provenance9b4aaf4 committed. RR006/RR007 untouched. All233 prior positive-duration moment endpoints remainINCONCLUSIVE.

## Next Step

Reconcile coverage gap without more samples/seed rerolls/criteria changes (user explicitly prohibits). Source/evidence diagnosis may explain gap but no further recovery or newruns currently authorized. Do not markticketdone or start later dependenttickets while registered criterion unmet. Preserve originalbase2df15d1d652d9dfb2ff582e389c2ea1a02825746 and recoverycount3. Future resumption needs newevidence or an explicit prospective scope/diagnostic decision, not relaxing old results.

## Open Questions

Why channel11+z never reacts in both existing traces is unresolved. Current observeddata cannot establish required fullsetting applicability. No physicalfailure verdict inferred from lackofcoverage.

## Settled Decisions (do not re-litigate)

User approved GPSauto delegated; one extraRR008 recovery used (count3). Independentreview and freshsmoke gates satisfied. Explicit noRR006/007reruns/replacements, noextrasamples/capchanges, no233inconclusives treatedasPASS. Normative11.4.1 retained. No policyblocker. Sharedcheckoutonewriter.

## Uncommitted Work

- **Git status (project):**  M crates/g4-chemistry/src/lib.rs,  M docs/07-validation-strategy.md,  M validation/gates.json,  M validation/geant4/BUILD_NOTES.md,  M validation/geant4/README.md,  M validation/geant4/RUN_REQUESTS.md,  M validation/geant4/g4rs_ref/CMakeLists.txt,  M validation/geant4/g4rs_ref/g4rs_ref.cc, ?? crates/g4-chemistry/src/syn.rs, ?? crates/g4-chemistry/tests/geant4_syn_pair.rs, ?? crates/g4-chemistry/tests/geant4_syn_paired.rs, ?? crates/g4-chemistry/tests/geant4_syn_scheduler.rs, ?? crates/g4-chemistry/tests/syn_analytic_moments.rs, ?? crates/g4-chemistry/tests/syn_kernels.rs, ?? crates/g4-chemistry/tests/syn_support/mod.rs, ?? validation/geant4/SYN2_FEASIBILITY.md, ?? validation/geant4/g4rs_ref/SynKernels.hh, ?? validation/geant4/g4rs_ref/SynPaired.cc, ?? validation/geant4/g4rs_ref/SynScheduler.cc
- **Git status (session dir):**  M .work/sessions/2026-11-08__irt-syn-parallel-checkpoint-direction/.session-config.json, ?? .work/sessions/2026-11-08__irt-syn-parallel-checkpoint-direction/03-implement/02-syn-kernels/commit-log.md, ?? .work/sessions/2026-11-08__irt-syn-parallel-checkpoint-direction/HANDOFF.md
- **Why not committed:** Ticket02 code/docs remain partial and not fully accepted; evidence and provenance separatelycommitted. Required compile/fmt/clippy/workspacetests passed recovery3; no scientificsource changes afterward, only resultdocumentation. Final RR008 consumer intentionally reportsINCONCLUSIVE. NoPR/push. All generators/modes/files preserved; no committed evidence overwritten.

## Machine State (auto-filled)

- **Ticket queue:** 01-reference-contract: done, 02-syn-kernels: pending, 03-serial-syn: pending, 04-timed-injection: pending, 05-scoring: pending, 06-checkpoint: pending, 07-spatial-ownership: pending, 08-parallel-syn: pending, 09-cli: pending, 10-release-validation: pending
- **Recent commits:** 9b4aaf4 data: record RR-008 provenance and inconclusive coverage, 2264b14 data: preserve complete inconclusive RR-008 scheduler traces, f3c169f data: record provenance for IRT-syn diagnostic campaigns, 9607036 data: preserve complete IRT-syn pair and mapping diagnostics, 2df15d1 chore(gps): ticket 01-reference-contract done, 7496eae docs: define IRT-syn reference contract (ticket 01), f5f2eaf chore(gps): plan 2026-11-08__irt-syn-parallel-checkpoint-direction
