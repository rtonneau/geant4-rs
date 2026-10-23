# Handoff: m7-sbs-oracle-and-ode

**Session:** 2026-10-23__m7-sbs-oracle-and-ode
**Saved:** 2026-10-23T20:40:29.796Z
**Current phase:** ship
**Active ticket:** 02-irt-sbs-statistical-oracle

## Where I Stopped

Ticket 02 is blocked at the fixed-seed IRT/SBS comparison. The only failing
point is H+H at tau=0.25: IRT=0.002240, source-faithful SBS=0.000880, against a
combined three-sigma bound of 0.000632. Work stopped after diagnosing the
origin of that discrepancy; no production fix was selected.

## Reasoning So Far

The test correctly repeats SBS steps until each observation boundary, and its
physical parameters and derived time scale agree with the IRT setup. The
failure is deterministic (`cargo test -p dna-chemistry --test sbs_validation
-- --nocapture`) and the other five observation points pass.

Two interacting effects explain the early deficit:

1. The tau=0.25 observation time is one binary64 ULP larger than the initial
   adaptive timestep. Exact equality handling therefore does not mark the
   large step as observation-capped; the bridge is attempted only over the
   one-ULP remainder.
2. When that near-equality is treated as capped, the executable Geant4
   `G4DNASmoluchowskiReactionModel::FindReaction` loop quirk becomes dominant:
   it resumes at the coordinate that already exceeded R^2, counts that
   coordinate twice, inflates the post-step distance, and suppresses bridge
   encounters.

Controlled diagnostic results at tau=0.25 were:

- current source-faithful behavior: SBS=0.000880 (fails);
- one-ULP-near equality classified as capped, source bridge quirk retained:
  SBS=0.001020 (fails);
- ordinary Euclidean bridge distance alone: SBS=0.000880 (unchanged because
  the large interval still did not invoke the bridge);
- one-ULP-near equality plus ordinary Euclidean distance: SBS=0.002160
  (passes against IRT=0.002240, and all six points pass).

Copying the Geant4 `adaptive <= userMinTimeStep` branch literally into the
isolated observation loop was rejected: `userMinTimeStep` is a scheduler
minimum rather than simply observation time remaining, and the probe made all
later points substantially worse. Every temporary DEBUG-H probe was restored.

## Next Step

Move directly to Ticket 04 and implement the compile-only Geant4 11.4.1 H+H
two-body reference mode. The agent must not run the Geant4 executable. Give the
user the exact build/run command and CSV contract for survival counts at
tau=[0.25,0.5,1,2,4,16]. Once the user supplies the CSV, compare the first
point in particular with the two competing expectations: about 0.0010 when
the executable bridge-distance quirk is preserved versus about 0.0022 for the
Euclidean/IRT result. That measurement decides whether Ticket 02 should test
source-faithful Geant4 SBS behavior or agreement with the exact IRT law.

## Open Questions

- What survival counts does the compiled Geant4 11.4.1 reference produce?
- Does the actual scheduler activate the along-step bridge at the first
  observation boundary despite the one-ULP construction difference?
- If Geant4 reproduces the early deficit, should the statistical oracle accept
  that source-faithful discrepancy rather than require IRT/SBS equality?

## Settled Decisions (do not re-litigate)

- Geant4 11.4.1 is the normative C++ source version.
- The SBS oracle diffuses both particles independently using the existing Rust
  Brownian kernel and uses inclusive endpoint contact.
- The Rust kernel preserves the executable `FindReaction` coordinate
  double-count until the C++ reference result justifies a different oracle.
- Exact boundary equality is observation-capped and invokes the bridge; this
  change alone does not resolve the statistical failure.
- The H+H comparison uses N=100,000, r0=0.2 nm, fixed independent seeds, the
  six pre-registered tau points, and combined three-sigma binomial bounds.
- The C++ reference mode is compile-only for the agent: only the user runs it.
- No source change should be made merely to turn the statistical test green
  before the C++ evidence is available.

## Uncommitted Work

- **Git status (project):**  M crates/dna-chemistry/src/sbs.rs,  M crates/dna-chemistry/tests/sbs.rs, ?? crates/dna-chemistry/tests/sbs_validation.rs
- **Git status (session dir):**  M .work/sessions/2026-10-23__m7-sbs-oracle-and-ode/.session-config.json, ?? .work/sessions/2026-10-23__m7-sbs-oracle-and-ode/03-implement/02-irt-sbs-statistical-oracle/commit-log.md
- **Why not committed:** Ticket 02 is not complete. The equality semantics and
  statistical harness are retained for review, but the pre-registered test is
  red and the correct oracle depends on the user-run Geant4 reference data.

## Machine State (auto-filled)

- **Ticket queue:** 01-source-faithful-sbs-kernel: done, 02-irt-sbs-statistical-oracle: pending, 03-v10-lsoda-well-mixed-oracle: pending, 04-geant4-two-body-reference-mode: pending, 05-preregister-geant4-comparison: pending, 06-document-oracles-and-open-gate: pending
- **Recent commits:** b486d2e chore(gps): ticket 01-source-faithful-sbs-kernel done, bd3b2b1 feat: add source-faithful SBS oracle (ticket 01), 43dbac8 chore(gps): plan 2026-10-23__m7-sbs-oracle-and-ode
