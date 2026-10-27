# Ticket 05: geant4-pair-producer

**Model:** heavy
**Effort:** light

**Acceptance Criteria:**
- [ ] Add a new g4rs_ref mode for the R04 matrix using option3 IRT, exact channel parameters/units, output path, seed and pair count arguments, with complete-only CSV marker.
- [ ] Preserve every existing mode and fixture byte-for-byte; compile-check the new mode against installed Geant4 11.4.1 without running it.
- [ ] Add one or more new RR IDs with exact user command, schema, provenance fields and build notes.

**Files to Touch:**
- `validation/geant4/g4rs_ref/g4rs_ref.cc`
- `validation/geant4/README.md`
- `validation/geant4/BUILD_NOTES.md`
- `validation/geant4/RUN_REQUESTS.md`

**Verification Step:**

Run:
```bash
cmake --build validation/geant4/g4rs_ref/build-msvc
```

Expected:
The new mode compiles and links; no Geant4 executable is run.

**Notes:**

Use the project's documented CMake recipe and configured G4_ROOT to configure `build-msvc` before this build. Never overwrite or regenerate RR-003.
