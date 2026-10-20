# Ticket 04: add-geant4-spatial-diagnostic-mode

**Model:** heavy
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] `g4rs_ref` gains a separately named `physchem_spatial_diagnostic` mode; `physchem`, `physchem_prechem`, their arguments, physics settings, and output formats remain byte-for-byte behaviorally unchanged.
- [ ] The new mode records the same five boundaries, categories/species, identities/ordinals, moments, maxima, and radial bins as the Rust schema, with runtime guards for 900 keV, global 1 ps stage semantics, alive/killed state, and complete seed/history output.
- [ ] The mode accepts an output path and the preregistered seed/history design on the command line and writes one long-form CSV with embedded provenance comments sufficient for the paired consumer to reject partial or wrong-stage output.
- [ ] The generator compiles and links against the installed Geant4 11.4.1 using paths expanded from `.claude/.claude-project.json`; the executable is never run by the agent.

**Files to Touch:**
- `validation/geant4/g4rs_ref/g4rs_ref.cc`
- `validation/geant4/g4rs_ref/CMakeLists.txt`

**Verification Step:**

Run:
```bash
cmake --build .scratch/tests/2026-10-19__m6-900kev-spatial-rms-investigation/g4rs-ref-build
```

Expected:
The `g4rs_ref` target compiles and links successfully against Geant4 11.4.1; no generator executable is run.

**Notes:**

Resolve Geant4 paths only from the validated environment and project config. Reuse shared accumulation helpers where that does not alter existing modes. Lifecycle hooks must observe state without perturbing scheduler ordering. If a requested boundary cannot be observed safely, stop and document the source limitation rather than approximating it silently.
