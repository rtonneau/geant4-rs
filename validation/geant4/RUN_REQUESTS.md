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

No request exists yet.

| id | seed (gps) | program | command | rng seed, statistics | outputs | provenance | consumer | status |
|---|---|---|---|---|---|---|---|---|
