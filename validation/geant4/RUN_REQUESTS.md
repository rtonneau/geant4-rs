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
- Status: **awaiting run**. Agent compile-check only; no executable run.

Require 144 rows, exact coverage, valid counts, the requested seed/statistics,
exit 0 and final complete marker before committing new evidence. The frozen
Bonferroni/effect/equivalence and precision-expansion rules are in
`docs/validation/pair-case-matrix.md`. Future precision levels use new paths;
never replace RR-003 or previously committed reference files. Geant4 agreement
alone does not establish the independent charged Brownian physical-law gate.
