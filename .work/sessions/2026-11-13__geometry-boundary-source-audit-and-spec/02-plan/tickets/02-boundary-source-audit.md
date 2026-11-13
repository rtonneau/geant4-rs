# Ticket 02: boundary-source-audit

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] docs/04 has a geometry-boundary section citing 11.4.3 path:line for: world-exit kill (G4Transportation), interaction length carried over and rescaled across boundaries (G4VEmProcess), solvation displacement clamped to 0.8 × safety (G4DNAOneStepThermalizationModel), dissociation product displacement clamped to 0.8 × safety plus the OUTSIDE_OF_MOTHER_VOLUME warning (G4DNAMolecularDissociation), molecules outside water killed (G4DNABrownianTransportation)
- [ ] It records the correction that the interaction length is not reset at a boundary
- [ ] Every cited line is re-checked against the 11.4.3 source

**Files to Touch:**
- `docs/04-scientific-algorithms.md`

**Verification Step:**

Run:
```bash
grep -c "11.4.3" docs/04-scientific-algorithms.md
```

Expected:
A count of at least 5.

**Notes:**

Only facts go here. Questions the source leaves open go in ticket 3.
