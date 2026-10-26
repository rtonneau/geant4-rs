# Ticket 01: validate-table-domains

**Model:** heavy
**Effort:** high

**Acceptance Criteria:**
- [ ] `DnaFile::load_from` and direct model constructors reject missing or extra cross-section components; Born ionisation/excitation use five channels and Sanche uses nine, as checked against Geant4 11.4.1 source.
- [ ] Zero-column angular and short-column ionisation blocks return `DataError` without indexing panics. The `[1.0, 1.0000000000001]` endpoint regression returns finite bounded interpolation or a typed error.
- [ ] Raw values, scaled component values, node sums and queried partial/total values are finite and nonnegative. A table whose scaling or sum overflows fails construction; valid data retain existing interpolation and total/partial semantics.
- [ ] Focused regressions and property tests exercise both too-few and too-many components, malformed blocks, narrow grids and overflow.

**Files to Touch:**
- `crates/g4-data/src/catalog.rs`
- `crates/g4-data/src/xs.rs`
- `crates/g4-data/src/blocks.rs`
- `crates/g4-physics/src/models/born_excitation.rs`
- `crates/g4-physics/src/models/born_ionisation.rs`
- `crates/g4-physics/src/models/sanche.rs`
- `crates/g4-data/tests/real_tables.rs`
- `crates/g4-physics/tests/windows.rs`

**Verification Step:**

Run:
```bash
cargo test -p g4-data -p g4-physics --locked
```

Expected:
All unit, integration and property tests pass; malformed fixtures return typed errors and the committed tables still load.

**Notes:**

Use catalog `DnaFile::n_components()` for exact file shapes and enforce matching counts again in direct model constructors. Preserve the source's five-shell/nine-level interpretation and existing valid interpolation formulas. Test errors rather than panic messages.
