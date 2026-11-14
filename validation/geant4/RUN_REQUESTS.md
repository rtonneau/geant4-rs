# Geant4 run requests

Log of the Geant4 simulations that an agent session needs but may not run itself. The project lead
runs them (Geant4 11.4.1, see `BUILD_NOTES.md`) and normally commits the output. A request explicitly
classified as diagnostic-only may instead retain complete output and provenance as local-only
artifacts when its result is recorded in the durable project documentation.

## Rules for agent sessions

- A session adds **new** C++ modes or programs under `validation/geant4/`. Existing modes keep their
  arguments, physics settings and output format (AGENTS.md, "Geant4 reference data and generators").
- A session may compile-check the new code against the installed Geant4. It **never runs** it, not even
  as a smoke test, and never commits a CSV it produced itself.
- Every new generator takes output path, seed and statistics on the command line and contains no
  machine paths.
- For each run the user must perform, the session appends one entry to the log below and updates
  `README.md` (arguments, schema, acceptance criteria) and `BUILD_NOTES.md` (build and run recipe).
- The Rust comparison that consumes the data is written in the same session as an `#[ignore]`d test,
  with tolerances and statistical families stated in its header **before** any data exists.
- Committed output files use **new** names in `validation/reference/`. Existing files are never
  overwritten. A diagnostic-only request may use new local-only names under `.scratch/`; its log entry
  must identify that exception and must not present those artifacts as committed reference evidence.

## Gate flow

1. The producing session ends with the gate **awaiting data**: C++ written and compile-checked, request
   logged, ignored Rust test written. It does not claim any Geant4 agreement.
2. The user runs the command, checks exit code 0 and complete non-empty output, writes the
   `provenance*.toml` (Geant4 version, data sets, constructors, exact command, seed, date, generator
   commit and dirty state, compiler, platform), and either commits data and provenance or follows an
   explicitly logged diagnostic-only/local-only exception.
3. The paired `<slug>-close-gate` seed verifies the provenance, runs the consuming test, and records the
   result (pass or fail, with evidence) in `docs/08`. Tolerances are never loosened to pass.

## Entry format

| Field | Content |
|---|---|
| id | `RR-NNN`, increasing |
| seed (gps) | requesting seed slug and session id |
| program | generator mode or program, with its source path |
| command | exact command line (no machine paths) |
| rng seed, statistics | seed, events per energy or setting |
| outputs | new files in `validation/reference/`, or explicitly identified local-only diagnostic files |
| provenance | `provenance*.toml` file to produce |
| consumer | the `#[ignore]`d Rust test and the command that runs it |
| status | `awaiting run`, `data committed`, `completed local-only`, or `closed` |

## Log

