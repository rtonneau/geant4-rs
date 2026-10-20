# Ticket 04: add-geant4-spatial-diagnostic-mode

**Status:** ✅ Done

## Local Test Result

Configured the requested scratch build against `${G4_ROOT}/lib/cmake/Geant4` with MSVC 19.50.35729, Ninja, and the repository vcpkg toolchain. Then ran the verification build through `vcvars64.bat`:

`cmake --build .scratch/tests/2026-10-19__m6-900kev-spatial-rms-investigation/g4rs-ref-build`

Passed: `g4rs_ref.cc` compiled and `g4rs_ref.exe` linked against Geant4 11.4.1. The executable was never run.

## Review Notes

Added the isolated `physchem_spatial_diagnostic <out.csv> <histories_per_seed> <seed,...>` mode. Existing `physchem` and `physchem_prechem` branches, arguments, and writers are unchanged. The new mode is fixed at 900 keV, refuses overwrite, emits the Ticket 02 long-form schema plus completion/provenance comments, records physical handoffs, initial solvated electrons and water states, per-pass daughter births, and final alive pre-IRT survivors, and supplies explicit empty categories.

Reviewed the hooks against Geant4 11.4.1 source. Initial molecules are captured in the first zero-step pre-callback after the main list is populated; daughters are diffed after each zero step; final survivors are captured at global 1 ps before positive-time IRT selection. Track IDs, physical parents, source ordinals, zero-time passes, and per-mother processing ordinals remain within-implementation identifiers. Runtime guards reject missing lineage, wrong time/step, reactions, incomplete snapshots, and unknown water states. `CMakeLists.txt` required no change because its existing Geant4 link and Windows stack configuration cover the new mode.

## Blockers / Challenges

The repository-local `.claude/.claude-project.json` referenced by the global instructions is absent, but the validated `DEV_DIR` and `G4_ROOT` environment variables resolved the documented Geant4 11.4.1 source/install paths. The first CMake attempt selected unavailable NMake; the scratch directory was preserved under a failure-specific name and reconfigured with the installed MSVC/Ninja toolchain. Qt5 was resolved from the same local prefix documented in `validation/geant4/BUILD_NOTES.md`. No generator run was attempted.

## Commits

- 6a70b55 feat: add Geant4 spatial diagnostic mode (ticket 04)

## Time Spent

7m (ticket-start.js to ticket-complete.js)
