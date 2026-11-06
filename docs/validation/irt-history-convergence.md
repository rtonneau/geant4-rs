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

### R05-C1 result: INCONCLUSIVE; G2 remains blocked

**Implementation:** complete. **Evidence:** preserved in the repository,
auditable without sampling. **Verdict:** **INCONCLUSIVE**, not G2 acceptance.
Protocol commit: `c7188b35bd06420e1ba914c5f404e0c2c2fcfc5b`, made before
sampling. The protocol above is unchanged. No sample expansion, exclusion,
seed change, default-cutoff change or engine modification followed the result.

The fixed campaign completed all **524,288 history executions** (131,072
distinct inputs, four policies each), with **46,080 batch count rows** and all
**1,080 endpoints** present. The explicit statistical test exited **101**, as
required for an unpassed scientific gate: **687 PASS, 393 INCONCLUSIVE, 0 FAIL,
0 INVALID**. Ordinary test success does not override this verdict.

Each table cell below counts 90 endpoints (six times times 15 species).

| Ensemble | Comparison | PASS | INCONCLUSIVE |
|---|---|---:|---:|
| Clustered | 1/all-pairs | 68 | 22 |
| Clustered | 2/all-pairs | 71 | 19 |
| Clustered | 4/all-pairs | 0 | 90 |
| Clustered | 1/2 | 73 | 17 |
| Clustered | 1/4 | 68 | 22 |
| Clustered | 2/4 | 71 | 19 |
| Dilute | 1/all-pairs | 67 | 23 |
| Dilute | 2/all-pairs | 67 | 23 |
| Dilute | 4/all-pairs | 0 | 90 |
| Dilute | 1/2 | 68 | 22 |
| Dilute | 1/4 | 67 | 23 |
| Dilute | 2/4 | 67 | 23 |

Of the inconclusive endpoints, **114** have finite intervals crossing an
acceptance boundary (99 clustered, 15 dilute). Another **279** have no observed
paired differences (90 clustered, 189 dilute), so the registered rule refuses
a zero-width statistical claim. At multiplier 4 the initial span is less than
two cutoff lengths on every axis: all axes have one cell, including products
clamped to its edges. Its counts and resource event/candidate totals match
all-pairs. This geometric fact explains 180 of the zero-difference endpoints;
it does not retroactively exempt them from the protocol.

Examples at **1 us**, multiplier **1/all-pairs**; intervals below are relative
differences in percent, to compare with the fixed **[-1%,+1%]** band:

| Ensemble/species | Indexed mean | All-pairs mean | 99% relative interval |
|---|---:|---:|---:|
| Clustered HO2m | 0.432297 | 0.441940 | [-3.91134%, -0.42808%] |
| Clustered Om | 0.457932 | 0.461014 | [-2.23567%, +0.92768%] |
| Clustered H | 1.284164 | 1.292374 | [-1.36193%, +0.09857%] |
| Dilute HO2m | 0.512146 | 0.514526 | [-2.35524%, +1.46561%] |
| Dilute Om | 0.569443 | 0.569519 | [-1.52705%, +1.52594%] |

The clustered HO2m interval excludes zero but overlaps the acceptable band:
the data do not resolve whether its difference exceeds the allowed magnitude.
The absence of FAIL rows is therefore not evidence that cutoff bias is absent.
The registered endpoint intervals are not simultaneous intervals.

**Coverage limitation:** all observed reference means were at least 0.436234
molecule/history, so no campaign endpoint exercised the rare-mean absolute
branch. Deterministic comparator tests exercise that branch and the exact 0.1
boundary; this is not statistical evidence for rare populations. A future
campaign would need separately preregistered rare-population fixtures and a
precision plan; R05-C1 was not adapted after seeing this limitation. Even if
zero-difference cases were resolved analytically in a future protocol, the
114 unresolved finite intervals would still block G2.

### Measured resource counts

