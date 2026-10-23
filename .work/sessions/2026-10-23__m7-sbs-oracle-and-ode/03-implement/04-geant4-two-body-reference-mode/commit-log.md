# Ticket 04: geant4-two-body-reference-mode

**Status:** ✅ Done

## Local Test Result

PASS (compile/link only; the generator was not run).

- `cmake --build .scratch/tests/2026-10-23__m7-sbs-oracle-and-ode/g4-two-body-build-no-ui`
  rebuilt `g4rs_ref.cc` and linked `g4rs_ref.exe` successfully with MSVC 19.50,
  `RelWithDebInfo`, and `Geant4_DIR=E:/GEANT4/geant4-v11.4.1-install/lib/cmake/Geant4`.
- A source-contract check verified the new CLI, all six tau values, 0.2 nm separation,
  stable CSV header, exclusive output creation, SBS selection, watched times, and trailing
  completion marker.
- `git diff --check -- validation/geant4/g4rs_ref/g4rs_ref.cc` passed.

## Review Notes

Checked every acceptance criterion against the final diff. The new `two_body_h_h` mode is
additive and keeps all existing mode names and argument parsing unchanged. It creates two option3
H molecules 0.2 nm apart, selects SBS, records cumulative H+H reactions at the six registered
observation times, validates every history/row, and refuses output overwrite. Review of Geant4
11.4.1 `G4Scheduler` confirmed that watched times become scheduler stop times and the post-step
action observes each capped boundary. Replaced a heap-allocated chemistry gun with process-lifetime
static storage and removed unused state after review.

## Blockers / Challenges

The ticket's literal verification command uses invalid PowerShell syntax for `ProgramFiles(x86)`,
and this shell has no `QT5_PREFIX`. Per `validation/geant4/BUILD_NOTES.md`, the exported Geant4
package requires Qt metadata even for this batch executable. Configuration therefore used
scratch-only imported-interface Qt target stubs; compilation and linking still used the installed
Geant4 11.4.1 headers and libraries. A normal user build should set `QT5_PREFIX` and use the
documented build command. `G4LEDATA` is also unset, so no generator mode was run, as required.

## Commits

- efd367c feat: add Geant4 H+H reference mode (ticket 04)

## Time Spent

14m (ticket-start.js to ticket-complete.js)
