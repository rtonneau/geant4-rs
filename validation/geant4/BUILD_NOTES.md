# g4rs_ref: build and runtime notes (Geant4 11.4.1, Windows)

Issues met while building and smoke-testing the generator for the first time (2026-10-15),
with the fix for each. Reuse for any new Geant4-DNA test program on this machine.

## Environment (this machine)

| Item | Value |
|---|---|
| `DEV_DIR` | `C:\Users\rtonneau\DEV` |
| `G4_ROOT` (install) | `E:\GEANT4\geant4-v11.4.1-install` |
| Datasets | `E:\GEANT4\geant4-data` (`GEANT4_DATA_DIR`), DNA needs `G4EMLOW8.8` |
| Compiler | MSVC (VS 18 Community), generator Ninja |
| vcpkg | `%DEV_DIR%\vcpkg` (HDF5 etc.). Old caches pointed at `C:/DEV/vcpkg`, which is stale |

## Build issues

### 1. `cl` / `ninja` not on PATH
PowerShell/Git Bash sessions have no MSVC environment.
Fix: run configure and build through `vcvars64.bat`:
```powershell
$vc='C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat'
cmd /c "`"$vc`" >nul 2>&1 && cmake -S . -B build2 -G Ninja ... && cmake --build build2"
```
(`vswhere.exe is not recognized` printed by the bat is harmless.)

### 2. `find_package(Geant4)` fails: `Could not find ... Qt5Core`
`Geant4Config.cmake` (install built with Qt support) requires Qt5 even for a batch program.
Fix: point CMake at the conda Qt5 used by the other projects:
```
-DQt5Core_DIR=E:/miniconda3/envs/qt5/Library/lib/cmake/Qt5Core
-DCMAKE_PREFIX_PATH=E:/miniconda3/envs/qt5/Library
```
Full configure used:
```
cmake -S . -B build2 -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo ^
  -DCMAKE_TOOLCHAIN_FILE=%DEV_DIR%/vcpkg/scripts/buildsystems/vcpkg.cmake ^
  -DGeant4_DIR=%G4_ROOT%/lib/cmake/Geant4 ^
  -DQt5Core_DIR=E:/miniconda3/envs/qt5/Library/lib/cmake/Qt5Core ^
  -DCMAKE_PREFIX_PATH=E:/miniconda3/envs/qt5/Library
```
CMake reports `CMAKE_TOOLCHAIN_FILE` as unused when the cache already has one; harmless here.

### 3. The old `build/` directory is unusable
Its cache has `Qt5Core_DIR-NOTFOUND` and the stale `C:/DEV/vcpkg` paths. Use a fresh
directory (`build2`) rather than patching the old cache.

### 4. Running the exe: environment
```powershell
$env:G4LEDATA = "E:\GEANT4\geant4-data\G4EMLOW8.8"
$env:PATH = "$env:G4_ROOT\bin;E:\miniconda3\envs\qt5\Library\bin;$env:DEV_DIR\vcpkg\installed\x64-windows\bin;$env:PATH"
```
`geant4.bat` does not set `G4LEDATA` (the line is commented out), so set it by hand.
Geant4 DLLs, Qt and vcpkg DLLs must all be on PATH.

### 5. Tooling quirk
`Remove-Item -Recurse` on a path built inside the same command that also called `cmd /c ...` was
rejected as a protected path (`/c`). Run deletions as a separate command.

## Runtime issues

### 6. `xs` mode: access violation (0xC0000005) right after the banner
`rm->Initialize()` does not build the physics tables, and `G4EmCalculator` needs them.
Fix: call `rm->BeamOn(0)` before using `G4EmCalculator`.

### 7. `mfp` / `track`: abort with `de0001` on K-shell ionisation
Symptom (exit code 0xC0000409, log tail):
```
G4Exception : de0001 issued by : G4AtomicTransitionManager::Shell()
No de-excitation for Z= 8  shellIndex= 0. AtomicShell not found - check if data are uploaded
```
Cause (verified in source):
- `G4EmDNABuilder` (via `G4EmBuilder::PrepareEMPhysics`) always registers a `G4UAtomicDeexcitation`.
- `G4DNABornIonisationModel2` (and `G4DNAEmfietzoglouIonisationModel`, `G4DNABornIonisationModel`)
  call `GetAtomicShell()` and `GenerateParticles()` for K-shell ionisation whenever
  `fAtomDeexcitation != nullptr`. They do not check `IsFluoActive()`.
- With `SetFluo(false)`, `G4UAtomicDeexcitation::InitialiseForNewRun` returns early, so the
  transition tables are never loaded and `Shell()` fails.
- Loading the tables manually would not give "off": `GenerateParticles(v, shell, Z, gCut, eCut)`
  never checks `isActive`, so fluorescence/Auger would be produced anyway.

Fix (in `main`, right after `rm->Initialize()`):
```cpp
G4LossTableManager::Instance()->SetAtomDeexcitation(nullptr);
```
Models read the pointer at their own initialisation (during BeamOn), so they see `nullptr` and
skip de-excitation: the K-shell binding energy stays local. This is the intended "fluo/auger off"
reference for geant4-rs M2. Keep `SetFluo/SetAuger/SetAugerCascade/SetPixe(false)` too.

Note for any DNA program with `G4EmDNAPhysics_option2`: if you want de-excitation ON, set
`SetFluo(true)` (and `SetAuger(true)` as `G4EmDNAPhysics` does) instead of the line above.

## Smoke-test results (not reference data)
- `xs`: 624 data rows (208 energies x 3 processes).
- `mfp` 2000 events x 9 energies: ~1.4 s. `n_other = 0`.
- `track` 20 events x 5 energies: ~2.4 s, dominated by 900 keV. `n_other = 0`.
- Rough extrapolation: 1e6 mfp events/energy ~10 min; 1e4 track events/energy ~20 min (unmeasured).

## To port back to `C:\Users\rtonneau\DEV\RUST\geant4-rs\validation\geant4\g4rs_ref`
- `g4rs_ref.cc`: the `BeamOn(0)` call in `xs` mode, the `SetAtomDeexcitation(nullptr)` line, and
  `#include "G4LossTableManager.hh"`.
- `README.md`: drop the "untested / never compiled" status; add the Qt5 and `G4LEDATA` notes above.