| id | seed (gps) | program | command | rng seed, statistics | outputs | provenance | consumer | status |
|---|---|---|---|---|---|---|---|---|
| RR-001 | `m6-diagnose-and-fix`, `2026-10-16__m6-diagnose-and-fix` | `g4rs_ref physchem_prechem`, source `validation/geant4/g4rs_ref/g4rs_ref.cc` | `g4rs_ref physchem_prechem ../../reference/physchem_prechem.csv 20261020 1000:20000,10000:2000,100000:200,900000:20` (from `validation/geant4/g4rs_ref`, executable on PATH) | 20261020; 20000/2000/200/20 events at 1/10/100/900 keV | `validation/reference/physchem_prechem.csv` (new) | `validation/reference/provenance_m6_prechem.toml` (new; README fields) | `crates/dna-sim/tests/physchem_reference.rs::m6_prechem_counts_and_rms_match_geant4`; `cargo test -p dna-sim --test physchem_reference m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact --nocapture` | closed 2026-10-17: data `e808b2f`, `ffe4b48`; gate failed 16/28 rows (docs/08 "RR-001 result") |
| RR-002 | `m6-900kev-spatial-rms-investigation`, `2026-10-19__m6-900kev-spatial-rms-investigation` | `g4rs_ref physchem_spatial_diagnostic`, source `validation/geant4/g4rs_ref/g4rs_ref.cc` | initial command: `g4rs_ref physchem_spatial_diagnostic ../../reference/spatial_geant4_900kev_40.csv 40 20261021,20261022,20261023,20261024,20261025` | five independent seeds 20261021..20261025; expanded from 40 to 80 complete 900 keV histories/seed | local-only `.scratch/rr-002-80/spatial_geant4_900kev_80_local.csv` and matching Rust CSV | local-only `.scratch/rr-002-80/provenance_m6_spatial_900kev_80_local.toml` | `crates/dna-sim/tests/physchem_reference.rs::m6_900kev_spatial_diagnostic`; matching 80-history streaming ignored consumer | completed 2026-10-23: both exits 0 and exact five x 80 coverage; ratio 0.981740457, 95% interval [0.949564151, 1.015007983], relative half-width 3.333% <= 4%; no 160/seed expansion; artifacts not promoted to committed references |
| RR-003 | `m7-sbs-oracle-and-ode`, `2026-10-23__m7-sbs-oracle-and-ode` | `g4rs_ref two_body_h_h`, source `validation/geant4/g4rs_ref/g4rs_ref.cc` | `g4rs_ref two_body_h_h ../../reference/two_body_h_h.csv 202610231 100000` | seed 202610231; 100,000 independent H+H pairs; r0=0.2 nm; tau 0.25,0.5,1,2,4,16 | `validation/reference/two_body_h_h.csv` (new) | `validation/reference/provenance_m7_two_body_h_h.toml` (new; README fields) | `crates/dna-chemistry/tests/geant4_two_body.rs::m7_irt_survival_matches_geant4_sbs`; `cargo test --release -p dna-chemistry --test geant4_two_body m7_irt_survival_matches_geant4_sbs -- --ignored --exact --nocapture` | awaiting clean committed rerun; an explicitly authorized 2026-10-23 scratch run exposed and diagnosed two generator defects and is not reference evidence |

### RR-001 purpose and acceptance (2026-10-17)

M6 is **awaiting data**. The old first-positive-step pre-callback contains zero-time IRT reactions;
its aggregate CSV cannot reconstruct the pre-reaction survivors or positions (docs/08 Q25/R17).
The new post-zero-step callback captures the merged alive list as soon as no water states remain,
before the next model selection initializes IRT. No chemistry processes or reaction table are changed.
The new mode compiles and links against Geant4 11.4.1; it has **not been run** by an agent.

The project lead first runs the separate smoke case in `BUILD_NOTES.md` section 7, then the production
command above. Require exit 0, four energy completion lines, correct statistics and 28 rows. Runtime
guards reject any preceding chemical reaction or positive-time step, wrong time/status, or missing
snapshot. Do not commit partial/smoke data. The consumer rejects missing/duplicate species, reaction-only
products, invalid moments or provenance. Every species/energy row retains the original count z <= 4.5,
15% count limit when both means exceed 0.1/event, and 20% RMS limit; criteria are fixed before the run.
The paired `m6-close-gate` seed checks this result before declaring closure. A failure leaves Q21/R14
and any additional cause open; neither the original gate nor its historical evidence is replaced.

### RR-002 purpose and acceptance (2026-10-20)

RR-002 localizes the common 900 keV spatial residual across five lifecycle boundaries. The mode
compiled and linked against Geant4 11.4.1. The completed local expansion contains 400 histories per
implementation (five seeds x 80), exit 0, exact coverage, trailing completion markers and matching
provenance. The consumer rejected wrong stage/settings, malformed rows, duplicate/missing categories
and incomplete seed/history blocks before analysis.

The frozen 20,000-replicate independent-sample hierarchical bootstrap uses analysis seed 20261026.
First detected divergence is descriptive, not causal. Sample growth is controlled only by primary
relative interval half-width: 40 to 80 to 160 histories per seed in both implementations, stopping at
800 histories each. New levels require new filenames/provenance and complete runs; never append
duplicate identities. RR-002 records evidence only and neither reopens nor closes M6.

