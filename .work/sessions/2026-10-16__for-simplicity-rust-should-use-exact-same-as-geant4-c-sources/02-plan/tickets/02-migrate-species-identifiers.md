# Ticket 02: migrate-species-identifiers

**Model:** opus
**Effort:** high

**Acceptance Criteria:**
- [ ] Rust variants use chemistry casing wherever syntax permits: `OH`, `OHm`, `e_aq`, `H2O2`, `HO2`, `HO2m`, and `Oxy`; narrowly scoped lint allowances document the deliberate naming exception.
- [ ] All Rust call sites use the renamed variants and `chemistry_name()`; the temporary `tag()` and `from_tag()` APIs are removed.
- [ ] Embedded species and dissociation TOML use canonical Geant4 configuration keys, including `°OH` and `HO2°`, and all chemistry loaders consume them strictly.
- [ ] CSV, scoring, CLI-facing output, fixtures, and hand crosschecks emit or expect canonical names only.

**Files to Touch:**
- `crates/dna-core/src/species.rs`
- `crates/dna-chemistry/src/`
- `crates/dna-chemistry/tests/`
- `crates/dna-sim/src/physchem_scoring.rs`
- `crates/dna-sim/tests/`
- `data/chemistry/species_option3.toml`
- `data/chemistry/dissociation_option3.toml`
- `scripts/hand_physchem.py`

**Verification Step:**

Run:
```bash
cargo test -p dna-core && cargo test -p dna-chemistry && cargo test -p dna-sim
```

Expected:
All three crates pass their tests; generated and parsed species names are canonical, and no old Rust variant or generic tag API remains.

**Notes:**

Do not change diffusion coefficients, charges, radii, reaction products, reference CSV files, or charged Geant4 runtime-name parsing. `°` cannot appear in a Rust identifier, so variants `OH` and `HO2` expose `°OH` and `HO2°` through `chemistry_name()`.
