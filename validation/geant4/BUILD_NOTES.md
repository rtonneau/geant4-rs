# g4rs_ref: build and run runbook (Geant4 11.4.1, Windows / MSVC)

How to build and run the reference generator `validation/geant4/g4rs_ref` (C++, links the installed
Geant4) and what to do when it fails. Written for an agent with no prior context: follow sections 1-4
in order, then use section 5 (symptom -> fix) if something breaks. Do not change the physics settings
of the generator (see `../README.md`); only fix build/runtime problems.

All paths below are **variables**, never hard-code them. The checked-in generator source and
`CMakeLists.txt` contain no machine paths.

## 1. Prerequisites (verify first, stop and report if one is missing)

| Variable | Meaning | Example (author's machine) |
|---|---|---|
| `G4_ROOT` | Geant4 **install** dir (contains `lib\cmake\Geant4`, `bin\`) | `E:\GEANT4\geant4-v11.4.1-install` |
| `G4LEDATA` | dataset dir that ends in `G4EMLOW8.8` (DNA data) | `E:\GEANT4\geant4-data\G4EMLOW8.8` |
| `DEV_DIR` | dev root, only used to find vcpkg | `C:\Users\rtonneau\DEV` |
| `QT5_PREFIX` | prefix that contains `lib\cmake\Qt5Core` (e.g. a conda env `...\envs\qt5\Library`) | `E:\miniconda3\envs\qt5\Library` |

Also needed: Visual Studio with the C++ workload (MSVC, `vcvars64.bat`), CMake >= 3.16, Ninja, and
vcpkg (`%DEV_DIR%\vcpkg`, only if the Geant4 install was built with vcpkg libraries such as HDF5; its
`installed\x64-windows\bin` must be on PATH at run time).

Checks:
```powershell
Test-Path "$env:G4_ROOT\lib\cmake\Geant4\Geant4Config.cmake"     # must be True
Test-Path "$env:G4LEDATA\dna"                                    # must be True (G4EMLOW8.8)
Test-Path "$env:QT5_PREFIX\lib\cmake\Qt5Core"                    # must be True
```
Geant4 must be the version in `../README.md` (11.4.1, data set G4EMLOW 8.8); other versions change the
reference numbers. If `G4_ROOT` or `DEV_DIR` is unset, stop and ask the user (the user-level setup
script is `~/dotfiles/scripts/setup-dev-env.ps1`); never guess a path.

## 2. Build (copy-paste, run from `validation/geant4/g4rs_ref`)

Why this is not plain `cmake`: PowerShell / Git Bash have no MSVC environment (issue A), and the Geant4
install was built with Qt, so `find_package(Geant4)` demands Qt5 even for a batch program (issue B).
Use a **fresh, untracked build directory** (`build-msvc`); never reuse an old `build/` (stale caches).

```powershell
$ErrorActionPreference = 'Stop'
$vc = (Get-ChildItem "C:\Program Files\Microsoft Visual Studio\*\*\VC\Auxiliary\Build\vcvars64.bat" |
       Select-Object -Last 1).FullName
$qt = $env:QT5_PREFIX -replace '\\','/'
$tc = "$env:DEV_DIR/vcpkg/scripts/buildsystems/vcpkg.cmake" -replace '\\','/'
$g4 = "$env:G4_ROOT/lib/cmake/Geant4" -replace '\\','/'
cmd /c "`"$vc`" >nul 2>&1 && cmake -S . -B build-msvc -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_TOOLCHAIN_FILE=$tc -DGeant4_DIR=$g4 -DQt5Core_DIR=$qt/lib/cmake/Qt5Core -DCMAKE_PREFIX_PATH=$qt && cmake --build build-msvc"
if ($LASTEXITCODE -ne 0) { throw "build failed" }
```
Success = last line `Linking CXX executable g4rs_ref.exe`. `vswhere.exe is not recognized` printed by the
bat file is harmless. "`CMAKE_TOOLCHAIN_FILE` unused" is harmless. Do not commit `build-msvc/`.

## 3. Run environment (needed in every new shell)

```powershell
$env:G4LEDATA = "<dir ending in G4EMLOW8.8>"       # geant4.bat does NOT set it
$env:PATH = "$env:G4_ROOT\bin;$env:QT5_PREFIX\bin;$env:DEV_DIR\vcpkg\installed\x64-windows\bin;$env:PATH"
```
Geant4, Qt and vcpkg DLLs must all be on PATH, otherwise the exe exits immediately with a missing-DLL
error (exit code `-1073741515`, 0xC0000135). Redirect output to a log file (`*> run.log`): Geant4 prints
thousands of lines.

## 4. Commands and smoke tests

Run from `validation/geant4/g4rs_ref`. Smoke tests first (outputs to a temp dir, not to
`validation/reference`), check exit code 0 **and** non-empty data rows, then production.

| Mode | Smoke test | Production (see `../README.md` for arguments and file names) |
|---|---|---|
| `xs` | `.\build-msvc\g4rs_ref.exe xs $env:TEMP\xs.csv` (624 rows) | same, output in `validation/reference/` |
| `mfp` | `... mfp $env:TEMP\mfp.csv 2000 1` (~1.4 s) | `1000000 20261` |
| `track` | `... track $env:TEMP\t.csv 20 1 100` (~2.4 s) | `10000 20262 100` |
| `xslow`, `slow` | `... slow $env:TEMP\s 1 1000:1` | spec in `../README.md` |
| `physchem` | `... physchem $env:TEMP\p.csv 1 1000:200` (~10 s) | `20261019 1000:20000,10000:2000,100000:200,900000:20` |

After a `physchem` or `slow` run check: all requested energies are present (stderr has one
`physchem: <E> eV done` line per energy), `n_other_process` / `n_other_secondary` are 0 for `slow`.
Files are flushed after each energy, so **a crash leaves a valid-looking partial CSV: never commit
one**. Provenance values to record: compiler = `cl` banner (run `cl` inside the vcvars shell), OS
version, `git rev-parse HEAD`, whether the generator had uncommitted changes, exact command line, seed.

Timing (author's machine, serial, one core): `mfp` 1e6 events/energy ~10 min, `track` 1e4 ~20 min,
`physchem` ~0.025 s per keV of primary energy (+~4 s start-up), production list ~17 min. Run long jobs in
the background and poll the log, do not use a short tool timeout.

## 5. Symptom -> fix (all fixes already in the checked-in source unless marked BUILD)

| # | Symptom | Cause | Fix |
|---|---|---|---|
| A | `cl`/`ninja` not found | no MSVC environment in the shell | BUILD: wrap configure+build in `vcvars64.bat` (section 2) |
| B | CMake: `Could not find ... Qt5Core` | Geant4 install built with Qt | BUILD: `-DQt5Core_DIR` and `-DCMAKE_PREFIX_PATH` to `QT5_PREFIX` |
| C | cache contains `Qt5Core_DIR-NOTFOUND` or stale `C:/DEV/vcpkg` | old build dir | BUILD: new build dir, not patching the cache |
| D | `Remove-Item -Recurse` rejected as protected path | quirk when combined with `cmd /c` in one command | run deletions as a separate command |
| E | exit `-1073741515` at start | DLLs not on PATH | section 3 |
| F | `xs`: access violation (0xC0000005) after the banner | `rm->Initialize()` does not build physics tables | call `rm->BeamOn(0)` before `G4EmCalculator` (done) |
| G | `de0001 ... No de-excitation for Z= 8 shellIndex= 0` (exit 0xC0000409) | see 5.1 | `G4LossTableManager::Instance()->SetAtomDeexcitation(nullptr)` after `Initialize()`, **in all modes** (done) |
| H | `physchem`: access violation in `G4ITModelManager::Initialize` | see 5.2 | `param->SetTimeStepModel(G4ChemTimeStepModel::IRT)` before `Initialize()` (done) |
| I | `physchem`: only `e_aq`, or `sum_r2 = 0` | wrong snapshot hook, see 5.3 | snapshot in `UserPreTimeStepAction` (done) |
| J | `physchem`: ~1 `e_aq` per event at 1 keV (expected ~40) | secondaries killed | stacking returns `fUrgent` in physchem mode (done) |
| K | `physchem` 900 keV: exit `-1073741571` (0xC00000FD stack overflow) | rare, seed dependent, cause not traced | `/STACK:536870912` in `CMakeLists.txt` (done); rerun completed |

### 5.1 K-shell `de0001` (issue G)
`G4EmDNABuilder` always registers a `G4UAtomicDeexcitation`. `G4DNABornIonisationModel2` (and the
Emfietzoglou / Born models) call `GetAtomicShell()` and `GenerateParticles()` for K-shell ionisation
whenever the pointer is non-null, without checking `IsFluoActive()`. With `SetFluo(false)` the transition
tables are never loaded, so `Shell()` fails. Loading them would not give "off" (`GenerateParticles`
ignores the flag). Nulling the pointer makes the models skip de-excitation: the K-shell binding energy
stays local, which is the intended reference ("fluo/auger off"). Keep `SetFluo/SetAuger/SetAugerCascade/
SetPixe(false)` too. For a program that wants de-excitation ON use `SetFluo(true)` + `SetAuger(true)`
instead. A short smoke test can miss this (rare event): test with several hundred events.

### 5.2 Chemistry needs a time-step model (issue H)
`G4EmParameters::GetTimeStepModel()` defaults to `Unknown`, so `G4EmDNAChemistry_option3` registers no
step model (and registers Brownian transport for every non-IRT model). Stack:
`G4DNAChemistryManager::Run -> G4Scheduler::Process -> G4ITModelProcessor::Initialize ->
G4ITModelManager::Initialize`. The reference example `examples/extended/medical/dna/chem6` (`beam.in`)
sets `/process/chem/TimeStepModel IRT` before `/run/initialize`; do the same in code, before
`rm->Initialize()` (the physics list reads it in `ConstructProcess`).

### 5.3 Species list at 1 ps (issue I)
- `G4UserTimeStepAction::StartProcessing()` runs before tracks are merged into the main list: it sees
  nothing.
- At the first `NewStage()` the list still holds undissociated `H2O^0`, `H2O^1`, `e_aq^-1`. The
  dissociation (`H2O^1 -> H3O^1 + °OH^0`, excitation channels) happens in zero-length steps at t = 1 ps.
- Snapshot = `G4ITTrackHolder::Instance()->GetMainList()` (skip `H2O`) at the first
  `UserPreTimeStepAction()` with `G4Scheduler::GetTimeStep() > 0`, then `G4Scheduler::Stop()`. Do not use
  `G4MoleculeCounter`. `SetEndTime(2 ps)` must exceed the 1 ps birth time; `SetUserAction` is called after
  `rm->Initialize()`.
- Run-manager actions must forward `BeginOfRunAction/EndOfRunAction` to `G4DNAChemistryManager`
  (`GetInstanceIfExists()`), and `Stacking::NewStage()` calls `G4DNAChemistryManager::Instance()->Run()`
  when the stack is empty (as chem6).

### 5.4 CSV species names
`physchem.csv` holds native `G4Molecule::GetName()` = `<definition>^<charge>`: `H3O^1`, `°OH^0` (UTF-8
degree sign), `e_aq^-1`, `H^0`, `H_2^0`, `OH^-1`, `O^-1`, `°O^0`, `H2O2^0`, `HO_2°^0`, `HO_2^-1`, `O_2^0`,
`O_2^-1`, `O_3^0`. The Rust side maps them with `Species::from_geant4_name` (`crates/g4-chemistry`). A name that
panics there ("unknown Geant4 species") means a spelling to add in that function, not a CSV edit.

## 6. Debugging a crash without a debugger

No `cdb`/WinDbg is needed. Temporarily add to `g4rs_ref.cc` (Windows only, remove afterwards): `#include
<windows.h>`, `<dbghelp.h>`, `#pragma comment(lib,"dbghelp.lib")`, and in `main`
`SetUnhandledExceptionFilter(handler)` where the handler calls `SymInitialize`, `CaptureStackBackTrace`
and `SymFromAddr` and prints each frame to stderr. Geant4 DLL export names are enough to locate the
failing component (this found issue H in one run). A stack overflow (K) needs a vectored handler plus
`SetThreadStackGuarantee`. Watch for `\n` inside C++ string literals when generating code through a
script: it silently becomes a real newline and breaks compilation (`C2001`); prefer the Edit tool.

## 7. RR-001: M6 pre-IRT snapshot (2026-10-17, user-run only)

`physchem_prechem` is additive; do not change or rerun the existing `physchem` mode to obtain these data.
Its new post-zero-step hook captures after all water decays and before IRT initialization (README).
The mode was run on 2026-10-17 at the project lead's request (smoke and production passed all guards;
RR-001 is closed, see `RUN_REQUESTS.md`). Agent sessions otherwise never run it, not even the smoke test.

Build with section 2's recipe, replacing `-B build-msvc` and the build argument with the repository-root
scratch directory `.scratch/tests/2026-10-16__m6-diagnose-and-fix/g4-prechem-build` (use the appropriate
relative path from the working directory). This was verified with MSVC 19.50.35729, Ninja,
RelWithDebInfo and installed Geant4 11.4.1. No generator or CMake source contains machine paths.
The first compile succeeded but the sandbox rejected the vcpkg PowerShell post-link dependency copy
("The system cannot execute the specified program"); repeating `cmake --build` with host permission
completed the link. This is a build-environment issue, not a Geant4 runtime failure.

For the **project lead**, after setting section 3's runtime environment, from `validation/geant4/g4rs_ref`:

```powershell
$m6Exe = '../../../.scratch/tests/2026-10-16__m6-diagnose-and-fix/g4-prechem-build/g4rs_ref.exe'
& $m6Exe physchem_prechem "$env:TEMP/physchem_prechem_smoke_20261017.csv" 1 1000:200 *> "$env:TEMP/physchem_prechem_smoke_20261017.log"
if ($LASTEXITCODE -ne 0) { throw 'prechem smoke failed; keep the log, do not commit its CSV' }
```

The smoke output must be a **new** path (the mode refuses overwrite), with nonempty species rows and
`n_events=200`. Runtime guards require a zero step at global 1 ps, no preceding chemical reaction,
no remaining water, only alive 1 ps molecules and a snapshot for every event. If a guard fires, retain
the log for diagnosis; do not disable the guard or commit partial output.

For production, put the executable's build directory on PATH, then use the exact RR-001 command:

```powershell
$env:PATH = "$(Split-Path -Parent (Resolve-Path -LiteralPath $m6Exe));$env:PATH"
g4rs_ref physchem_prechem ../../reference/physchem_prechem.csv 20261020 1000:20000,10000:2000,100000:200,900000:20 *> "$env:TEMP/physchem_prechem_production.log"
if ($LASTEXITCODE -ne 0) { throw 'prechem production failed; do not commit partial output' }
```

Check four completion lines, all four requested energies and counts, 28 nonempty data rows, and the
pre-IRT marker before committing the **new** CSV and `provenance_m6_prechem.toml` (README's fields).
Do not overwrite any historical CSV/provenance. Then run only the paired ignored test named in RR-001.

## 8. RR-002: 900 keV spatial diagnostic (2026-10-20, user-run only)

The separately named `physchem_spatial_diagnostic` mode was compile-checked but never executed by the
agent. Configure section 2's recipe into
`.scratch/tests/2026-10-19__m6-900kev-spatial-rms-investigation/g4rs-ref-build`. MSVC 19.50.35729 and
Ninja compiled and linked it against installed Geant4 11.4.1.

A plain PowerShell configure selected unavailable NMake, so configuration was repeated inside
`vcvars64.bat`. Geant4's exported configuration also required Qt5; `QT5_PREFIX`/`Qt5Core_DIR` were
passed exactly as in section 2. No machine path was added to source or CMake.

The project lead sets section 3's runtime environment and runs the exact RR-002 command in README and
`RUN_REQUESTS.md`, using new output paths only. Require exit 0, five completion lines, the trailing
complete marker, exact five x 40 coverage and a nonempty long-form body before committing the CSV and
provenance. Do not run the paired ignored consumer until both are committed. If a runtime guard fires,
retain the log and partial scratch output for diagnosis; do not commit it or weaken the guard.

## 9. RR-003: H+H two-body SBS reference (2026-10-23)

Build the corrected `two_body_h_h` mode with sections 1-3. Its reaction callback must recognize
Geant4's canonical neutral-hydrogen name `H^0`; its CLI must reject an overflowing seed. A historical
scratch CSV produced before those checks is diagnostic-only and must not be promoted.

From `validation/geant4/g4rs_ref`, first use a new temporary path:

```powershell
g4rs_ref two_body_h_h "$env:TEMP/two_body_h_h_smoke_seed1_n1000.csv" 1 1000 *> "$env:TEMP/two_body_h_h_smoke_seed1_n1000.log"
if ($LASTEXITCODE -ne 0) { throw 'two-body smoke failed; retain log and do not run production' }
```

Require six ordered data rows, exact 1,000-pair coverage, monotonic reacted counts, a nonzero reaction
count by tau 16, seed 1 in the completion marker and exit 0. Then run the exact RR-003 production
command from `README.md`; it normally completes in seconds after Geant4 initialization. Require exit
0, six rows, exact 100,000-pair coverage, seed 202610231, monotonic counts and the final completion
marker. Do not commit the smoke CSV or any partial output. Write the specified provenance using the
clean generator commit and `generator_dirty = false`, commit both new reference files, and only then
run the ignored close-gate test.


## 10. RR-004: R04 IRT pair matrix (user-run only)

Use section 2's configure/build recipe with `-S validation/geant4/g4rs_ref`
from the repository root and build directory
`.scratch/tests/2026-10-27__pair-oracles-and-reference-gate/g4rs-ref-build`.
The equivalent verification command is `cmake --build` with that directory;
keep all build artifacts out of the source tree. If QT5_PREFIX is unset, obtain
its existing configured prefix from a known successful local CMakeCache.txt,
validate that it contains `lib/cmake/Qt5Core`, and supply it to the same recipe.
Do not hardcode that machine path in source or project configuration.

No agent execution of this mode is authorized, including smoke tests. The
project lead sets section 3's runtime environment, adds this build directory to
PATH and runs README's exact RR-004 command. An optional user smoke command is
`g4rs_ref pair_matrix_irt <new-scratch-output.csv> 1 100`; it is diagnostic-only.
Require exit 0, 144 ordered rows, eight completed channels and the trailing
marker before creating provenance. The mode rejects overwrites, invalid integer
arguments, incompatible Geant4 versions and unexpected sampled results.
Its 64-bit seed is explicitly decomposed for Ranecu; do not use `atol` or pass
20261027404 to Windows `long`. Preserve RR-003 and all old modes/fixtures.

Compile verification (2026-10-27): MSVC 19.50, Ninja, RelWithDebInfo and
installed Geant4 11.4.1 compiled and linked this mode. Initial sandbox
configuration failed with C1902 (PDB manager mismatch); the identical configure
and build outside the sandbox succeeded. No generator executable was run.
