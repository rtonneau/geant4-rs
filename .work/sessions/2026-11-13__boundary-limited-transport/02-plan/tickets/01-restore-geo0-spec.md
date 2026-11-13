# Ticket 01: restore-geo0-spec

**Model:** medium
**Effort:** medium

**Acceptance Criteria:**
- [ ] The PR #30 text lost between 4cf47d6 and HEAD is back: the docs/04 boundary audit, the docs/07 finite-volume criteria, the docs/09 G-B1/G-B2, the docs/08, reproducibility and migration-note edits, and `.work/adr/0005-finite-volume-enum.md`.
- [ ] Reconciled with the shipped Volume (Box/Tubs/Sphere, centre offset, `Option<Volume>` None = unbounded), with no loss of the current D.5 primitives section.

**Files to Touch:**
- `docs/04-scientific-algorithms.md`
- `docs/07-validation-strategy.md`
- `docs/08-roadmap.md`
- `docs/09-open-questions-and-risks.md`
- `docs/reproducibility.md`
- `docs/geant4-11.4.3-migration.md`
- `.work/adr/0005-finite-volume-enum.md`

**Verification Step:**

Run:
```bash
git diff 4cf47d6 HEAD --stat -- docs .work/adr
```

Expected:
Only intended differences remain (the primitives additions and the reconciliation edits); no PR #30 section is missing.

**Notes:**

Check the 4cf47d6..HEAD diff hunk by hunk and keep the newer primitives content.
