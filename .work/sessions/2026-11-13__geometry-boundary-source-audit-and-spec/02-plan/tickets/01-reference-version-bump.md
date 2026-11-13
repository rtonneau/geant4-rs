# Ticket 01: reference-version-bump

**Model:** medium
**Effort:** medium

**Acceptance Criteria:**
- [ ] AGENTS.md and the docs name Geant4 v11.4.3 as the reference version
- [ ] The docs state that the existing RR-001…008 data and provenance were produced with 11.4.1 and are kept unmodified
- [ ] A note lists the files under processes/electromagnetic/dna that differ between 11.4.1 and 11.4.3, with any physics-relevant changes summarised
- [ ] No file under validation/reference/ is modified

**Files to Touch:**
- `AGENTS.md`
- `docs/02-cpp-reference-inventory.md`
- `docs/geant4-11.4.3-migration.md`
- `validation/geant4/README.md`

**Verification Step:**

Run:
```bash
git diff --name-only HEAD -- validation/reference; grep -n "11.4.3" AGENTS.md
```

Expected:
No reference paths listed, and AGENTS.md shows the 11.4.3 reference line.

**Notes:**

Stage only the ticket's files. The working tree holds unrelated uncommitted RR-008 work.
