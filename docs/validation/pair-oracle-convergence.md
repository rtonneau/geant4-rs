# R04 independent Brownian oracle convergence

## Current continuation (2026-10-29)

The current method and evidence are in
[the continuation report](r04-independent-2026-10-29.md) and
[the charged finite-time PDE report](pair-coulomb-pde.md). The PDE now supplies
90 finite-time physical anchors, and trajectory stopping is connected to a
censored eventual endpoint with simultaneous population Monte Carlo bands.
The general eventual endpoint is still **INCONCLUSIVE**. Exact charged TDC
contact also falsifies physical agreement of the effective-distance law;
that resolved counterexample does not complete numerical convergence of the
remaining endpoints. Analytic agreement and direct Geant4 evidence remain
separate verdicts. G1 cannot pass and R05 is not advanced.

The sections below retain the **historical** initial implementation and runs.
Statements there about absent PDE anchors, disconnected escape helpers and
the old charged upper-bound-only output describe that earlier state, not the
current `pair_oracle/experiment.rs` harness. No historical data are rescored
or overwritten by the continuation.

## Historical initial implementation and evidence

The neutral oracle currently reports **INCONCLUSIVE**. This is executable
numerical evidence, not closure of the Brownian part of G1. A successful Rust
test means the experiment and verdict checks ran; it does not mean scientific
agreement passed.

## Independent algorithm

`tests/pair_oracle/neutral.rs` integrates the relative radial diffusion in
dimensionless units a=D=1: `dr = (2/r) dt + sqrt(2) dW + dL`, outside r=1.
It uses Euler frozen radial drift and the exact conditional minimum of the
one-dimensional Brownian bridge during each step. Negative minima produce a
Skorokhod reflection increment L. TDC absorbs at the first bridge contact;
PDC kills when L exceeds an independent exponential threshold of rate kappa.
Contact TDC is immediate; contact PDC accumulates local time from zero.
Reaction is recorded at the step endpoint, with every observation landing
exactly on a step boundary. The drift, reflection and reaction-time placement
remain discretization errors that require refinement. There is no production
diffusion, SBS, IRT or Gaussian sampling call. Only the general history RNG is
shared; Box–Muller normals and bridge uniforms are generated here.

The neutral boundary coefficient is `kappa=f/(1-f)`, where f=kobs/kdiff.
Here kdiff includes the source A+A half factor, while relative motion uses the
sum of both diffusivities. Thus the synthetic neutral PDC f=0.5 has kappa=1.
Inputs come from the independent analytic fixture; rates are converted using
the exact Avogadro constant. Geant4 11.4.1 source checked for conventions:
`source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc:298-339` and the
reaction-parameter provenance described in `pair-case-matrix.md`. The oracle
does not copy the source's independent-reaction-time sampler.

## Precision, refinement and unresolved tail

Seed 20261027402 and history ID
`((case*3+distance)*5+level)*1000000+pair` make levels independent and preserve
prefixes during sample expansion. All six neutral cases and three distances
run at least 10,000 pairs and the first three frozen h levels. Samples grow
only through 10k, 20k, 40k, 80k, 160k, 320k, 640k, 1m. Refinement uses only
1/16, 1/32, 1/64, 1/128, 1/256. Three combined SE must be below 0.005 before
precision is called adequate; half-count stabilization prevents zero SE for
unobserved rare events. Both absolute change <0.005 and compatibility within
three combined SE are required. Sample expansion depends only on that width,
not on significance. At least three levels run even for deterministic contact.

The default runtime budget is 10k pairs and three levels. Set
`PAIR_ORACLE_MAX_PAIRS=1000000` and `PAIR_ORACLE_LEVELS=5` to allow the entire
precommitted schedule. A runtime budget ending before adequate precision or
refinement explicitly reports INCONCLUSIVE. This budget is computational,
not a change to the scientific acceptance criteria.

Survivors at tau=100 remain unresolved: they are **not** eventual nonreactors.
No analytic tail continuation is counted as independent Brownian evidence.
The eventual endpoint and hence the complete case remain INCONCLUSIVE even
if all finite endpoints converge. Independent eventual-mass resolution and
fully precise finite-time runs remain necessary before Brownian G1 can pass.

