# R05 serial IRT history convergence

## Preregistration: R05-C1 (2026-11-06)

**Implementation:** in progress. **Evidence:** absent. **Verdict:** not evaluated.
G2 is not passed. This protocol must be committed before the first comparison
sample is generated or inspected. Results will be appended separately, leaving
this protocol unchanged. No Geant4 generator is part of this campaign.

This tests sensitivity to candidate truncation in the Rust serial history
engine, not agreement with a separate many-body physical model. G1-F's
Geant4 11.4.3 pair acceptance permits this integration; it does not validate
the scheduler, replace the 11.4.1 source baseline, or pass R07's later
full-track count gate. R06 production integration remains blocked unless G2
passes all its local requirements.

### Fixed inputs and sampling budget

- Use the unchanged embedded option3 species and all 72 reaction descriptors.
  Every history starts at absolute 1 ps with 30 molecules, species in
  `Species::ALL` order repeated twice. Creation order is never shuffled.
- Use master seed **20261106007** and `RngStreams`. Clustered history IDs are
  **0..65535**; dilute history IDs are **65536..131071**. Each ensemble has
  **65,536 histories**, partitioned into **64 consecutive batches of 1,024**.
  No pilot, optional stopping, sample expansion, seed substitution or endpoint
  removal is permitted within R05-C1.
- Construct positions with that history's `Stage::PhysChem` stream. For each
  molecule, draw three open uniforms in x/y/z order. The **clustered** ensemble
  has three 10-molecule spurs: molecule `i` has center
  `((floor(i/10)-1)*1200, 0, 0)` nm and independent offsets
  `4*(u-0.5)` nm on each axis. The **dilute** ensemble puts every molecule
  independently in `[-1200,1200]^3` nm with coordinates `2400*(u-0.5)`.
  These deliberately synthetic ensembles preserve local clusters or dilute
  separations; neither is an irradiation distribution or a concentration box.
  Space is unbounded, with no reflecting or periodic boundary.
- Run each immutable snapshot through four search policies: all-pairs and
  indexed cutoff multipliers **1, 2, 4**. Reset the `Stage::Chemistry` stream
  to its beginning for each policy. This is paired common-random-number
  sampling, not a promise of identical subsequent draws when candidates differ.
- Use inclusive absolute horizon **1,000,000 ps (1 us)** and right-continuous
  count observations at **10, 100, 1,000, 10,000, 100,000, 1,000,000 ps**.
  Score all **15 mobile species**, with no G-value normalization or reservoir
  counts. Separate existing tests cover pre-IRT and post-contact 1 ps semantics.
- Explicit limits: 100,000 retained arena records, 100,000 pending events,
  100,000 executed reactions and 1,024 requested observations; tracing disabled.
  Any incomplete history, invalid count/statistic or missing batch makes the
  campaign **INVALID**, never a reduced-sample success.

All six ordered comparisons are required in each ensemble, at every time and
for every species: **1/all-pairs, 2/all-pairs, 4/all-pairs, 1/2, 1/4, 2/4**.
The right-hand policy is the reference. This produces **1,080 endpoints**.
Initial candidate counts must demonstrate actual pruning by multiplier 1 in
both ensembles; otherwise the workload has not tested truncation and is INVALID.

### Fixed interval and verdict rules

For each endpoint retain paired batch means `X_b` (left policy), `Y_b`
(reference), and `D_b = X_b - Y_b`. Calculate means and the unbiased sample
covariance over the **64 independent batches**, retaining the pairing. The
constant **2.66** is the conservative two-sided 99% Student-t critical value
rounded upward for 63 degrees of freedom. These are batch-normal approximate
intervals, not exact finite-sample coverage for arbitrary count distributions.

- If the reference mean `y < 0.1` molecule/history, use the absolute interval
  `d +/- 2.66 * sqrt(s_DD/64)`, in molecule/history. The entire interval must
  lie within **[-0.01,+0.01]**.
- Otherwise use a Fieller interval for the relative difference
  `r = (E[X]-E[Y])/E[Y]`. Invert
  `(d-r*y)^2 <= (2.66^2/64)*(s_DD - 2*r*s_DY + r^2*s_YY)`.
  Equivalently solve `A*r^2 + B*r + C <= 0`, where
  `A=y^2-q*s_YY`, `B=-2*d*y+2*q*s_DY`, `C=d^2-q*s_DD`, and
  `q=2.66^2/64`. Require a finite bounded interval (`A>0` and finite real
  roots) wholly within **[-0.01,+0.01]**. This preserves numerator/denominator
  covariance instead of treating an estimated reference mean as exact.