For the first registered input, initial candidate visits in
`[all-pairs,1,2,4]` order were **[435,135,265,435]** clustered and
**[435,96,396,435]** dilute. An empty-descriptor geometric probe measured these
without any chemistry RNG draws or extra sampled chemical histories. Both
ensembles therefore exercised actual omission at multiplier 1.

The following totals include initialization and product scheduling over all
65,536 histories in each row. A candidate visit precedes descriptor matching;
attempted channels include escapes; scheduled events include later stale events.

| Ensemble/policy | Candidate visits | Attempted channels | Scheduled events | Executed reactions |
|---|---:|---:|---:|---:|
| Clustered/all-pairs | 53,085,603 | 23,220,912 | 854,605 | 604,123 |
| Clustered/1 | 16,538,124 | 7,322,621 | 855,454 | 604,580 |
| Clustered/2 | 34,480,819 | 15,466,930 | 855,863 | 605,091 |
| Clustered/4 | 53,085,603 | 23,220,912 | 854,605 | 604,123 |
| Dilute/all-pairs | 46,504,939 | 21,352,503 | 310,593 | 310,363 |
| Dilute/1 | 10,250,615 | 4,816,109 | 310,277 | 310,047 |
| Dilute/2 | 41,523,492 | 19,077,314 | 310,762 | 310,521 |
| Dilute/4 | 46,504,939 | 21,352,503 | 310,593 | 310,363 |

Maxima are per history, not totals or simultaneous allocations across workers:

| Ensemble/policy | Pending events | Arena/index identities | Candidate row | Retained payload bytes |
|---|---:|---:|---:|---:|
| Clustered/all-pairs | 23 | 65 / 0 | 36 | 12,016 |
| Clustered/1 | 24 | 71 / 71 | 14 | 11,792 |
| Clustered/2 | 25 | 65 / 65 | 36 | 12,864 |
| Clustered/4 | 23 | 65 / 65 | 36 | 13,080 |
| Dilute/all-pairs | 8 | 55 / 0 | 36 | 6,704 |
| Dilute/1 | 9 | 56 / 56 | 35 | 8,640 |
| Dilute/2 | 9 | 56 / 56 | 36 | 7,824 |
| Dilute/4 | 8 | 55 / 55 | 36 | 7,256 |

The separately rerun `history_spatial` high-population fixture passed:
10,000 H atoms in 5,000 contacting pairs, 20 nm grid spacing, 0.01 nm pair
separation, 1 ps horizon, multiplier 1, full network and tracing disabled.
It measured **5,000** candidate visits, attempted pair channels, scheduled
events, executed reactions and peak pending events; **15,000** retained
arena/index identities; candidate-row maximum **1**; and **2,630,432 bytes**
peak retained engine payload. Its finite budgets are the same as above.

Payload accounting uses vector capacities and map key/value payloads. It
excludes map-node spare slots/links, allocator overhead, stack, transient
placement, caller-owned input/network and observation output. It is a measured
allocation-payload lower bound, **not process RSS**; this campaign did not
remeasure process RSS. No runtime scalability or full-track workload claim is
made from these synthetic fixtures.

### Provenance, artifacts and verification

Run started **2026-11-05 23:53:22 UTC** (2026-11-06 local Europe/Brussels), with
engine base `f62e1e66df578ec194ba208248872b92a3c42e71`, HEAD at the protocol
commit, and an uncommitted test harness. Platform: Windows 10.0.26100,
`x86_64-pc-windows-msvc`, rustc **1.97.0 (2d8144b78 2026-07-07)**,
LLVM 22.1.6, optimized test profile. Histories ran through a test-only Rayon
batch wrapper; each history stayed serial and batch output/reduction order was
fixed. Exact bytes used by the compiled test were hashed in its output:

