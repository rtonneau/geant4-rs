# SYN-2 prospective precision feasibility (2026-11-08)

## Resolution approved 2026-11-09

The user selected the paired/common-input diagnostic redesign while retaining
the cap. The budget blocker is resolved prospectively by the separate mapping
estimand in docs/07 and RR-007; the independent comparison below remains
historical and unpassed. No claim is made that its precision problem disappeared.
The finite grid validates source-transform implementation only, with labelled
raw random primitives and exact branch auditing. Analytic marginal/free-diffusion
checks, physical invariants and the ordinary scheduler trace remain separate.
The old calculation, alternatives and stop record below are preserved.

## Historical stop record

**BLOCKED on a prospective budget/estimator decision.** No SYN-2 producer was
run and no tolerance, sample cap, source mode or reference file was changed.
This is a planning calculation, not an observed scientific failure. RR-006's
source-pair result is unaffected. Production remains unaccepted.

The intended full placement family is channels 1, 2, 11; r/a=1,3;
dt=0,1 ps,1 ns; and +x, +z, translated +x by (10a,-7a,3a).
Thus there are 18 settings per channel. For channels 1 and 2, the two parents
and common product centroid give nine coordinates: nine means and 45 unique
covariances (including diagonal variances), or 54 stochastic endpoints.
Channel 11 has no explicit products in the source table. Its product centroid
is undefined, not zero: six parent coordinates give six means and 21
covariances, or 27 stochastic endpoints. Product count is exactly zero there.
Two independent replicates give 4860 stochastic endpoints plus 108 pair-distance
and 108 product-count identities: **m=5076**. This family includes all within-
and between-parent/product coordinate covariances. The separately planned
free-diffusion analytic family has 16 diffusion coefficients (zero plus all
15 embedded species), four durations, three means and six covariances, two
replicates: 1152 endpoints. It is not needed to establish the blocker below.

For unpaired independent source/Rust variance estimates from an exactly normal
coordinate, each estimate has asymptotic standard error sigma^2*sqrt(2/n).
Their difference therefore has SE approximately **2*sigma^2/sqrt(n)**. An
applicable source coordinate is the y coordinate of a mobile parent in the
positive-duration +z diagnostic: CLHEP's axial construction fixes S1.y=0,
leaving a nonzero Gaussian S2.y contribution. This is a favorable case;
non-Gaussian moments do not acquire a precision guarantee from it.

Under docs/07's alpha=0.001, three looks and m=5076, the two-sided endpoint
alpha is 6.566850538481744e-8. With 30 independent batch estimates on each
side and equal variances, Welch degrees of freedom are 58. At the existing
one-million-pairs-per-setting cap:

| Assumption | Critical value | Expected interval half-width / sigma^2 | Zero-effect expected-width n per side |
|---|---:|---:|---:|
| Welch df=58 | 6.191773873380006 | 0.012383547746760 | 1,533,523 |
| Welch df=29 (one side dominates) | 7.184916444274134 | 0.014369832888548 | 2,064,921 |
| Infinite df, optimistic normal limit | 5.402636348872022 | 0.010805272697744 | 1,167,540 |

Every expected half-width exceeds the frozen **0.01 sigma^2** variance band,
even assuming zero discrepancy. Random estimates of the interval width can
fluctuate; this calculation is not a claim of mathematical impossibility of
a chance PASS. It shows that the approved cap cannot provide a credible
full-family precision plan. Waiting for unusually narrow estimated intervals
or rerunning until lucky would violate the preregistered policy.

The 1,533,523 figure is only the zero-effect expected-width boundary, **not a
high-power sample-size recommendation**. A larger campaign needs an explicit
power/precision margin, batch design, independent moment/pilot estimates for
non-Gaussian coordinates, and resource estimates before registration.
The computation uses SciPy's Student-t and normal inverse survival functions,
independently checked by the new Rust incomplete-beta Student-t helper. The
script and output are `02-syn2-feasibility.py` and `02-syn2-feasibility.log`
under `.scratch/tests/2026-11-08__irt-syn-parallel-checkpoint-direction/`.