The primary Rust/Geant4 pooled final-survivor RMS ratio was 0.981740457, with a 95% interval of
[0.949564151, 1.015007983]. The 3.333% relative interval half-width met the <= 4% stopping target, so
the 160-history level was not run. Because the interval includes 1.0, no primary difference was
statistically resolved; this is not an equivalence proof. RR-001's +12.1% to +13.5% excess was not
reproduced. The completed 80-history artifacts remained local-only and were not promoted to committed
reference evidence; the full scientific interpretation is in `docs/08-roadmap.md`.

### RR-003 purpose and acceptance (2026-10-23)

RR-003 tests the executable Geant4 11.4.1 option3 SBS H+H survival curve against the fixed-seed Rust
IRT sampler at the six V9 times. The CSV must have exactly six ordered rows, 100,000 covered pairs per
row, monotonic reacted counts, the requested seed and the trailing completion marker. Provenance must
fix Geant4 11.4.1, G4EMLOW 8.8, option3, SBS, reaction id 1, r0=0.2 nm, the exact tau family, seed,
sample count and command, plus nonempty generator commit, compiler, platform and date fields.

The frozen pointwise rule is
`|F_IRT-F_G4| <= 3*sqrt(2*F_V9*(1-F_V9)/100000)` at every one of the six points. It was inherited
unchanged from the V9/IRT-SBS comparison and was not fitted to Geant4 output. The first authorized
scratch run preceded this request entry, so the project does not claim retrospective preregistration.
That run found a canonical-name callback defect (`H` versus `H^0`) and Windows seed overflow; its CSVs
remain diagnostic-only. RR-003 stays open until the corrected generator is committed, the exact clean
command above produces new reference data and provenance, and the paired close-gate test records the
expected pass or failure without changing tolerances.


### RR-004: R04 option3 isolated IRT matrix (2026-10-27)

- Session: `2026-10-27__pair-oracles-and-reference-gate`.
- Program: `g4rs_ref pair_matrix_irt`, source `validation/geant4/g4rs_ref/g4rs_ref.cc`.
- Exact command (from `validation/geant4/g4rs_ref`, executable on PATH):
  `g4rs_ref pair_matrix_irt ../../reference/pair_matrix_irt_r04_n100000.csv 20261027404 100000`.
- Seed: 20261027404; CLHEP RanecuEngine with README's lossless two-word mapping.
- Statistics: 100,000 independent samples for each of eight literal channels
  and three separations; five finite CDF points plus eventual nonreaction.
- New outputs: `validation/reference/pair_matrix_irt_r04_n100000.csv` and
  `validation/reference/provenance_r04_pair_matrix_n100000.toml`.
- Schema, units, provenance fields and complete-only guards: README R04 section.
- Consumer: ignored `g4-chemistry` test `geant4_pair_r04`;
  do not substitute historical RR-003's SBS consumer.
- Status: **awaiting complete run**; smoke attempt failed as described below.

Smoke attempt 2026-10-29 (explicitly requested): the rebuilt generator exited 1
after completing channels 1, 2, 8 and 9, with `unexpected IRT result` on
channel 11. Its scratch CSV has no completion marker and is not evidence.
Production was not started. Geant4 11.4.1's neutral PDC branch computes
`b = (r0 - sigma)/2`, which is zero at the registered contact distance; its
`SamplePDC` calculation then includes division by zero and indeterminate
arithmetic. The exact returned value was not logged. This edge case needs a
source-faithful resolution before retrying; do not classify an unexpected
return as physical nonreaction or change the registered grid silently.

