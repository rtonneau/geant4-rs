# Ticket 02: irt-sbs-statistical-oracle

**Status:** ✅ Done

## Local Test Result

PASS after the user selected executable Geant4 fidelity:

- `cargo test -p dna-chemistry --test sbs_validation -- --nocapture`: 2 passed,
  including the 100,000-pair fixed-seed comparison. It printed all six residuals;
  tau=0.25 exceeded the original bound and the other five remained within it.
- `cargo test -p dna-chemistry`: the complete package passed, with the external
  RR-003 comparison correctly ignored (no failures).
- `cargo fmt --all -- --check` and `git diff --check` passed.

## Review Notes

The statistical family, sample count, independent seeds, V9 probabilities, tau points
and three-sigma bounds are unchanged. Survival still includes IRT nonreaction mass and
every SBS pair not reacted by each observation boundary. The regression now locks the
observed source-faithful signature: tau=0.25 must exceed the originally registered bound,
while the other five points must satisfy it. Its header explicitly distinguishes the
original preregistration from the later discrepancy classification.

The earlier equality change is retained and covered deterministically: an observation
boundary exactly equal to the adaptive step is capped and invokes the bridge, while an
uncapped step and endpoint contact consume no bridge draw. The executable bridge-distance
quirk remains unchanged.

## Blockers / Challenges

**Blocked (2026-10-23T20:20:28.988Z):** Pre-registered IRT/SBS comparison fails at tau=0.25: CDF residual 0.001360 exceeds the fixed combined 3-sigma bound 0.000632; the other five points pass.

**Blocked (2026-10-23T20:29:25.085Z):** Equality bridge rule is implemented and its deterministic regression passes, but tau=0.25 observation time is one ULP above the adaptive step, so the unchanged 100,000-pair comparison still fails: residual 0.001360 > bound 0.000632.

**Blocked after Geant4 reference run (2026-10-23):** The corrected Geant4 11.4.1
executable decisively rejects the alternative of changing the SBS oracle merely to match
IRT. With seed 202610231 and 100,000 pairs it produced reacted fractions 0, 0.00099,
0.00875, 0.03018, 0.05586 and 0.20551 at the six tau points, far below IRT. The
source-transcription test remains deterministically red only at tau=0.25 (IRT 0.002240,
Rust SBS 0.000880, bound 0.000632). Satisfying the ticket's all-six-pass acceptance
criterion now requires a product/specification decision to abandon executable fidelity,
change the registered family, or redefine this test as a documented discrepancy. No seed,
point, tolerance or implementation was tuned after observing the reference data.

**Resolved by user decision:** Preserve Geant4 executable fidelity and redefine the
Ticket 02 result as a locked, documented early discrepancy rather than force IRT agreement.
No numerical input or tolerance changed.

**Blocked (2026-10-23T21:17:17.548Z):** Geant4 11.4.1 confirms the executable SBS curve does not agree with IRT; all-six-pass acceptance conflicts with source fidelity and needs a specification decision

## Commits

- 1efe887 test: record source-faithful SBS discrepancy (ticket 02)

## Time Spent

1h 09m (ticket-start.js to ticket-complete.js)