| Artifact | SHA-256 |
|---|---|
| Preregistered document bytes | `958d596308a725a43a9f67c0135c7ce939c620370a6fb471267945c53e0013ab` |
| `history_convergence.rs` | `f26918299b0e05c2ac1f1a03d116037478e9b256d397c6af2877083ab2e6a1e8` |
| `Cargo.lock` | `469fb57fc7b52930c82e2730f98bab13911863df80826c7394c678027fbdbe1d` |
| Embedded species bytes | `4d727b7e87b5696889d4b7ad9dc69814cfe65200593f7f1d2fc5a9dbef965f87` |
| Embedded reaction bytes | `8a992acb1f9973f84fa5149528a6599010cd270895fa6eef4682136babf2888d` |
| Inspected 11.4.1 `G4DNAIRT.cc` | `cdd2bec595262ac672f4d5d5f4b15b28d04f06d11579aca254692ca6e23b4686` |
| Full campaign log | `39a826333dd794b8c4f5534a67d7d18b042092018be347817ca9550a60865e2f` |
| Extracted endpoint table | `3e71fe4d94765b3c7a9384762290fcc8ae5e8daaaeb145a871a94045122c9d890` |

The compact evidence bundle is
[`validation/irt_history/evidence/r05_c1/`](../../validation/irt_history/evidence/r05_c1/).
Its `batches.csv` contains **3,072 rows**, each retaining all 15 integer species
count sums for an ensemble/batch/policy/time, plus the 1,024-history denominator.
These are the same **46,080 sums** printed in the original campaign, grouped
without numerical transformation. `endpoints.csv` retains all **1,080** original
means, intervals, verdicts and reasons, including every unresolved endpoint.
`provenance.json` records the original log identity, protocol/implementation
revisions, dirty state, inputs, command, compiler/platform, policies, resource
records and the archived artifact identities. This is a lossless extraction
from the original hash-verified log after review; **no resampling** took place.

| Archived artifact | UTF-8/LF bytes | SHA-256 of UTF-8/LF bytes |
|---|---:|---|
| `batches.csv` | 307,340 | `1620d29286313055a39c6ebfb8995dd22066bbc0dc9e596d5dddadfc2b64c452` |
| `endpoints.csv` | 140,750 | `53a130820d9cf56155e01be58b47c3a1871538f6143f1a8bb54c36cf64c9bd04` |
| `audit_r05_c1.py` | 7,494 | `795521da3a3565cacd51cb6d28709f261986c66de643682ae9856012516ff95e` |

A fresh checkout audits these files with **Python 3.10+**, standard library
only, without rebuilding Rust, fetching data or running any stochastic history:

```sh
python validation/irt_history/audit_r05_c1.py
```

The script checks sizes/hashes (normalizing Git's CRLF to LF), exact schemas
and complete unique grids, integer populations, paired means/covariance and
every reported interval/verdict. Audit exit **0** establishes evidence integrity
and reproduction of the recorded **INCONCLUSIVE** result; it does **not** pass
G2. Missing, malformed, altered or inconsistent evidence exits nonzero.

The original 2,171,398-byte campaign log and diagnostic logs remain preserved
under `.scratch/tests/2026-10-29__serial-irt-history-engine/`, but are optional:
the archived audit does not read scratch files. Re-executing the Rust gate
above remains available separately; cross-platform bitwise agreement is not
claimed for new simulation runs.

The comparator evaluates the registered Fieller quadratic through an
algebraically equivalent regression of `D` on `Y`. Computing the nonnegative
residual sum of squares avoids cancellation in nearly proportional samples;
no variance clipping or changed threshold is used. An independent 60-digit
Python `Decimal` audit of the captured batch sums directly solved the original
quadratic. The archived script verified all 46,080 sums and all 1,080 verdicts;
the largest interval-endpoint difference was **2.88e-16**. A relocated standalone
bundle and CRLF-converted copies passed the audit. Altered bytes, missing
batches, negative counts, duplicate endpoints and changed verdicts were
rejected; structural/numerical checks also rejected copied malformed fixtures
after their hashes were updated. Original evidence was not changed.

The six deterministic harness tests passed. Required formatting, workspace
Clippy with warnings denied, and workspace tests passed. The separately
invoked statistical gate failed with the INCONCLUSIVE verdict above, as
designed. No Geant4 generator was run. The production implementation, reference
CSVs and provenance files were not modified.