Diagnostic rerun 2026-10-29: after adding failure-only value logging, the
100-pair seed-1 smoke exited 1 at channel 11, contact, pair index 1. The
returned native value was `-108695652173.91306`, or
`-108695652173913.05 ps`, not the `-1 ps` no-reaction sentinel. For `b=0`,
`SamplePDC` computes `p=0`, `q=inf`, and then `X=0`; its acceptance expression
contains `0/0`. Its 10,001-rejection fallback returns `-1` in the sampler's
intermediate units, and the caller divides it by the relative diffusion
coefficient. This explains the observed negative value. The diagnostic output
is incomplete and is not a reference fixture. Exact contact cannot be sampled
faithfully by this unmodified Geant4 path after the reaction gate accepts.
An analytically derived limiting sampler would be a modified reference, not a
direct Geant4 result; no such substitution is authorized by this registration.

Require 144 rows, exact coverage, valid counts, the requested seed/statistics,
exit 0 and final complete marker before committing new evidence. The frozen
Bonferroni/effect/equivalence and precision-expansion rules are in
`docs/validation/pair-case-matrix.md`. Future precision levels use new paths;
never replace RR-003 or previously committed reference files. Geant4 agreement
alone does not establish the independent charged Brownian physical-law gate.


### RR-005: G1-F comparable option3 IRT matrix (2026-11-05)

**Status: closed at 100k; GEANT4 INCONCLUSIVE (2026-11-05).**
Generator `4e5ea99`, complete data `eba9b3c`, provenance `b7febf0`.
Exit 0, eight completion lines, 120 rows. Consumer exit 101: 87 PASS,
33 insufficient-precision endpoints, zero FAIL. G1-F remains open; next
precision level 200k requires new files. Full results: docs/08.
Prospectively approved replacement gate; RR-004 remains unchanged and unpassed.

- Program: `g4rs_ref pair_matrix_irt_g1f`, Geant4 11.4.3 / G4EMLOW 8.8.
- Working directory: `validation/geant4/g4rs_ref`; executable on PATH.
- Command: `g4rs_ref pair_matrix_irt_g1f ../../reference/pair_matrix_irt_g1f_n100000.csv 20261027404 100000`.
- New files: that CSV and `validation/reference/provenance_g1f_pair_matrix_n100000.toml`.
- Require a clean committed generator, exit 0, 120 ordered rows, eight channel
  declarations, the exact 24-endpoint exclusion/contact-regression declaration,
  complete counts and the final completion marker; then record actual provenance
  and checksum using README's template. Do not commit partial or smoke output.
- Consumer: `cargo test --release -p g4-chemistry --test geant4_pair_g1f g1f_option3_irt_matches_geant4 -- --ignored --exact --nocapture`.
- Acceptance and precision expansion: README G1-F section and
  `docs/validation/g1-f-source-fidelity.md`. A 100k run can be INCONCLUSIVE;
  new 200k/400k/800k/1m files are permitted only for insufficient precision.
- Exact PDC contact is excluded explicitly; the direct SamplePDC fallback
  diagnostic runs separately before RNG reset. Rust's contact limit is not
  represented as a Geant4 return. No physical-law PASS is claimed.

Agents may compile-check this mode but must not run it, including smoke tests.

Version amendment: explicitly authorized 2026-11-05 to use 11.4.3. The earlier
11.4.1 production output is scratch-only; no reference fixture was accepted.

### RR-005 authorized 200k expansion (2026-11-05)

Status: closed, GEANT4 PASS. The user authorized this precision-only expansion
after 100k was inconclusive. From `validation/geant4/g4rs_ref`:
`g4rs_ref pair_matrix_irt_g1f ../../reference/pair_matrix_irt_g1f_n200000.csv 20261027404 200000`.
New CSV `pair_matrix_irt_g1f_n200000.csv` (data commit `51c22b6`) and
`provenance_g1f_pair_matrix_n200000.toml` (`0b96afe`) preserve 100k evidence.
Producer exit 0, eight channel completion lines, 120 rows. Consumer with
`G1F_GEANT4_PAIRS=200000` exited 0: all 120 included endpoints PASS;
24 PDC exact-contact endpoints excluded. No thresholds or seeds changed.