## Reproduction and retained evidence

Run `cargo test --release -p g4-chemistry --test pair_brownian neutral -- --nocapture`.
Every case/distance/sample size/level/observation prints counts, unconditional
CDF, independent anchor, last-level difference and three-SE width. Separate
rows retain unresolved survivor counts and the final scientific verdict.
The initial run log is retained under
`.scratch/tests/2026-10-27__pair-oracles-and-reference-gate/evidence/ticket03-neutral.log`.
No reference data file or Geant4 generator is changed or run by this experiment.

Initial run (2026-10-27), n=10,000 at each of h=1/16, 1/32, 1/64:

| Case | Survivors at tau=100, h=1/64 (r/a=1, 1.5, 3) | Complete verdict |
|---|---|---|
| option3-1 | 0, 3519, 7094 | INCONCLUSIVE |
| option3-2 | 0, 3498, 6931 | INCONCLUSIVE |
| option3-11 | 4706, 6647, 8432 | INCONCLUSIVE |
| option3-17 | 2650, 5229, 7821 | INCONCLUSIVE |
| syn-neutral-tdc-ab | 0, 3569, 6990 | INCONCLUSIVE |
| syn-neutral-pdc-aa | 5152, 6843, 8542 | INCONCLUSIVE |

Only the three immediate TDC contact finite-time series met the finite
convergence/precision test at this budget. The raw log retains all 270
finite-time endpoint records, including all intermediate levels; the table
does not replace them. Two Rust tests passed, including rejection of
underpowered equal estimates and an outside-limit difference.

## Charged physical oracle

`tests/pair_oracle/charged.rs` integrates the physical Coulomb potential
`U/kT=c/r`, where `c=rc/a` is signed: positive is repulsive. The radial
backward generator is `d²/dr² + (2/r+c/r²)d/dr`; thus the Euler drift includes
both radial entropy and the Coulomb force. It uses the same independently
implemented bridge minimum, reflection/local-time boundary, observation
alignment, seed and frozen schedules as the neutral experiment. It never
maps distance through the IRT effective-distance approximation. The six
charged cases include literal and synthetic A+A, attractive and repulsive
TDC/PDC. A+A changes the macroscopic rate convention, never the relative
particle diffusion coefficient.

The physical boundary is calibrated to the preregistered macroscopic rate
fraction `f=kobs/kdiff` (including the A+A half factor in both rates):
`kappa = f/(1-f) * c/(1-exp(-c))`. TDC absorbs at contact. This gives the
exact eventual reaction probability
`u(r)=f*expm1(c/r)/expm1(c)`, with f=1 for TDC. It follows by solving
`u''+(2/r+c/r²)u'=0`, `u(infinity)=0`, and the radiation backward condition
`u'(1)=kappa*(u(1)-1)`. These equations also document the boundary convention
independently of Geant4's charged alpha, whose A+A factors need not give this
same physical radiation boundary.

The new `pair_coulomb_anchors.json` fixture records drift and eventual mass
computed independently with mpmath at 80 and 160 digits; rounded results must
agree. Regenerate with `analytic.py --physical --output
crates/g4-chemistry/tests/fixtures/pair_coulomb_anchors.json` using the Python
setup documented above. Existing approximation anchors remain unchanged.
Executable checks compare drift and boundary to these anchors, test both
force signs, immediate absorbing contact and exact zero-charge recovery of
the neutral integrator with identical streams.

Every timestep prints the physical eventual upper bound and the separate
Geant4 finite-time CDF. A resolved violation of the physical upper bound is
FAIL, using the frozen 216-endpoint Bonferroni threshold and 0.005 effect
limit. Exact eventual physical-versus-approximation differences >=0.005
are separately FAIL. Smaller exact eventual differences are labelled
PASS(eventual-only), never a complete physical PASS. Finite-time Coulomb CDF
anchors have not been solved here: satisfying the eventual upper bound is
not finite-time agreement, and is explicitly INCONCLUSIVE. Survivors at
tau=100 are unresolved, not eventual nonreactors. Thus G1 remains OPEN even
where refinement converges; full finite-time physical-law validation and
independent tail resolution are still required.

