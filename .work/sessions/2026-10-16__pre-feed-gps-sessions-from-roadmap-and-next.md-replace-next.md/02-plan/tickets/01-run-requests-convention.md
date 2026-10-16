# Ticket 01: run-requests-convention

**Model:** sonnet
**Effort:** medium

**Acceptance Criteria:**
- [ ] `validation/geant4/RUN_REQUESTS.md` exists, under 300 lines, path-independent (no machine paths).
- [ ] It states the rule: gps sessions add new C++ modes or programs under `validation/geant4/` (existing modes, arguments and output formats untouched), may compile-check them, never run them.
- [ ] It defines the entry format: id, requesting session/seed, mode/program, exact command line, seed, statistics, expected output files in `validation/reference/` (new names only), provenance file to produce, consuming `#[ignore]`d Rust test, status (`awaiting run` / `data committed` / `closed`).
- [ ] It states the gate flow: producing session ends with the gate "awaiting data"; a paired `*-close-gate` seed finishes after the user commits the data and provenance.
- [ ] `validation/geant4/README.md` and AGENTS.md (section "Geant4 reference data and generators") link to it.
- [ ] A one-line note that no request exists yet (the file starts with an empty log table).

**Files to Touch:**
- `validation/geant4/RUN_REQUESTS.md`
- `validation/geant4/README.md`
- `AGENTS.md`

**Verification Step:**

Run:
```bash
wc -l validation/geant4/RUN_REQUESTS.md && grep -n "RUN_REQUESTS" AGENTS.md validation/geant4/README.md && git diff --stat -- validation/reference
```

Expected:
Under 300 lines; at least one match in each of AGENTS.md and README.md; empty diff for `validation/reference`.

**Notes:**

AGENTS.md rules to respect: never overwrite reference files or existing generator modes; provenance needs Geant4 version, data sets, constructors, exact command, seed, date, generator commit, compiler and platform. Keep the AGENTS.md edit to one or two lines. See `.work/sessions/*pre-feed*/01-grill/resume.md` for the agreed convention.
