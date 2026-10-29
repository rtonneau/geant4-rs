# R04 independent evidence continuation, 2026-10-29

R04 does not close G1. **ANALYTIC: PASS** against the preregistered source
law. **Physical agreement of that law: FAIL** at exact charged absorbing
contact. **General Brownian numerical convergence and eventual-endpoint
completion: INCONCLUSIVE. GEANT4: REQUESTED.** No R05 work is undertaken.
The exact-contact falsification is a physical-versus-source-law comparison;
it is not a new direct paired comparison with production sampler counts.

The acceptance matrix, seeds, physical maximum steps, sample expansion
sequence and effect/family limits have not changed. Existing dirty work,
reference files and earlier evidence are preserved. No Geant4 generator was
run, no reference source/build patched, and no contact position shifted.

## Trajectory stopping and eventual nonreaction

`pair_oracle/stopped.rs` uses the same radial Euler/bridge arithmetic, random
draw order, local-time boundary and history IDs as the previous neutral and
charged integrators. Tests compare finite counts exactly for zero charge and
both charge signs, and prove prefix batching agrees with a single batch.

The new endpoint records the first **sampled surviving radius** at or above
an outer sphere R. It records reaction if the path reacts before that stopping
time. Paths reaching neither state by the deterministic horizon remain
unresolved. After an eventual escape is recorded, the path still runs through
tau=100 to preserve every finite observation. A later finite-time reaction
does not retroactively turn its earlier escape into a confirmed nonreaction
or move it between the disjoint stopping categories.

Checking only sampled radii deliberately delays detection of an outer
crossing. For the continuum process sampled on that grid this is a stopping
time at a radius >=R; missed hidden crossings cannot produce a false escape.
Reaction takes priority over the outer test within a step. Numerical contact
placement and frozen drift still need refinement; there is no claim that an
Euler path has the exact continuum stopping distribution.

All steps obey `dt <= h`, including optional continuation beyond tau=100.
The initial stopping audit uses R=16 and horizon=100. Both are recorded and
configurable. The endpoint preserves the radius, accumulated local time,
exponential threshold, RNG and spare Box–Muller variate across observations
and continuation. No shared wall-clock cutoff decides a path's category.

