# R04 independent evidence implementation plan (2026-10-29)

**Goal:** complete as much of the frozen R04 evidence as can be justified,
including an honest unresolved verdict when numerical or computational limits
prevent closure. The authority is `pair-case-matrix.md`; its criteria are unchanged.

**Architecture:** preserve the independent radial Euler/bridge process and
history streams. Add trajectory stopping records and population probability
bounds, not sampled analytic tails. Validate charged finite-time anchors with
an independently refined backward PDE. Keep physical and effective-distance
laws, executable tests and scientific verdicts separate.

**Constraints:** no changes to reference files, Geant4 source, frozen seeds,
sample sequence, maximum physical time steps, family alpha or effect limits.
Existing dirty work is retained. No Geant4 generator is run. Work remains in
the user-requested checkout. The user authorized autonomous implementation.

## Tasks and checks

- [x] Add tested stopping records: reacted, escaped at a sampled endpoint,
  unresolved at the horizon. A sampled endpoint crossing is a stopping time;
  missed intervening exits delay stopping and cannot falsely certify escape.
  The exact continuum contact-return probability supplies only a bound.
- [x] Add population intervals with simultaneous Monte Carlo coverage, keeping
  censoring, return risk and unbounded discretization error visible separately.
  Test zero counts, all-unresolved paths, absorbing contact, both charge signs,
  conservation, RNG prefix stability and neutral recovery.
- [x] Independently validate PDE space/time/outer/Robin errors; retain all
  refinements, neutral controls and charge signs in new evidence files.
- [x] Integrate physical finite anchors and trajectory bounds into the frozen
  schedule, retaining every estimate and seed/input/source provenance. Reuse
  unchanged sample prefixes without rerunning histories. Expand on Monte Carlo
  precision, never on significance. Report unavailable limits explicitly.
- [x] Investigate discrepancies, run analytic family and required repository
  checks, document actual completed schedule and separate verdicts. Do not R05.

## Decisions before new results

The default stopping audit observes the registered trajectories through tau=100
and records the first sampled radius >=16. Longer deterministic horizons and
larger outer radii are configurable and retain dt<=h. This is an initial
censored endpoint, not a promise that its identification interval resolves
0.005. Analytic continuation, increasing physical dt far from contact and
classifying terminal survivors as confirmed nonreactors are excluded.

For continuum trajectories, with reaction/escape/unresolved probabilities
A/E/U and contact-return upper bound q, eventual nonreaction is in
[(1-q)E, 1-A]. Monte Carlo intervals add simultaneous one-sided concentration
bounds over 216 endpoints, eight sample sizes and five levels. These bounds
do not cover Euler discretization. Frozen successive-level tests are retained;
without resolution of censoring and discretization there is no Brownian PASS.

## Progress

Initial read: all listed files and repository instructions inspected; DEV_DIR
and G4_ROOT are set. Local Geant4 IRT source was read, never modified. Existing
code only evaluates finite Brownian CDFs and has no complete scientific PASS.
PDE work is delegated independently; a read-only scientific review confirmed
that a tail time change would violate the frozen maximum physical dt.

Implementation/check tasks completed; scientific completion is deliberately not claimed.
All 36 series ran with precision-selected n up to 320k and all five levels
for nontrivial cases. All 180 finite refinement checks met their rule, while
the general eventual comparison remains INCONCLUSIVE (identification widths
0.016861–0.127948). Exact charged contact falsifies physical source-law
agreement. The detailed report and raw evidence record both findings.