Prospective alternatives requiring a decision:

1. Increase the per-setting sample/resource cap and register a power-aware
   campaign with adequate margin; preserve 1% bands and all endpoints.
2. Approve a specifically justified paired/common-input kernel diagnostic or
   separate analytic-reference estimator with a new estimand, multiplicity
   and power analysis. Such a diagnostic cannot silently replace independent
   whole-simulation comparisons or claim the old gate passed.
3. Keep the current contract and cap, retain SYN-2 as open, and defer the
   production milestone. Do not drop coordinates, enlarge tolerances, or hide
   unavailable product centroids to produce a pass.

Partial code is preserved and compiled: an explicit-diffusivity diagnostic
allows one zero-D parent without changing the embedded species table; tests
cover either immobile parent, zero/positive duration and coincident input.
Independent source-moment and Student-t helpers are numerical preparation,
not a completed source-moment consumer. No new SYN-2 mode, campaign or fixture
is represented as ready. The ordinary scheduler-placement trace also remains
unimplemented; the existing zero-duration API is based on static call-path
evidence, not a newly executed trace. Positive-duration source anomalies do
not by themselves imply failure of that dormant ordinary branch.

## Prospective analytic marginal joint-power check (2026-11-09)

This is a new check of the retained independent analytic gate, not a rerun
of the replaced two-population source gate. Draft full family m=6228 includes
5076 placement endpoints and 1152 diffusion endpoints. Fifteen mobile species,
three positive diffusion durations, three coordinate variances and two seeds
supply 270 mutually independent normal-variance estimates under independent
setting streams. All must pass, in addition to other endpoints.

At the full 1m cap and even infinite batch degrees of freedom, the simultaneous
critical value is 5.43919539908 and expected normalized half-width 0.00769218.
Using SE=sqrt(2/n), the normal approximation gives individual equivalence
power 0.897294 and joint power about 1.96e-13 for those 270 variances alone.
At 3000 histories times 300 draws (900k), even 3000 batches give critical
5.45309, width 0.00812899, individual power 0.790561 and joint 2.77e-28.
Thus an expected interval width below 1% does not establish adequate power.
Using 100 batches instead gives still lower power; a low batch count is not
the sole issue. Optimistic normal planning for 95% joint power over only
these 270 variances gives n about 1,682,221, outside the retained cap; this is
not a full-family recommendation or authorization to increase samples.

These are asymptotic approximations with fixed expected interval widths,
not rigorous impossibility bounds. Random estimated interval widths, other
endpoints and their dependence need an actual power design. No marginal
acceptance outcomes have been inspected and no bands, cap or multiplicity
changed. A draft consumer exists but remains unexecuted. Any coupling or
variance-reduced estimator that changes this prospective independent design
needs an explicit validation claim and prospective approval. Exact script
and output: session scratch `02-syn2-analytic-feasibility.py` and `.log`.

The independent-setting allocation above was a draft, not a contractual
requirement. It is superseded prospectively by the README common-input
analytic manifest: same per-history primitive stream across settings,333
draws/history and one batch/history. All marginal endpoints, independent
replicates,1% bands and family alpha stay unchanged. Gaussian diffusion
variances are therefore six distinct streams, rather than270 independent
streams; the earlier joint-power estimate is not a product blocker. Arbitrary
cross-setting dependence is allowed by Bonferroni. The distinct placement
transforms have their own analytic expectations; shared inputs do not assert
that these distributions are equal. No acceptance outcomes informed this
coupling, and no successful full-family gate is predicted.

Observed follow-up under the frozen common-input design: free diffusion and
all zero-duration placement marginals passed at the final999000-draw look.
233 positive-duration source-placement endpoints remained INCONCLUSIVE,
none FAIL. The prospective coupling improved redundant-setting power but
was not a promise of complete acceptance. All three looks are preserved;
no further expansion, new seeds or threshold changes were attempted. This
is a remaining direct-diagnostic precision limit, not evidence of failure
in the statically inferred ordinary zero-duration scheduler path.