The exponential boundary-local-time convention is consistent with the
encounter formulation reviewed by [Grebenkov (2023)](https://pmc.ncbi.nlm.nih.gov/articles/PMC10674959/).
The return bound follows directly from this problem's backward
generator, independently of the IRT sampling formula.

Let A, E and U be the probabilities of reaction, escape and unresolved status.
For the continuum radial generator `d²/dr² + (2/r+c/r²)d/dr`, the probability
of ever returning to the contact sphere from R is

```text
q(c,R) = expm1(c/R) / expm1(c),   c != 0
q(0,R) = 1/R.
```

This solves the backward harmonic equation with q(1)=1 and q(infinity)=0.
It is monotone decreasing in R. Return to contact is necessary for reaction,
so it upper-bounds subsequent reaction even for a radiation boundary. The
strong Markov property yields the population identification interval

```text
(1-q) E <= P(eventual nonreaction) <= 1-A.
```

Its empirical width is `U/n + q E/n`. The code does **not** sample a Bernoulli
outcome from q, impute a tail reaction time, use an analytic radiation tail,
or label any escaped/surviving path a confirmed nonreactor. This bound is not
a pathwise confidence interval and is not an exact continuation of a path.

For sampling uncertainty, use two one-sided Hoeffding bounds with a union
bound over 216 endpoints, eight sample sizes and five levels:

```text
w = sqrt(log(2*216*40/0.001)/(2*n))
L_E = max(0, E/n-w); L_A = max(0, A/n-w)
population band = [(1-q)*L_E, 1-L_A].
```

This accounts for selection among the finite set of 40 looks and does not
assume independence between category counts. It covers Monte Carlo error in
the sampled categories, not Euler discretization. Exact absorbing initial
contact is handled separately as a proven deterministic [0,0] nonreaction
endpoint; zero observed rare counts elsewhere do not earn zero uncertainty.

The frozen eventual convergence rule expects an observed Bernoulli endpoint.
A midpoint of a wide censored interval cannot be substituted for it. Thus
these bounds are executable independent trajectory evidence, but the general
registered eventual comparison remains **unresolved**. Increasing n cannot
remove `U/n + q E/n`; a PASS is never inferred from category precision alone.

## Finite-time physical law and numerical error

The independent backward PDE supplies all 90 charged finite-time anchors.
[Its report](pair-coulomb-pde.md) records space, time, outer-domain and two
Robin-stencil checks, both charge signs and all six neutral exact-law controls.
The maximum summed empirical numerical uncertainty is 0.000221989, below the
prospective 0.0005 anchor target. This is a numerical error estimate, not a
certified enclosure. No PDE anchor is generated from Brownian trajectories
or the Geant4 distance mapping.

The Brownian harness now compares neutral estimates with neutral analytic
anchors and charged estimates with the physical PDE, carrying its numerical
error estimate. It retains the source-law anchors in separate columns.
Last-level changes and three combined SE are reported at every finite point.
Only endpoints meeting the frozen refinement conditions can emit a finite
physical-law diagnostic other than INCONCLUSIVE.

Finite z diagnostics retain the frozen `z=4.580907331740323` over 216 endpoints.
Because n and the level are selected from observed widths/refinement, these
fixed-z diagnostics alone are **not** a simultaneous sequential family
guarantee. The additional 40-look Hoeffding guarantee applies to the eventual
category bands only. No family PASS is claimed from the finite diagnostics.

## Registered computational schedule

The runner permits all five frozen levels and the full sample expansion up
to one million, with seed 20261027402 and the original history ordinal
`((case*3+distance)*5+level)*1000000+pair`. Expanded batches add independent
new histories and retain all earlier prefixes; no samples are discarded.

At every used n, the first three levels run. If their MC widths remain too
large, n expands through the frozen sequence before spending work on finer
levels. Once precise, every noncontact series runs through h=1/256 because
its eventual comparison is unresolved. Exact absorbing contact can stop at
the first three levels. Widths, not law-test significance, select expansion.
The final size must resolve the three-SE precision check for finite counts
and both stopping categories, plus the finite family half-width with the
anchor error estimate. Unresolved censoring is reported separately.

Each JSONL row retains case, distance, n, h, level, time/count or stopping
categories, anchors, uncertainty, last-level changes and diagnostics. Source
and fixture hashes, dirty revision, compiler/platform, environment and exact
command are retained with the run. The runner refuses an existing output
directory. `summarize_brownian.py` refuses incomplete families and verifies
the sample/level sequence before producing a summary.

Reproduce into a **new** output directory:

```text
python validation/pair_oracles/run_brownian.py --output .scratch/r04-brownian-reproduction --threads 6
python validation/pair_oracles/summarize_brownian.py .scratch/r04-brownian-reproduction --output .scratch/r04-brownian-summary.json
```

## Resolved physical discrepancy

At absorbing r/a=1, a physical path reacts at tau=0 exactly. Its CDF is 1 at
every registered positive time and its eventual nonreaction mass is 0.
Neither Monte Carlo noise, PDE discretization nor a remote boundary can alter
these values. The retained charged TDC trajectories reproduce them exactly.

The Geant4 effective-distance law instead replaces the initial radial
distance by `c/expm1(c/r)` before using its neutral-like first-passage formula.
In the read-only 11.4.1 `G4DNAIRT.cc:286-356`, the TDC early-contact check is
strict (`sigma > r0`), so equality proceeds into that mapping. The resulting
source-law finite probabilities differ from the exact physical boundary by
nearly one at some registered contact observations. For option3-9 its
eventual nonreaction mass is 0.2819278247 rather than zero. The synthetic
charged TDC case shows the same finite-time issue.

These are resolved counterexamples to physical agreement under the frozen
limits, independent of general trajectory convergence. The separately run
analytic family verifies the production sampler against its registered
effective-distance law. That analytic PASS does not make the source law a
physical Coulomb law. The PDC contact failure in direct Geant4 remains a
separate unresolved reference issue, never reclassified as nonreaction.

The final source audit verifies the versioned 11.4.1 `G4DNAIRT.cc` against
tag commit `f01f313f122a19ae77c6f27192c9464ba0279b5e`; its SHA-256 is
`cdd2bec595262ac672f4d5d5f4b15b28d04f06d11579aca254692ca6e23b4686`.
The generic checkout was discovered to be on an 11.5 beta revision and was
excluded as a final reference. The release source and installed version
headers both report 1141. This correction required no change to the numerical
inputs or conclusions; the relevant formulas were confirmed in 11.4.1.

## Investigation of a finite Brownian diagnostic

Option3-8 at r/a=1.5 and tau=1 retains the following estimates at the
precision-selected n=160,000. The independent PDE anchor is 0.1971383059,
with empirical error estimate 0.000032517.

| h | Brownian CDF | Difference from PDE |
|---|---:|---:|
| 1/16 | 0.22488750 | +0.02774919 |
| 1/32 | 0.21431250 | +0.01717419 |
| 1/64 | 0.20865625 | +0.01151794 |
| 1/128 | 0.20208125 | +0.00494294 |
| 1/256 | 0.20181875 | +0.00468044 |

The last difference between levels is 0.0002625, below three combined SE
(0.00425809), but the final nominal z against the PDE is 4.6646, slightly
above 4.5809. The harness therefore emits a statistical diagnostic FAIL.
Its absolute discrepancy is still below 0.005; this is **not** a resolved
effect-band exceedance or a simultaneous sequential family conclusion.
Other points in this series remain INCONCLUSIVE under the physical-law
diagnostic because their uncertainty intervals do not fit inside the effect
band, despite meeting the adjacent-level refinement conditions.

The decreasing excess across levels is consistent with residual time-step
bias. A plausible mechanism is drift freezing during inward excursions:
for this repulsive case the true radial drift rises from about 3.177 at
r=1.5 to 6.149 at contact, while the bridge uses its starting drift throughout
the step. Underestimating that increase can overcount contacts. This explains
a direction of error, not a proven quantitative error bound or an exclusive
cause. PDE refinement errors are much smaller than the observed excess, and
the exact zero-charge stream check rules out a different Gaussian/bridge
implementation between charged and neutral paths.

No n expansion was triggered by this z, no extra registered h level was
invented, and no parameter or threshold was fitted to remove the discrepancy.
The example shows why adjacent-level agreement alone does not establish
agreement with the independent physical law.

## Completed run and retained estimates

The available computational schedule completed in **2133.58 seconds** with
six workers. It retained **2,790 finite-time and 558 eventual-category rows**
across all 36 case/distance series. Each series used the frozen expansion
sequence from 10k through its precision-selected final n. The permitted
640k and 1m sizes were not needed for the MC-width checks. No expansion or
stopping decision depended on whether the law-test significance passed.

All **180 finite endpoints meet the last-two-level refinement rule** at the
selected final n/h. Their physical-anchor diagnostics are **169 PASS,
1 FAIL and 10 INCONCLUSIVE**, subject to the fixed-z/selection qualifications
above. All 30 nontrivial series ran all five h levels; the six deterministic
absorbing contacts used the required first three. These are results from
retained estimates, not inferences from passing executable tests.

All 36 physical eventual anchors lie inside the final population MC bands.
That containment is only a consistency check. The 30 nontrivial empirical
identification intervals alone have widths **0.016861 to 0.127948**, before
MC uncertainty or discretization error. Every one exceeds 0.005. The six
absorbing contacts are exactly resolved, but the general registered eventual
comparison is **not completed**.

Final n values below are ordered r/a=1, 1.5, 3. Diagnostic counts cover each
case's 15 finite points. The last column excludes MC and numerical error.

| Case | Final n at the three distances | Finite diagnostic P/F/I | Largest eventual identification width |
|---|---|---|---:|
| option3-1 | 10k, 320k, 320k | 15/0/0 | 0.062107 |
| option3-2 | 10k, 320k, 320k | 15/0/0 | 0.062138 |
| option3-8 | 10k, 160k, 80k | 11/1/3 | 0.020201 |
| option3-9 | 10k, 320k, 320k | 14/0/1 | 0.070240 |
| option3-11 | 320k, 320k, 160k | 15/0/0 | 0.078600 |
| option3-17 | 320k, 320k, 160k | 14/0/1 | 0.070982 |
| option3-42 | 160k, 160k, 80k | 15/0/0 | 0.127948 |
| option3-51 | 80k, 40k, 40k | 15/0/0 | 0.060085 |
| syn-neutral-tdc-ab | 10k, 320k, 320k | 15/0/0 | 0.062018 |
| syn-charged-tdc-aa | 10k, 320k, 160k | 11/0/4 | 0.049122 |
| syn-neutral-pdc-aa | 320k, 320k, 160k | 15/0/0 | 0.080368 |
| syn-charged-pdc-ab | 320k, 320k, 160k | 14/0/1 | 0.107103 |

The postprocessor separately lists **10 exact contact counterexamples** to
source-law physical agreement. These have no Monte Carlo or time-step
uncertainty and exceed the frozen 0.005 effect threshold. Their FAIL does
not convert other unresolved endpoints into resolved ones.

Evidence lives in `validation/pair_oracles/evidence/brownian_20261029/`:

- `estimates.jsonl` and `run.log`: every attempted level and sample size.
- `summary.json`: final intervals, diagnostics and exact counterexamples.
- `provenance.json`: command, versions, settings, hashes and completion.
- `source-snapshot.zip` and `.json`: exact dirty-tree input/source bytes.
- `rng-provenance.json` and `postrun-verification.json`: RNG/manifests checked
  against captured HEAD, plus post-run archive/source integrity checks.

All captured hashes and archived source bytes match the worktree after the
run. The supplemental RNG check addresses its omission from the runner's
initial hash list. The archive permits reconstruction of the dirty inputs
on top of the recorded repository revision.

## Repository and numerical checks

All required commands were executed successfully:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`: 491 passed, 0 failed, 20 ignored across 57 result blocks.
- Python Coulomb PDE suite: 7 tests passed.
- Release Brownian unit checks: 10 passed; the two scientific experiments
  subsequently completed separately with the recorded scientific diagnostics.
- Release analytic family: 3 tests passed, including all 216 preregistered
  comparisons and biased-sampler rejection.

`validation/pair_oracles/evidence/checks_20261029.json` records the checks;
the sibling `checks_20261029/` directory retains repository and analytic logs.
Ignored external-reference tests were not treated as passing evidence.
Automatic approval review blocked optional removal of generated Python
cache files; they were left in place and are not evidence inputs.

## Remaining limits

The chosen outer audit intentionally leaves a measurable return/censoring
interval. A completed computational schedule therefore does not complete the
preregistered eventual-mass evidence. Much longer fixed-step trajectories
and larger radii, with adequately resolved errors, would be needed to close
it using this method. Accelerating distant excursions by increasing physical
dt or switching to a logarithmic operational clock would change the frozen
maximum-step protocol; neither is silently introduced here.

Some finite endpoint discrepancies can reflect residual Euler/bridge drift
or boundary errors even when two adjacent estimates happen to satisfy the
numerical gate. The independent PDE provides a way to expose these errors;
it does not justify refitting the Brownian process or weakening criteria.
Direct unmodified Geant4 evidence is still missing/invalid, so its verdict
remains REQUESTED. All unresolved requirements remain prerequisites for any
claim of complete R04 validation.
