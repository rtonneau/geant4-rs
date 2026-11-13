# G. Validation strategy

> **Prospective production update (2026-11-08):** IRT-syn uses the separate [SYN gates below](#irt-syn-prospective-gates-2026-11-08). Historical V/G/RR criteria and results remain unchanged. Option3 IRT agreement cannot be used as IRT-syn evidence.

> **Approved update (2026-11-05):** [G1-F source-fidelity decision](validation/g1-f-source-fidelity.md) prospectively replaces the original combined G1 as the R05 real-model integration prerequisite. G1-F remains OPEN; the original G1 was not passed. Historical criteria and evidence below remain unchanged. R05 infrastructure and deterministic synthetic tests may proceed before G1-F; real chemical model integration may not.

Four levels (never conflated; reports state which level a claim has):
L1 mathematical/algorithmic unit tests; L2 numerical comparison with Geant4 algorithms (same inputs, deterministic quantities: cross sections, CDF tables); L3 statistical comparison with Geant4 simulations (distributions); L4 comparison with published/experimental data. **Agreement at L2/L3 does not establish L4.**

## G.1 Reference data generation
Offline, by a developer with Geant4 11.4.1 + G4EMLOW 8.8: small C++ apps in `validation/geant4/` (read-only use of the reference tree; built outside it) derived from examples `mfp`, `range`, `slowing`, `spower`, `dnaphysics`, `chem6`. Outputs: CSV + `provenance.toml` (Geant4 version/SHA, dataset version+MD5, physics constructor, seed, number of histories, date, command line) in `validation/reference/`. Size budget: < 1 MB per fixture, < 20 MB total. CI never builds Geant4.
Pre-registered acceptance criteria below are fixed **before** the first comparison and changed only via a documented commit.

## G.2 Tests and criteria
| ID | Quantity | Level | Criterion |
|---|---|---|---|
| V1 | Table loading, units, boundaries | L1 | exact table nodes returned to 1e-12 rel; out-of-range -> typed error |
| V2 | Total cross sections vs Geant4 (`G4VEmModel::CrossSectionPerVolume`) at nodes and 200 log-spaced off-node energies per model | L2 | nodes 1e-9 rel; off-node 1e-6 rel (same log-log law); any larger deviation is investigated, not tolerated |
| V3 | Interaction-type probabilities at 20 energies | L2 | 1e-9 rel |
| V4 | Sampled secondary energy / scattering angle distributions per model at 10 energies, 10^6 samples | L3 | two-sample KS and chi2 (equal-probability bins) p > 0.001 each; with 10 energies x 3 observables x models, expect false alarms at p<0.001 for ~1 in 1000 tests; use Bonferroni over the test family for the overall verdict |
| V5 | Mean free path, track length, CSDA-like range (electron 100 eV-1 MeV; proton within the model ranges only) | L3 | difference of means within 3 combined standard errors AND < 2 % absolute (relative systematic tolerance) |
| V6 | Stopping power / W-value | L3, L4 | L3 as V5; L4 vs ICRU/NIST ESTAR/PSTAR liquid water: report, no pass/fail until Geant4's own deviation (known) is tabulated |
| V7 | Energy conservation | L1 | per history |E_in - sum(parts)| < 1e-9 eV-relative; 10^4 histories at each energy |
| V8 | Physchem: species yield per event type, displacement RMS | L1, L3 | channel branching: chi2 p>0.001; displacement RMS within 3 sigma of nominal |
| V9 | Chemistry kinetics single reaction (A+B, A+A) in a box vs analytic Smoluchowski / Collins-Kimball | L1 | within 3 sigma of the analytic curve, N>=10^4 pairs |
| V10 | Well-mixed limit vs ODE | L1 | long-time agreement 1 % for a closed-system test case |
| V11 | G(t) for e_aq, OH, H, H2, H2O2 at {1 ps, 10 ps, 100 ps, 1 ns, 10 ns, 100 ns, 1 us}, e- 1 keV..1 MeV and p, comparing to Geant4 chem6 (IRT) run with identical reaction list | L3 | per species and time |G_rs - G_g4| < 3 sqrt(s_rs^2 + s_g4^2) at every point and Bonferroni-corrected family p > 0.01; additionally relative systematic < 5 % at t >= 1 ns for N_hist chosen so statistical error < 2 % |
| V12 | G(t) vs published (e.g. J. Appl. Phys. 125 (2019) 104301 data cited by chem6; Elliot/Bartels e_aq data) | L4 | report with literature uncertainty; no pass/fail threshold set a priori beyond "documented" |
| V13 | Thread-count / seed reproducibility | L1 | identical results for 1, 2, 8 threads (same platform); different seeds -> statistically compatible |
Statistical details: batch means (>=30 batches) for correlated scores; Geant4 reference errors from its own batch means; independent-seed repeat of each reference run to estimate its run-to-run scatter; report both z and effect size. No result is called "scientifically valid" from L2/L3 alone.

**M2 implementation notes.** V7 and V13 run in `cargo test` (ledger: 10^4 histories at five energies, about one minute with `[profile.test] opt-level = 3`). V2-like and V4 run against the raw data files and an independent Python re-implementation (`scripts/hand_cross_sections.py`), not against Geant4. V2/V3/V5 against Geant4 are in `crates/g4-sim/tests/geant4_reference.rs`, `#[ignore]`d until `validation/reference/*.csv` and `provenance.toml` exist (`validation/geant4/README.md`); their criteria are stated in that file's header. Performance baseline: `crates/g4-sim/tests/baseline.rs` (`--ignored`).

**M3 implementation notes.** V7 now runs on the whole cascade (normal run 10^4/10^4/2000/200/20 histories at 100 eV, 1, 10, 100 keV, 1 MeV, about 1 min; the `--ignored` full run with up to 10^4 histories and 200 at 1 MeV takes 5 min and was executed once: max residual 5.7e-16 ... 2.5e-13). V13 additionally pins 1/2/8-thread bitwise equality of spectra, radial profile and the physicochemical-event stream. The new cross sections are checked V2-style against `scripts/hand_cross_sections.py` (Sanche, Melton, `r_mean`), V4-style against the raw tables (Sanche levels, Melton absorption fraction, solvation displacement; families and alphas in `crates/g4-physics/tests/low_energy.rs`). Physical sanity (not Geant4): `crates/g4-sim/tests/physical_sanity.rs`, thresholds written in docs/08 before the numbers were seen. Geant4 comparison of the cascade: `crates/g4-sim/tests/slowing_down_reference.rs` (ignored until `validation/reference/{xs_lowenergy,slow_sums,slow_spectra,slow_radial}.csv` and `provenance_m3.toml` exist); criteria in its header: V2 1e-9 / 1e-6, V5 family with z < 4 (77 comparisons, family-wise false-alarm 0.5 %; docs/07's 3 sigma is for a single quantity) and the 2 % relative cut applied only where the combined standard error is below 0.7 %, two-sample chi-square for the spectra (alpha 1e-3/28), radial cumulative fractions with z < 4.

**M7 local-oracle notes.** V9 now includes a validation-only, source-faithful SBS path for option3 H+H at the six registered dimensionless times. The original pointwise rule is the combined three-sigma binomial bound for two 100,000-pair curves. Source-faithful Rust SBS exceeds that bound only at tau 0.25; this discrepancy is locked as a regression rather than hidden by changing Geant4's executable bridge semantics. V10 uses 512 fixed-seed exact Gillespie histories for option3 reaction ID 2 and a pinned-`rsoda` LSODA solve at 360 us; its four mean populations agree within 1% (maximum observed error 0.0824%), with conservation checked per history. These are L1 local oracle results. The L3 Geant4 H+H comparison is separately preregistered in RR-003 and remains **awaiting data**; no external pass is claimed.

**R04 preregistration (2026-10-27).** The [isolated pair case matrix](validation/pair-case-matrix.md) fixes eight literal option3 channels and four distinctly synthetic dimensionless cases before new statistical runs. It fixes the three starting separations, five positive observation scales, eventual no-reaction endpoint, independent seeds, 0.001 Bonferroni families, pair-count and timestep schedules, <0.005 Brownian convergence rule, <0.01 Geant4 effect limit, precision requirements and INCONCLUSIVE handling. R04 analytic, independent Brownian and Geant4 verdicts are separate; G1 requires all three to pass. This prospective R04 contract does not alter historical V9 or RR-003 criteria or evidence.

## G.3 Regression fixtures
(A *fixture* is a small stored input or expected-output file that a test compares against, e.g. a CSV of Geant4 reference values or a saved result of a fixed-seed run. Not the same as the `data/` cross-section tables, which are simulation inputs.)
Deterministic tiny runs (fixed seed, 10^3 histories) store summary scalars to 1e-12 relative in `tests/fixtures/`; intentionally regenerated via `cargo xtask`-like script with a changelog entry whenever model or RNG consumption changes.

## G.4 CI (GitHub Actions)
- `ci.yml` runs on pushes and PRs on Linux and Windows with pinned Rust 1.97.0. Each job runs `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked` and `python scripts/run_validation_gates.py --tier required`. The required tier checks the committed G4EMLOW data and reference fixtures against the versioned `validation/gates.json` inventory. Missing or altered required files fail preflight. Platform results remain unverified until the respective jobs run.
- The maintained statistical selection is manual: `python scripts/run_validation_gates.py --tier statistical`. It runs four M2 comparisons, two M3 comparisons and the M6 RR-001 pre-IRT 1 ps comparison after the same evidence preflight. The inventory names historical exclusions: post-zero-time M6 comparisons and exploratory or scratch-producing diagnostics. A blanket `--ignored` run is not the maintained tier. There is no scheduled `statistical.yml` or `bench.yml` workflow at present.
- M6 closure remains qualified by its recorded limitations. RR-002 reached its preregistered precision target and did not resolve a primary spatial RMS difference; this does not establish equivalence. Its CSV/provenance remain local-only in `.scratch/rr-002-80/`, outside the committed required tier. Details and numerical results are in `docs/08-roadmap.md`.

## IRT-syn prospective gates (2026-11-08)

### Approved paired-diagnostic amendment (2026-11-09)

The user chose: "Keep the current cap; redesign the diagnostic using paired
common random inputs and separately justify what it validates." The historical
independent source/Rust SYN-2 moment comparison is **not passed**. Its budget
calculation remains in `validation/geant4/SYN2_FEASIBILITY.md`.

RR-007 prospectively replaces only the independent source-versus-Rust
placement-moment comparison with an
exhaustive finite-input mapping diagnostic: 16 labelled primitive tapes across
75 scenarios (the 54 original placement settings, 12 one-immobile-parent
settings, nine mobile-coincident error settings). The manifest and exact input
recipe are in the Geant4 README. Compare 8064 parent/product coordinate outputs
using `abs(delta) <= 1e-12 nm + 1e-12*abs(reference)`; branch draw masks,
product count, parent-removal and expected error outcomes are exact identities.
Zero-product centroids are undefined, never invented zero measurements.

Both implementations receive the same explicit standard-normal x/y/z and
azimuth/polar/product uniform primitives, with semantic labels and consumption
checks. Equal seeds for different RNG algorithms do not meet this requirement.
The C++ adapter compiles the SHA-pinned source body unchanged under a renamed
class, intercepting only the named RNG call sites; Rust uses its independent
transform. No transformed C++ output is supplied as Rust input. This deterministic
finite-grid comparison has no stochastic confidence interval or multiplicity
correction; every registered row and coordinate must pass. It validates only
the mapping on the registered inputs, not RNG quality, a continuous-input
distribution, whole-scheduler behavior or independent physical correctness.

The 1% analytic marginal/free-diffusion criteria, physical invariants and
ordinary-scheduler placement trace remain separate requirements. Their stochastic
families retain alpha .001, three looks and the actual full endpoint count.
The one-million-samples-per-setting, two-hour/eight-GiB invocation and 24-hour
gate budgets remain unchanged. No historical source mode, fixture or tolerance
is amended retroactively. A source-faithful dormant-branch anomaly remains
labelled diagnostic and is not activated in ordinary Rust placement.

These gates are preregistered requirements, all **OPEN**, for the approved
IRT-syn production direction. They supplement rather than rescore V1-V13,
G1/G1-F/G2 and RR-001 through RR-005. Source semantics, exclusions and the
11.4.1/11.4.3 distinction are fixed by the
[reference contract](../validation/geant4/README.md#irt-syn-reference-contract-2026-11-08).
L1 algorithmic, L2 deterministic source, L3 statistical source and L4 physical
evidence must remain separate verdicts. In particular, source placement quirks
may pass L2/L3 while failing physical invariance; that cannot close production.

### Endpoint families and acceptance

| Gate | Frozen endpoints and cases | Acceptance |
|---|---|---|
| SYN-1 pair law | Literal option3 channels 1, 2, 8, 9, 11, 17, 42, 51 under the IRT-syn constructor; separations r/a = 0.5, 1, 1.5, 3; CDF at tau = D*t/a^2 = 0.01, 0.1, 1, 10, 100 and sampler no-event mass (192 endpoints). Separate initial-overlap and post-start diagnostic labels; a horizon-censored sample is not an eventual no-event. | Source comparison: simultaneous difference intervals entirely inside +/-0.01 absolute probability. Analytic type-0 CDF/no-event separately uses the same bound; type-1 delay mixture needs an independently integrated oracle before implementation acceptance. Source errors are diagnostic outcomes, never zero counts or excluded rows silently. |
| SYN-2 diffusion and placement | Free diffusion at D = 0 and each embedded mobile-species D; dt = 0, 1 ps, 1 ns, 1 us. Pair placement for channels 1, 2, 11 at r/a = 1 and 3; dt = 0, 1 ps, 1 ns; orientations +x, +z and translated by (10a,-7a,3a). Record each coordinate mean, variance, cross-covariance, pair distance and product centroid; zero-D partner and coincident inputs are separate edge tests. | Deterministic identities and source formulas to 1e-12 relative with 1e-12 nm absolute position floor. Stochastic mean/covariance intervals within 1% of analytic characteristic scales (sqrt(2Ddt) and 2Ddt); source placement moments within 1% of independently calculated source scales. Zero scales require exact identity. Translation/rotation and dimensional tests are independent physical gates; source anomalies must be resolved, not hidden by tolerance. |
| SYN-3 unary law | Each first/pseudo-first-order literal in the embedded table, isolated at lambda*t = 0.01, 0.1, 1, 10; additionally two synthetic competing hazards with ratios 1:1 and 1:9, and zero total rate. Record survival and channel probabilities at those four times, and products/parent removal. | Analytic survival exp(-sum(lambda)*t), rate-weighted competition, source-literal products and zero-rate identity; simultaneous probability difference intervals within +/-0.01. This is an extension gate; default option3 IRT-syn is not a unary oracle. Any custom Geant4 process gets its own source review and L3 verdict. |
| SYN-4 serial chronology | Two isolated pairs; three molecules sharing a reactant; two independent equal-time reactions; pair versus unary tie; product creating a new partner; dead/stale candidate; stop exactly at event and one representable value on either side. Observe after each event and at 1 ps, 1 ns, 1 us. | Exact scripted-clock event order, identity uniqueness, no double consumption, nondecreasing times, stoichiometry and saved candidate preservation. Independent stochastic runs use the count/mean rules below for all species and reaction-channel counts. Source tie differences are documented deviations, not statistical noise. |
| SYN-5 timed injection and scoring | One 100 keV primary; two 100 keV primaries at (0,0,0)/(10 nm,0,0), both +z, arrivals (0,0), (0,1 ns), (0,10 ns); one 1 MeV primary. Score the five V11 species at 1 ps, 10 ps, 100 ps, 1 ns, 10 ns, 100 ns, 1 us from first arrival, plus immediately before/after each later injection. | Energy ledger and chronology exact within V7; no future energy in denominator, no chemistry before handoff. G(t) mean comparison intervals inside +/-max(0.05*abs(reference G), 0.01 molecules/100 eV); precision half-width <= half that band. Initial handoff/M6 discrepancies stay separately visible. No claim that old V11 passed. |
| SYN-6 restart | The SYN-4/5 workloads stopped at 1 ps, 1 ns, 10 ns, at a pending event, and immediately before injection; compare uninterrupted, one restart and two restarts. Include all-no-event and residual-unary clocks. | Exact persistence of configuration/data/model identities, time, molecule IDs, saved positions, pending stochastic state, irradiation and scoring; corrupt/incompatible files rejected. Scripted continuation exact. Stochastic survival/count/G(t)/reaction-channel families obey the same bands as uninterrupted comparisons, including survivors conditioned on no reaction at stop. Bitwise replay is not required. |
| SYN-7 regions and workers | SYN-4/5 with 1, 2, 8 workers; region faces, edges and corners passing through a pair midpoint, then translated by a/2; pairs on either side of cutoff; simultaneous competing pairs straddling regions; product migration. Compare all-pairs serial, indexed serial and parallel. | Exact candidate coverage, one canonical owner per pair, conflict resolution and deterministic-clock results. No physical boundary reflection/wrapping. Same statistical bands as serial; cutoff contribution separately bounded below. Thread-dependent RNG streams must not silently change physical inputs. |
| SYN-8 release and cost | All applicable SYN gates, existing required transport/reference gates, 100 keV and 1 MeV single-electron end-to-end comparisons; sparse SYN-5 two-track and dense 16-track 100 keV coincident-arrival workloads. Record peak memory, total runtime, candidate count and chemistry runtime at 1/2/8 workers. | All scientific gates PASS, no unresolved source-policy exclusions affecting production input. Report median runtime over 5 fixed workloads after one warmup; report speedup, including slowdown, without a preclaimed target. L4 remains an explicitly separate literature/experimental investigation. |

SYN-2 positive-duration placement cases are direct-kernel diagnostics; the
reviewed normal scheduler call path leaves the placement duration at zero.
Separate an executable scheduler-placement test from that direct-kernel family.
Do not activate a dormant branch merely because its formula appears intended.

Physical formulas use nm and seconds, with explicit units in exported columns.
Exception: the dimensionally inconsistent direct-kernel source placement width
must be evaluated numerically in Geant4 native units (mm and ns), including
its diffusion coefficients. Convert resulting positions to nm only afterward;
evaluating that literal expression in nm changes the source distribution.
This source diagnostic is separate from dimensionally consistent physical tests. SYN-5 observation is post-reaction and post-injection
for events at the same timestamp, with process/unary events before pairs as in
the reference scheduler, and canonical molecule/channel ordering for exact ties.
Insertion of irradiation and ordering relative to ties is a Rust extension:
at a common time, advance existing state, insert the complete arrival group,
settle zero-time reactions, then score. Handoff delays must be represented on
the same absolute timeline; primary arrival is not automatically chemical birth.
Record species counts as well as G(t), using deposited energy from arrivals
whose transport/handoff has completed by the observation; zero denominator is
an explicit undefined score, never zero or division by an eventual total.
Independent statistical samples are complete shared-volume irradiation histories,
not individual molecules or interacting tracks treated as independent histories.

For every stochastic species-count and reaction-channel-count mean in SYN-4,
SYN-6 and SYN-7, the equivalence band is +/-max(0.05*abs(reference mean),
0.1 events or molecules per complete shared-volume history), with interval
half-width <= half that band. The 0.1 absolute floor applies when the reference
mean is zero; deterministic identities and forbidden reactions still require
exact zero. Freeze the numeric band from an independent preregistered pilot
or analytic reference before comparison, never from the evaluated sample.
Restart and worker count endpoints use these same frozen bands; probability
endpoints use +/-0.01 and G-value endpoints use the SYN-5 bands.

Each implementing ticket freezes its complete numeric manifest (source-derived
coefficients, orientations, event scripts, seeds, exact endpoint count and commands)
before running comparisons. It may add edge cases; it may not remove the cases
above, change bands after seeing results, or treat missing cases as PASS. The
manifest enumerates every species, coordinate and channel endpoint, so multiplicity
uses the actual full family, not just this table's row count. Changes to scientific
semantics require a prospective documented amendment and new evidence names.

### Multiplicity, precision and resource policy

For each SYN gate and comparison type (analytic, Geant4, restart, worker), use
family alpha **0.001**, Bonferroni over every registered endpoint and **three**
permitted precision looks. At each look alpha per endpoint is `0.001/(3*m)`.
For two probability estimates, form Clopper-Pearson intervals for each side
with half that error budget and subtract interval bounds; for an exact oracle
use the single interval. For means use two-sided Welch-t intervals on at least
30 independent batch means and the same per-endpoint budget; paired workloads
may instead use a t interval on preregistered paired batch differences. Report
counts, n, means, variances, effect size, interval and verdict, not only p-values.
PASS requires the whole interval inside the effect band; an interval wholly
outside is FAIL; overlap with a boundary or inadequate precision is INCONCLUSIVE.
An ordinary failure to reject equality is not equivalence.

Probability sample sizes are 100,000, 200,000, then 1,000,000 **per setting**;
mean/history families use 300, 1,000, then 3,000 histories with at least 30
batches. Those are precision expansions, not reruns until lucky. Independent
Geant4/Rust seeds and a second independent seed replicate are required; include
both replicates in the declared family. Freeze seed values in each new run request
before generating data. A FAIL is investigated without escalating statistics
to erase it; a third-look INCONCLUSIVE stays open. Historical gate sample sizes
and alpha are unaffected.

Finite-search acceptance additionally requires the reference contract's accumulated
omitted-event probability bound <= **1e-6 per history** and comparison to all-pairs
execution. Test source cutoff, twice source cutoff and all-pairs with identical
input manifests; only all-pairs or a proven bounded search is eligible for the
new production policy. Search approximation, source agreement and statistical
noise receive separate verdicts. Transparent computational regions do not grant
permission to discard cross-region pairs.

Reference validation is limited initially to **2 hours and 8 GiB per invocation**,
**24 hours total per gate**, and **1 million pair samples or 3,000 histories per
setting**. Batch runs into new complete files within the existing fixture budget
(<1 MB per fixture, <20 MB total); retain raw traces in scratch. A resource limit,
incomplete file or missing completion marker is INCONCLUSIVE/failed execution,
never a scientific pass. The production implementation must expose finite resource
limits and fail explicitly rather than truncate chemistry. Exceeding a budget
requires a prospective amendment, not a hidden relaxation. Tests/consumers and
exact gate commands enter `validation/gates.json` when introduced; no blanket
ignored-test run or non-existent test command is prescribed by this document.

SYN-2 analytic sampling allocation (2026-11-09, frozen before execution):
the complete README manifest retains all6228 endpoints and1% bands, uses
independent master-seed replicates20261109081/082, and couples per-history
primitive RNG streams across settings. Each setting retains independent
histories; setting-to-setting dependence is permitted by Bonferroni and
reduces redundant variance fluctuations under exact diffusion scalings.
Placement uses distinct source-formula expectations under shared inputs,
not an assertion that differently parameterized distributions are equal.
The three looks are300/1000/3000 history batches with333 draws each
(max999000); each history is a batch, satisfying at least30 batches.
Prospective precision/power reasoning and superseded independent-setting
allocation are preserved in SYN2_FEASIBILITY.md. No full-family PASS was
predicted, and final-look INCONCLUSIVE stays open under the existing policy.
## Finite-volume boundary acceptance criteria (fixed 2026-11-13, before any data)

Source basis: docs/04 D.7 (Geant4 11.4.3 audit). Geometry: ADR 0005 (`Volume` enum). Reference: RR-009
(awaiting generator and run). These criteria are fixed now. Changing them later needs a prospective,
documented amendment, never a relaxation after seeing data.

**Settings (one family).** Electron primaries at the origin, isotropic, in a liquid-water world with
the same DNA physics and chemistry constructors as the existing `g4rs_ref` modes. 3 energies (1, 10,
100 keV) x 4 geometries (`Sphere` r = 0.1, 1, 10 um; `Box` half = (1, 1, 0.1) um) = 12 settings, plus
`Unbounded` as a control (Rust only). Histories: 10,000 per setting for each code, with independent seeds.

**Deterministic checks (every history, both codes, must hold exactly up to 1e-9 relative).**
- D1 energy closure: `E0 = E_dep_inside + E_escaped`, where `E_escaped` is the kinetic energy of tracks
  killed at the world boundary (docs/04 D.7).
- D2 no deposit, species or interaction point outside the volume, except a dissociation product
  flagged as `OUTSIDE_OF_MOTHER_VOLUME` (counted, see S3).
- D3 every boundary-limited step ends on the surface (`distance_to_out` <= 1e-9 x length scale).
- D4 `Unbounded` output is bit-identical to the current infinite-medium path (regression).

**Statistical endpoints (per setting; family of 12 x 2 = 24 endpoints, S1 and S2).**
- S1 mean escaped-energy fraction `E_escaped/E0`.
- S2 escape probability of the primary.
- S3 fraction of species alive at 1 ps whose clamped placement was shortened (solvation and
  dissociation 0.8 x safety rule) and count of outside-volume products. Reported for diagnosis; it enters
  the family only after G-B2 (docs/09) is answered.

Verdict per endpoint: two-sided Bonferroni interval at family alpha 0.001 on the Rust minus Geant4
difference. **PASS** if the whole interval lies within +-0.01 absolute. **FAIL** if the interval excludes
0 and lies wholly outside +-0.01. Otherwise **INCONCLUSIVE**, which can be resolved only by a
precision-only expansion of history counts with the same seeds family. The gate passes only when all
included endpoints PASS and D1-D4 hold.
