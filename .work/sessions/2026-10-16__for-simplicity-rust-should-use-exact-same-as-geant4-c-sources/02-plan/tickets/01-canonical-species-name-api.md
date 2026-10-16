# Ticket 01: canonical-species-name-api

**Model:** sonnet
**Effort:** medium

**Acceptance Criteria:**
- [ ] `Species::chemistry_name()` returns each of the 15 byte-for-byte Geant4 configuration keys, including `°OH`, `e_aq`, and `HO2°`.
- [ ] `Species::from_chemistry_name()` accepts exactly those canonical keys and round-trips all 15 species.
- [ ] A separately named compatibility parser accepts the former ASCII aliases without weakening strict canonical parsing.
- [ ] Charged `G4Molecule::GetName()` parsing remains unchanged.

**Files to Touch:**
- `crates/dna-core/src/species.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-core species
```

Expected:
All `dna-core` species tests pass, including exact-name, strict-parser, compatibility-parser, and charged-name cases.

**Notes:**

Add the canonical API before migrating callers so this commit remains independently buildable. Keep `tag()` and `from_tag()` only as temporary forwarding APIs for the next ticket. The canonical keys in `Species::ALL` order are `H3Op`, `°OH`, `OHm`, `e_aq`, `H`, `H2`, `H2O2`, `HO2°`, `HO2m`, `Oxy`, `Om`, `O2`, `O2m`, `O3`, `O3m`.