### RR-006: normative IRT-syn isolated pair diagnostics (2026-11-08)

Status: closed, source diagnostics PASS. Both complete 100000 and 200000 looks were precision-INCONCLUSIVE;
both complete 1000000 replicates passed 448 stochastic comparisons and 16
deterministic branch identities. The dedicated runner's logs are in
`.scratch/tests/2026-11-08__irt-syn-parallel-checkpoint-direction/02-rr006/`.
Six CSVs are committed in 9607036827be6954fb5423f294275d36c383661c; matching provenance in f3c169f. This is source-diagnostic acceptance only.

Original registration: execution is explicitly
authorized through the dedicated reference-run subagent in this GPS session.
Build `g4rs_ref` against discovered Geant4 11.4.1, G4EMLOW 8.8. Working
directory `validation/geant4/g4rs_ref`; executable on PATH. New commands:

```text
g4rs_ref syn_pair_matrix ../../reference/syn_pair_1141_r1_n100000.csv 20261108061 100000
g4rs_ref syn_pair_matrix ../../reference/syn_pair_1141_r2_n100000.csv 20261108062 100000
```

Run to new scratch paths first; require exit 0, eight channel completion
messages, 232 ordered rows and final complete marker. Preserve existing
reference files. A smoke run with seed 1 and 100 samples is scratch-only.
No output is a complete SYN-1 acceptance result: initial rejection/post-start
overlap are explicitly unresolved branch diagnostics, not full stepper clocks.
README RR-006 freezes schema, coefficients, all endpoints, seeds, family,
precision and provenance. Record actual library/data/compiler/source hashes
and generator revision/dirty status truthfully (uncommitted source requires
its complete named hashes). New provenance paths:
`provenance_syn_pair_1141_r1_n100000.toml` and
`provenance_syn_pair_1141_r2_n100000.toml`.

Exact consumer:
`cargo test --release -p g4-chemistry --test geant4_syn_pair syn_pair_source_diagnostics_match_geant4 -- --ignored --exact --nocapture`.
Only a precision-INCONCLUSIVE result permits 200000 then 1000000, with the
same seeds and new matching filenames; set `SYN_PAIR_SAMPLES` accordingly.
FAIL requires investigation. No placement, unary, or integrated scheduler
gate is represented by this request.

### RR-007: common-input IRT-syn placement mapping (2026-11-09)

Status: closed, finite-grid mapping PASS. Dedicated production exited 0 with 1200 rows, 144 expected errors and 8064 coordinates; strict Rust consumer exited 0. CSV committed in 9607036827be6954fb5423f294275d36c383661c and provenance in f3c169f. User prospectively
approved paired common random inputs while keeping the current cap. This is
finite-grid mapping evidence only; no independent moment/RNG/physical PASS.

Build opt-in target `g4rs_syn_ref` against 11.4.1 using
`G4RS_SYN_SOURCE_ROOT` for the verified read-only source root. CMake rejects
a source-body hash mismatch. The linked reference repository is never changed.
Exact registered command, from `validation/geant4/g4rs_ref`:

```text
g4rs_syn_ref paired ../../reference/syn_paired_1141.csv 20261109071 16
```

First smoke: new scratch output, seed1, one tape; require exit0 and75 rows.
Production: new scratch output, registered seed/tapes; require exit0,1200
ordered rows,8064 numeric parent/product coordinates,144 explicitly expected
coincidence errors, and final complete marker. Every other error aborts.
Producer and consumer enforce the <1MB fixture cap; original per-invocation
2-hour/8-GiB and24-hour gate limits remain. No larger precision run is needed
for this deterministic finite grid and no statistical inference is attached.
README freezes all inputs, semantic draw labels, schema, numerical tolerance
and exact provenance requirements. Only raw labelled primitives are common;
Rust reconstructs physical inputs independently from its embedded table.

