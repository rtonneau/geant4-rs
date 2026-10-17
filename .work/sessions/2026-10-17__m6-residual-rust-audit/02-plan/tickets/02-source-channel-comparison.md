# Ticket 02: source-channel-comparison

**Model:** opus
**Model (Jev):** opus-5.5, confidence 0.33
**Effort:** high

**Acceptance Criteria:**
- [ ] Rust channel tables, dissociation fractions, recombination and the process-to-channel mapping are compared with Geant4 11.4.1 `G4DNAWaterDissociationDisplacer`, `G4DNAWaterExcitationStructure`, `G4DNAElectronHoleRecombination`, `G4EmDNAChemistry_option3.cc` and the shell/level/DA to channel mapping, all from the `.cc` files.
- [ ] Each row has `file:line` references on both sides and is marked identical / differs / undecidable.
- [ ] Thermalisation and solvation are listed as not audited unless the ticket-01 relations point there.
- [ ] The Geant4 repository is not modified.

**Files to Touch:**
- `docs/08-roadmap.md` (draft table, finalised in ticket 04)

**Verification Step:**

Run:
```bash
git diff --stat
```

Expected:
Only docs changed; every table row cites both a Rust and a Geant4 `file:line`.

**Notes:**

Reference sources are under `source/processes/electromagnetic/dna/{management,models}/src/` of the Geant4 checkout named by `geant4SourcePath`. Do not guess; read the `.cc`, not just the `.hh`. Run after ticket 01 so the relations can steer the scope.
