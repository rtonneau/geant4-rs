# Ticket 01: document-m6-reference-state

**Model:** haiku
**Effort:** low

**Acceptance Criteria:**
- [ ] `docs/08-roadmap.md` records the first observed M6 gate failure exactly as `1000 eV H2O2: z = 111.6`, explains that the first assertion hides the rest of the comparison, and leaves the pre-registered tolerances unchanged.
- [ ] `docs/09-open-questions-and-risks.md` marks Q24 closed and points to `validation/geant4/BUILD_NOTES.md` section 5, issues G-K.
- [ ] No file under `validation/reference/` is modified.

**Files to Touch:**
- `docs/08-roadmap.md`
- `docs/09-open-questions-and-risks.md`

**Verification Step:**

Run:
```bash
git diff --check -- docs/08-roadmap.md docs/09-open-questions-and-risks.md
```

Expected:
Exit 0 with no output; the diff records the failure and Q24 closure without changing acceptance thresholds.

**Notes:**

This documentation commit precedes the diagnostic implementation because the gate header requires failures seen after inspecting the CSV to be recorded in docs/08. Do not alter C++ or reference evidence.
