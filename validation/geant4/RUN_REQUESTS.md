# Geant4 run requests

Log of the Geant4 simulations that an agent session needs but may not run itself. The project lead
runs them (Geant4 11.4.1, see `BUILD_NOTES.md`) and commits the output.

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
- Output files use **new** names in `validation/reference/`. Existing files are never overwritten.

## Gate flow

1. The producing session ends with the gate **awaiting data**: C++ written and compile-checked, request
   logged, ignored Rust test written. It does not claim any Geant4 agreement.
2. The user runs the command, checks exit code 0 and complete non-empty output, writes the
   `provenance*.toml` (Geant4 version, data sets, constructors, exact command, seed, date, generator
   commit and dirty state, compiler, platform) and commits data and provenance.
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
| outputs | new files in `validation/reference/` |
| provenance | `provenance*.toml` file to produce |
| consumer | the `#[ignore]`d Rust test and the command that runs it |
| status | `awaiting run`, `data committed`, or `closed` |

## Log

| id | seed (gps) | program | command | rng seed, statistics | outputs | provenance | consumer | status |
|---|---|---|---|---|---|---|---|---|
| RR-001 | `m6-diagnose-and-fix`, `2026-10-16__m6-diagnose-and-fix` | `g4rs_ref physchem_prechem`, source `validation/geant4/g4rs_ref/g4rs_ref.cc` | `g4rs_ref physchem_prechem ../../reference/physchem_prechem.csv 20261020 1000:20000,10000:2000,100000:200,900000:20` (from `validation/geant4/g4rs_ref`, executable on PATH) | 20261020; 20000/2000/200/20 events at 1/10/100/900 keV | `validation/reference/physchem_prechem.csv` (new) | `validation/reference/provenance_m6_prechem.toml` (new; README fields) | `crates/dna-sim/tests/physchem_reference.rs::m6_prechem_counts_and_rms_match_geant4`; `cargo test -p dna-sim --test physchem_reference m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact --nocapture` | closed 2026-10-17: data `e808b2f`, `ffe4b48`; gate failed 16/28 rows (docs/08 "RR-001 result") |

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
