# Ticket 01: extract-option3-reactions

**Model:** sonnet
**Effort:** high

**Acceptance Criteria:**
- [ ] `scripts/extract_reactions_from_cpp.py` reads `G4EmDNAChemistry_option3.cc` from a supplied Geant4 source root and emits exactly 72 reactions in `SetReaction` order, with IDs 1-72, source lines, section classes, ordered participants/products, executable rate literals, kinetic order and whether `SetReactionType` is called.
- [ ] `data/chemistry/reactions_option3.toml` contains those 72 executable entries, including the five known comment/rate discrepancies and reaction 53 exactly as `O3m + NoneM -> H3Op + O2m` at `2.66e3 / s`; it does not silently substitute comment values.
- [ ] The committed `cpp_reactions.json` fixture is reproducible byte-for-byte through the extractor's `--check` mode using the Geant4 11.4.1 release source.

**Files to Touch:**
- `scripts/extract_reactions_from_cpp.py`
- `data/chemistry/reactions_option3.toml`
- `crates/dna-chemistry/tests/fixtures/cpp_reactions.json`

**Verification Step:**

Run:
```powershell
python scripts/extract_reactions_from_cpp.py "$env:DEV_DIR/GEANT4/geant4-v11.4.1-source/geant4-v11.4.1" --check crates/dna-chemistry/tests/fixtures/cpp_reactions.json
```

Expected:
Exit code 0 and a message confirming that 72 extracted reactions match the committed fixture.

**Notes:**

Use only Python's standard library. Strip comments when parsing executable statements, but retain adjacent comments separately so the fixture can report the five rate discrepancies and the reaction-53 product discrepancy. Provenance names Geant4 release v11.4.1 and relative source paths/lines; the unpacked local tree has no Git metadata.
