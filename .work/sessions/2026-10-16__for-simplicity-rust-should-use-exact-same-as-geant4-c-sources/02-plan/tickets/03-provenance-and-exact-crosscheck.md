# Ticket 03: provenance-and-exact-crosscheck

**Model:** sonnet
**Effort:** medium

**Acceptance Criteria:**
- [ ] The C++ extraction fixture and crosscheck compare all 15 configuration keys directly, without a Rust alias translation layer.
- [ ] Chemistry data documentation, scientific inventory, units/reproducibility notes, glossary, and ADR consistently describe canonical species naming and the legal-identifier boundary.
- [ ] No documentation claims ASCII aliases are canonical, and the four bookkeeping configurations remain explicitly out of `Species` scope.
- [ ] Workspace formatting, linting, and tests pass.

**Files to Touch:**
- `scripts/extract_physchem_from_cpp.py`
- `crates/dna-chemistry/tests/common/mod.rs`
- `crates/dna-chemistry/tests/cpp_crosscheck.rs`
- `crates/dna-chemistry/tests/fixtures/cpp_extract.json`
- `data/chemistry/README.md`
- `docs/02-cpp-reference-inventory.md`
- `docs/08-roadmap.md`
- `docs/10-data-requirements.md`
- `docs/units.md`
- `docs/reproducibility.md`
- `.work/GLOSSARY.md`
- `.work/adr/0001-canonical-geant4-chemistry-names.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected:
Formatting and clippy complete without diagnostics, and every workspace test passes.

**Notes:**

Use the checked-out Geant4 v11.4.1 tree as the source of truth. Do not modify that tree or regenerate committed validation CSV files. If the extraction fixture is regenerated, verify that only species-name representation changed and that all 15 entries remain present.
