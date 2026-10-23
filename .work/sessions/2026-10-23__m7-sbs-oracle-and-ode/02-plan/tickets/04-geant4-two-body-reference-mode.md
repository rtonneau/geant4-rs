# Ticket 04: geant4-two-body-reference-mode

**Model:** medium
**Model (Jev):** sonnet-5.5 (confidence 0.16)
**Effort:** high

**Acceptance Criteria:**
- [ ] Add a new, separately named H+H two-body reference mode to `g4rs_ref` with output path, seed, and sample-count CLI arguments; existing mode names and arguments remain unchanged.
- [ ] Use option3 reaction ID 1, `r0 = 0.2 nm`, and the shared six tau points; emit a stable CSV containing observation time/tau, total pairs, reacted count, and surviving count, with completion and coverage guards.
- [ ] Refuse to overwrite an existing output and preserve deterministic seed handling.
- [ ] Configure and compile-check against the installed Geant4 11.4.1 in the session scratch directory without running any generator mode.

**Files to Touch:**
- `validation/geant4/g4rs_ref/g4rs_ref.cc`

**Verification Step:**

Run:
```powershell
$tc = "$env:DEV_DIR/vcpkg/scripts/buildsystems/vcpkg.cmake" -replace '\\','/'; $g4 = "$env:G4_ROOT/lib/cmake/Geant4" -replace '\\','/'; $vc = & "$env:ProgramFiles(x86)/Microsoft Visual Studio/Installer/vswhere.exe" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath; $vc = "$vc/VC/Auxiliary/Build/vcvars64.bat"; cmd /c "`"$vc`" >nul 2>&1 && cmake -S validation/geant4/g4rs_ref -B .scratch/tests/2026-10-23__m7-sbs-oracle-and-ode/g4-two-body-build -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_TOOLCHAIN_FILE=$tc -DGeant4_DIR=$g4 && cmake --build .scratch/tests/2026-10-23__m7-sbs-oracle-and-ode/g4-two-body-build"
```

Expected:
The command exits 0 and links `g4rs_ref.exe`; the executable is not run.

**Notes:**

The project lead, not the agent, will run smoke and production commands after Ticket 5 registers them. Do not create or modify files under `validation/reference/`.