- An interval wholly inside its acceptance band is **PASS**. An interval
  wholly outside on either side is **FAIL** (resolved bias). Every overlapping
  boundary, unbounded relative interval or unresolved precision is
  **INCONCLUSIVE**. Boundary equality is accepted.
- If all paired batch differences are exactly zero, report **INCONCLUSIVE**,
  not a zero-width statistical proof: no observed disagreement does not bound
  unseen rare disagreements. If they are constant but nonzero, report **FAIL**;
  zero estimated uncertainty must not conceal a deterministic discrepancy.
  Non-finite/negative variance, arithmetic overflow or malformed statistics
  is **INVALID**. Numerical roundoff may only be handled by a documented
  algebraically equivalent stable evaluation, not a tolerance change.

Each reported interval is an endpoint-wise 99% interval; no simultaneous 99%
coverage claim is made for the whole table. Global equivalence requires every
endpoint to pass (an intersection-union acceptance rule); a single unresolved
or failed endpoint blocks G2. Campaign precedence is INVALID, then FAIL, then
INCONCLUSIVE, then PASS. Always report all endpoint categories and the failed
or unresolved rows, rather than hiding them behind the aggregate verdict.

Do not silently change multiplier 1, the history algorithm, source policies or
the tolerance after results. Any new campaign requires a separately committed
protocol/version and fresh evidence. A failing convergence gate is scientific
evidence, not permission to weaken an assertion or label ordinary tests as G2.

### Harness verification and retained evidence

`crates/g4-chemistry/tests/history_convergence.rs` will contain deterministic
tests of the comparator: accepted equivalence, injected positive/negative
bias, a mean-compatible but too-wide interval, rare means, the exact 0.1
classification boundary, covariance-sensitive relative intervals, degenerate
samples and invalid inputs. Fixture and search checks must show that both
ensembles actually exercise omissions. These fast checks belong to ordinary
workspace testing.

The complete statistical gate is explicitly invoked with:

```sh
cargo test -p g4-chemistry --test history_convergence preregistered_cutoff_convergence_gate -- --ignored --exact --nocapture
```

It must fail if the scientific verdict is not PASS, while printing the complete
verdict and endpoint table. Its ignored status is a declared long statistical
tier, not passing evidence from `cargo test --workspace`. Record output in
`.scratch/tests/2026-10-29__serial-irt-history-engine/`. Retain compact batch
sufficient statistics (ensemble, batch, policy, time, species, count sum), so
paired covariance and all intervals can be reproduced without keeping full
histories. The harness may print these rows to the captured output. It must
not overwrite any committed reference dataset.

Record the protocol commit, engine base and dirty state, test-file hash,
compiler/target/platform, Cargo.lock and actual species/reaction input hashes,
seed/history mapping, and exact command before interpreting the output. Report
per-policy total candidate visits, attempted pair channels, scheduled/executed
events, maximum pending events, retained arena/index identities, largest
candidate row and peak retained engine payload. Re-run the existing 10,000-H
resource fixture and distinguish its measured engine payload from process RSS.

Required ordinary checks remain:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Source semantics and evidence boundary

The directly inspected Geant4 **11.4.1** file is
`source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc`:

- Constructor lines 75-76 sets
  `R = 1.45 nm + 2*sqrt(8*0.00946 nm^2/ps*absolute_end_ps)`.
  Rust multiplies this entire expression by 1/2/4, yielding approximately
  **551.650 / 1103.300 / 2206.600 nm** at 1 us. This heuristic is not a
  mathematical zero-tail guarantee or the reaction radius.
- `SpaceBinning` (128-158) fixes the initial bounds and cell counts;
  `IRTSampling` (161-174) inserts each track before querying it. `Sampling`
  (189-244) visits whole cells intersecting the newer track's +/-R coordinate
  box, before temporary diffusion of an older partner. There is no spherical
  distance rejection. `FindBin` (344-358) clamps to the original bounds.
- `MakeReaction` (393-508) preserves source-ordered products and inserts them
  into those fixed cells before sampling their reactions. Outside products
  land in edge cells. Rust retains these search semantics, lazy partner
  sampling and existing conditional placement; all-pairs changes enumeration
  only and is not an independent pair-physics oracle.
- Rust's canonical molecule-ID pair order, insertion-order event ties,
  inclusive horizon, retained unary fallback and explicit resource errors
  remain the previously documented intentional scheduler deviations. This
  campaign does not change them, upstream reaction 53's named accounting
  exception, or the unresolved independent physical-law limitations.

The local source version header was checked (`G4VERSION_NUMBER 1141`). The
installed 11.4.3 runtime was not used. The unchanged source-style candidate
default is **1**, pending this evidence; no universal density, horizon or
many-body physical-accuracy claim follows even from a PASS on these fixtures.

## Results

Not sampled. Preregistration commit and implementation are pending.