New evidence: `syn_paired_1141.csv` and `provenance_syn_paired_1141.toml`.
Consumer: `cargo test --release -p g4-chemistry --test geant4_syn_paired syn_paired_mapping_matches_geant4 -- --ignored --exact --nocapture`.
Do not alter RR-006 files, modes or provenance. Ordinary scheduler trace and
independent analytic marginal/free-diffusion acceptance remain separate work.

RR-007 recovery smoke (2026-11-09): native exit0, seed1/tapes1, 75 ordered rows, 9 expected coincidence errors, 504 finite coordinates, exact completion footer. Original failed smoke and native double-delete stack remain in session scratch. Subsequent independent review found no actionable readiness defect and production passed as recorded above. Use a retained native process handle and require a non-null exit code.

### RR-008: ordinary IRT-syn library scheduler trace (2026-11-09)

Status: registered, awaiting dedicated authorized runner. New target
`g4rs_syn_scheduler_ref` links ordinary installed Geant4; configure/build with
the same discovered normative11.4.1 environment. Exact manifest/schema,
provenance fields and acceptance are in README RR-008.

```text
g4rs_syn_scheduler_ref trace ../../reference/syn_scheduler_1141_r1.csv 20261109091 32
g4rs_syn_scheduler_ref trace ../../reference/syn_scheduler_1141_r2.csv 20261109092 32
```
Run new scratch outputs first. Smoke seed1/trials1 requires native exit0,
9complete rows and footer (no minimum stochastic coverage). Production requires
native exit0 and288ordered rows per replica; censored histories remain rows.
Retain native process handle before waiting; reject null ExitCode.
Two-hour/eight-GiB invocation and24-hour gate budgets retained; each CSV<1MB.
Strict consumer checks unchanged3a parent sites on actual positive-time events,
product support/count and post-step parent removal; at least one event per
channel/orientation per replica gives coverage, otherwise INCONCLUSIVE.
This addresses ordinary-path applicability only, not global scheduler or
reaction-time distribution acceptance. Source-marginal233 positive-duration
INCONCLUSIVE endpoints remain open regardless of the trace result.
New provenance names: `provenance_syn_scheduler_1141_r1.toml` and
`provenance_syn_scheduler_1141_r2.toml`. Consumer:
`cargo test --release -p g4-chemistry --test geant4_syn_scheduler syn_ordinary_scheduler_preserves_parent_sites -- --ignored --exact --nocapture`.

RR-008 recovery 3 (2026-11-09): native stacks identified two adapter initialization omissions. G4EmDNAPhysics construction resets the requested timestep model to Unknown; selecting IRT_syn after physics construction restores registration. With no transport BeamOn, Brownian transportation had no water-density table; the supported chemistry UseAsStandalone(true) initialization builds molecular process tables and closes geometry. Guards verify the enum before process construction/after initialization and the actual installed IRT model, stepper and reaction process in StartProcessing. Corrected scratch seed1/trials1 exited native 0 with nine ordered rows, three reactions and six censored histories; schema, footer, parent sites, product support/count and removal checks passed. This is recovery smoke only: independent review, fresh dedicated smoke and registered production/consumer remain pending. No reference-source, random stream, manifest, tolerance or statistical result changed; 233 positive-duration marginal endpoints remain INCONCLUSIVE.


### RR-008 recovery and registered result (2026-11-09)

The user-authorized recovery corrected trace-harness initialization order and
standalone molecular physics-table initialization. Independent review found no
actionable defect. A subsequent fresh smoke passed before production.
Both registered production runs exited 0 with 288 ordered rows and complete
footers. Replicate 1 has 80 reactions/208 censored trials; replicate 2 has
79 reactions/209 censored trials. All observed parent-site/product/removal
checks passed, but channel 11, +z had zero positive-time events in both runs.
Verdict: **INCONCLUSIVE**, not PASS. The exact registered Rust consumer exited
101 at the coverage gap in replicate 1, before examining replicate 2; a
separate full-output validator checked both files. No additional samples or
seed changes were made. The 233 analytic moment endpoints remain INCONCLUSIVE.

