---
name: g4-reference-run
description: Run a registered Geant4 reference generator (g4rs_ref) request from validation/geant4/RUN_REQUESTS.md, validate its output, commit data and provenance, and run the paired ignored Rust comparison test. Use only when the user explicitly asks to run a request (e.g. "run RR-002"); otherwise agent sessions only compile-check and log requests.
---

# g4-reference-run

Project-specific procedure for executing one `RR-NNN` request. Details live in the repo; read them
first, do not rely on this summary: `validation/geant4/RUN_REQUESTS.md` (the entry, its command and
acceptance text), `validation/geant4/BUILD_NOTES.md` (build recipe, run environment, symptom table),
`validation/geant4/README.md` (arguments, schemas, provenance fields), and AGENTS.md ("Geant4
reference data and generators").

## 0. Authorisation gate

AGENTS.md: agent sessions never run the generator. Proceed only if the user explicitly asked to run
this request in the current conversation. Otherwise stop after compile-checking and logging the entry.
Authorisation covers the named request only.

## 1. Preconditions

- The `RR-NNN` entry exists with status `awaiting run`; read its command, seed, outputs, provenance
  name and consumer test. The consumer test must exist and be `#[ignore]`d with criteria already
  stated in its header.
- Outputs are **new** files: stop if `validation/reference/<output>` or the provenance already exists.
- `git status` clean for `validation/geant4/`; record `git rev-parse HEAD` as the generator commit
  (`generator_dirty = false`). If dirty, say so and put `true` in the provenance.
- Never edit the Geant4 C++ repository, existing generator modes, or existing reference files.

## 2. Environment (never guess a path)

`G4_ROOT` and `DEV_DIR` must be set (else stop and point to `~/dotfiles/scripts/setup-dev-env.ps1`).
`QT5_PREFIX` and `G4LEDATA` are often unset in agent shells. Discover, do not invent:
- Qt: `CMAKE_PREFIX_PATH` / `Qt5Core_DIR` in the build directory's `CMakeCache.txt`.
- `G4LEDATA`: the directory ending in `G4EMLOW8.8` under the Geant4 data root (sibling of `G4_ROOT`).
- Runtime `PATH`: `$G4_ROOT\bin`, `<Qt prefix>\bin`, `$DEV_DIR\vcpkg\installed\x64-windows\bin`.
Confirm each directory exists; if one is ambiguous or missing, ask the user.

## 3. Build

Rebuild in the build directory named by the request (BUILD_NOTES section for the RR) with
`vcvars64.bat` wrapped around `cmake --build`; success = exit 0 and `Linking CXX executable`.
Fresh configure follows BUILD_NOTES section 2. Do not commit build directories.

## 4. Smoke, then production

1. Smoke case from BUILD_NOTES, output to the scratchpad directory or `$env:TEMP`. Require exit 0 and
   non-empty rows. Never commit smoke output.
2. Production: the exact RR command, with the output going to the **scratchpad directory**, not
   `validation/reference/`; log to a file; run in the background (tens of minutes).
3. Validate before copying: exit code 0; one completion line per requested energy/setting with the
   requested statistics; row count and schema from README; header marker lines required by the consumer
   test. A crash can leave a valid-looking partial CSV, so any shortfall means no copy.

## 5. Commit data, then provenance

1. Copy the validated CSV to the new name in `validation/reference/` and commit it alone.
2. Write `provenance_<name>.toml` with every field README requires: Geant4 version and commit, data
   set versions and md5, constructors, time-step model, snapshot boundary, `generator` = repository
   commit, `generator_dirty`, `data_commit` (the commit from step 1), date, seed, command line exactly
   as registered, compiler (`cl` banner from the vcvars shell), platform (OS version). Do not invent
   a runtime you did not measure. Commit it.

## 6. Consumer test and bookkeeping

- Run the paired test with the command in the RR entry (`-- --ignored --exact --nocapture`). If it
  fails on test plumbing (path, parser) fix only the plumbing; if it fails on a **criterion**, do not
  touch tolerances, statistical families or reference data. Make the test report all failing rows
  without changing what counts as a failure.
- Update the RR entry status (`closed`, with commits and pass/fail), `docs/08` (result table with
  Geant4 and Rust values), and `docs/09` (affected questions/risks); keep the earlier text as history.
- Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`; report what ran. Commit docs/tests. Do not push unless asked.

## Reporting

State plainly: exit code, completion lines, row count, test outcome with the failing rows if any, and
what is still open. A failed gate is a result to record, not something to fix by loosening criteria.
