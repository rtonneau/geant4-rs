# R04 independent Brownian oracle convergence

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

## Source-matched Geant4 IRT comparison (RR-004)

The R04 Geant4 verdict is **REQUESTED**. The user-run
`pair_matrix_irt_r04_n100000.csv` and its provenance have not been committed.
The ignored consumer `crates/g4-chemistry/tests/geant4_pair_r04.rs` rejects
missing, duplicate or unordered rows; incorrect option3 parameters, grid,
counts, kernel settings, completion marker, and SHA-256 or run provenance.
It compares independent Rust seed 20261027403 with Geant4 seed 20261027404,
using 100,000 pairs per case and distance. No Geant4 output is inferred from
the analytic fixture or from the Rust sampler.

The consumer currently checks that the channel comment contains two nonempty
reactant names, but does not verify their identities. The generator writes
`G4MolecularConfiguration::GetName()`, whose value is not necessarily the
chemistry-table label. A source-verified mapping is needed before the consumer
can strictly validate these names. The consumer also hardcodes the initial
100,000-pair filename, count and command: it cannot yet consume the registered
precision-expansion runs without further development. Neither limitation
authorizes accepting an incomplete or insufficiently precise result.

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