Complete data commit: `2264b14ffe024b2f44accb2c53b29ae40ee93d37`.
Provenance commit: `9b4aaf4`. Final comparison log is
`.scratch/tests/2026-11-08__irt-syn-parallel-checkpoint-direction/02-rr008/production-consumer-complete.log`.
RR-006/RR-007 were not rerun or replaced. Ticket 02 remains incomplete pending
the registered coverage gate; no later production acceptance follows from
these observed reactions alone.
### RR-009: finite-volume boundary reference (requested 2026-11-13)

Status: **closed 2026-11-14** (run 2026-11-13 at the user's request; Geant4 11.4.3 install, fresh
`build-1143` with vcpkg Qt6; all four runs exit 0 with three `done (10000 events)` lines each; data
`aca5947`, provenance `d48318f`). Consumer: 16 PASS, 0 FAIL, 2 INCONCLUSIVE (sphere 1000 nm, 10 keV,
S1 and S2: se 0.0026, so the interval half-width 0.0107 exceeds +-0.01), 6 NOT RUN (box); the
gate stays open (8 open endpoints). D1 holds on every Geant4 history (max relative 5.7e-14).
Header label `events 0` is a cosmetic generator bug (see provenance). Earlier status, kept as history:
awaiting run (generator written 2026-11-13, session
`2026-11-13__physicochemical-handoff`; compile-checked and linked with MSVC against the 11.4.1 and
11.4.3 installs; never run by an agent). Program: `g4rs_ref finite_volume`, source
`validation/geant4/g4rs_ref/g4rs_ref.cc`. The working name and shape above (`finite_volume_boundary
<out.csv> <seed> <histories>`) are replaced, before any run, by one invocation per geometry, because the
world solid is fixed at initialisation: `finite_volume <out_prefix> <seed> <sphere|box> <r | hx,hy,hz>
<E_eV:events,...> [end_time_ps]` (dims in nm, end time default 1e6 ps). Purpose and acceptance: docs/07
"Finite-volume boundary acceptance criteria"; G-B1/G-B2: docs/09. Geant4 version: 11.4.3 as
registered (state the install used in the provenance).

Commands (working directory `validation/geant4/g4rs_ref`, executable on PATH), seeds 2026111301..04,
10,000 histories per energy (1, 10, 100 keV), 12 settings:
```text
g4rs_ref finite_volume ../../reference/rr009_sphere_100nm 2026111301 sphere 100 1000:10000,10000:10000,100000:10000
g4rs_ref finite_volume ../../reference/rr009_sphere_1000nm 2026111302 sphere 1000 1000:10000,10000:10000,100000:10000
g4rs_ref finite_volume ../../reference/rr009_sphere_10000nm 2026111303 sphere 10000 1000:10000,10000:10000,100000:10000
g4rs_ref finite_volume ../../reference/rr009_box_1000x1000x100nm 2026111304 box 1000,1000,100 1000:10000,10000:10000,100000:10000
```

Outputs (new, exclusive creation): `validation/reference/rr009_<label>_histories.csv` and
`rr009_<label>_species.csv` for the four labels; provenance `validation/reference/provenance_rr009.toml`
(README fields, all four commands). Schemas: README "RR-009 finite-volume boundary". Consumer:
`crates/g4-sim/tests/rr009_finite_volume.rs::rr009_escape_observables_match_geant4`;
`cargo test --release -p g4-sim --test rr009_finite_volume -- --ignored --exact rr009_escape_observables_match_geant4 --nocapture`.
The species file answers G-B1/G-B2 (outside counts at 1 ps and at the end of chemistry); no Rust
consumer reads it yet. Known open item: the Rust source is not isotropic, so the box endpoints stay
`NOT RUN` in the consumer.