The initial charged run used n=10,000 and three levels and passed both
executable tests. The complete raw refinement and verdict log is retained at
`.scratch/tests/2026-10-27__pair-oracles-and-reference-gate/evidence/ticket04-charged.log`.
Run `cargo test --release -p g4-chemistry --test pair_brownian charged -- --nocapture`.
The source conventions were checked directly against Geant4 11.4.1
`source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc:286-356`.
No Geant4 generator or simulation was run.

Continuation 2026-10-29: a run capped at 20,000 pairs and five levels was
interrupted before completing the family after saturating the machine. The
partial log is retained locally as `continuation-brownian-20k-5levels.log`
beside the earlier evidence; it has no completion marker and does not establish
a family verdict. Some finite endpoints remain underpowered at 20,000 pairs,
and surviving trajectories remain unresolved at the eventual endpoint. The
full frozen precision and refinement schedule has not been completed.

### Exploratory continuation: finite-time PDE and escape bound

An exploratory calculation solved the backward equation `u_t=u_rr +
(2/r+c/r^2)u_r` for reaction probability on `1 <= r <= R`, with initial
`u=0`, absorbing contact `u(1,t)=1` or radiation boundary
`u_r(1,t)=kappa*(u(1,t)-1)`, and `u(R,t)=0`. The finite-domain calculation
uses centered spatial differences and backward Euler with geometrically
increasing time steps, independent of the Brownian paths. This is exploratory,
not a registered anchor or a completed Brownian comparison. A supporting
discussion of the backward equation and Robin boundary is in [Grebenkov,
Diffusion-Controlled Reactions: An Overview](https://pmc.polytechnique.fr/pagesperso/dg/publi/2023_07.pdf).

For zero charge the fine numerical result differs by at most 0.000167 from
the synthetic neutral TDC analytic fixture and 0.000336 from the PDC fixture.
Across the six charged cases and all fifteen finite-time points per case,
the difference between a coarse run (`dr=0.04`, `R=100`, maximum `dt=0.05`,
relative time step 0.01) and a fine run (`dr=0.02`, `R=120`, maximum
`dt=0.025`, relative time step 0.005) is at most 0.000517. For option3-9 and
synthetic charged PDC, increasing `R` from 100 to 120 at fixed fine grid
changes no displayed floating-point value; further halving `dr`, maximum
`dt`, and relative time step at `R=100` changes results by at most 0.000093
and 0.000133 respectively. These checks do not constitute a rigorous error
bound or validate all cases and boundary treatments. The exploratory script
and output are in the local ignored scratch evidence directory as
`explore_coulomb_pde.py` and `explore_coulomb_pde.log`; no new fixture or
scientific verdict depends on them.

The finite-domain method is now also implemented in
`validation/pair_oracles/coulomb_pde.py`, with neutral TDC and PDC anchors and
two-sign refinement checks in `test_coulomb_pde.py`. Run
`python -m unittest discover -s validation/pair_oracles -p test_coulomb_pde.py`
with NumPy and SciPy installed. This validates basic limits, but has not
produced validated charged anchors for the complete family or been connected
to the Brownian verdict.

A separate eventual-mass strategy is to continue each path until it reacts or
reaches an outer sphere of radius R, and bound any later reaction after escape
by the probability of ever hitting contact from R. For the continuum radial
Coulomb process this upper bound is `expm1(c/R)/expm1(c)`, with limit `1/R`
for zero charge; partial absorption cannot increase it. For example, `c=-1`
and `R=400` gives an upper bound about 0.00396. Such paths remain interval
censored, not observed nonreactions. The bound alone does not validate the
Euler process's outer-crossing detection or provide its discretization error;
both must be addressed, and the bound must be included in the endpoint's
uncertainty under the existing convergence and precision rules. The required
long excursions also make the frozen small-step schedule expensive. No tail
classification, full refinement schedule, or Brownian PASS is claimed.

The helper `tests/pair_oracle/escape.rs` implements the continuum return
probability and the corresponding interval for the conditional expected
reaction fraction, retaining both escaped and unresolved counts. This is not
a pathwise confidence interval: sampling uncertainty and crossing errors must
be accounted for separately. It is not yet connected to trajectory stopping,
and does not resolve the Brownian eventual endpoint by itself.

## Source-matched Geant4 IRT comparison (RR-004)

The R04 Geant4 verdict is **REQUESTED**. The user-run
`pair_matrix_irt_r04_n100000.csv` and its provenance have not been committed.
The ignored consumer `crates/g4-chemistry/tests/geant4_pair_r04.rs` rejects
missing, duplicate or unordered rows; incorrect option3 parameters, grid,
counts, kernel settings, completion marker, and SHA-256 or run provenance.
It compares independent Rust seed 20261027403 with Geant4 seed 20261027404,
using 100,000 pairs per case and distance. No Geant4 output is inferred from
the analytic fixture or from the Rust sampler.

### Exact-contact investigation (2026-10-29)

At neutral PDC contact, the registered analytic CDF has `b=0` and hence
`F(t)=W_inf * [1-erfcx(A*sqrt(D*t))]` for positive time, with eventual
reaction probability `W_inf`. It is continuous as `b` approaches zero from
positive separations, and `F(0+)=0`; this is a finite-time radiation-boundary
limit, not a zero-time contact reaction. The physical radiation boundary is
discussed in Grebenkov, [Diffusion-Controlled Reactions: An Overview](https://pmc.polytechnique.fr/pagesperso/dg/publi/2023_07.pdf).
The same algebraic limit applies to the source's charged-PDC expression when
its computed `b` is zero, but it remains the source's electrostatic
approximation, not independently validated Coulomb dynamics.

A separate generator could sample the acceptance gate and invert this
conditional limiting CDF, label the substituted rows and retain exact contact.
It would need independent numerical tests, a new mode and provenance, and a
prospective validation showing continuity and agreement with direct Geant4
away from the singularity. It cannot be presented as an unmodified Geant4
return or retroactively treated as the registered direct-source fixture. No
such corrected fixture or replacement criterion is claimed here.

The physical eventual anchor already exposes discrepancies independently of
the unrefined Brownian trajectories. For option3 ID 8 at r/a=1.5, the physical
eventual reaction probability is 0.2388043791 while the source-law anchor is
1; for ID 9 at r/a=1, they are 1 and 0.7180721753. These compare distinct
model laws and are not a completed Brownian convergence result. The numerical
Brownian paths and charged finite-time law still require independent
resolution; G1 remains open.

The consumer verifies source-ordered `G4MolecularConfiguration::GetName()`
reactant identities, and supports the registered 100k through 1m run files
with their corresponding provenance. This does not authorize accepting an
incomplete or insufficiently precise result.

The frozen family comprises 8 literal channels × 3 distances × 6 endpoints =
144 comparisons, with two-sided family alpha 0.001, pointwise Bonferroni
threshold `z = 4.495384196564731`, and absolute effect limit 0.01. The
standard error is the sum of two independent binomial variances; a half-count
stabilization prevents zero variance for an unobserved rare endpoint. PASS
requires both `|z| <= zcrit` and `|difference| + zcrit*SE < 0.01` at every
endpoint. A resolved effect outside the limit or a significant difference
within a precise interval is FAIL. If the adjusted interval does not fit
strictly inside the effect band, the result is INCONCLUSIVE and only the
preregistered complete-run expansion sequence (200k, 400k, 800k, 1m) can
resolve it. At probability 0.5, 100k per implementation has an adjusted
half-width slightly above 0.01 even with identical estimates; a successful
100k run need not be sufficient evidence.

Verdicts remain separate: **ANALYTIC** is the 216-endpoint
`pair_analytic.rs` test; **BROWNIAN** is the independent refinement experiment
above and remains **INCONCLUSIVE**; **GEANT4** is **REQUESTED** until complete
reference evidence is available. Consequently **G1 is OPEN**. The existing
V9 and RR-003 tests retain their historical criteria and verdicts.
