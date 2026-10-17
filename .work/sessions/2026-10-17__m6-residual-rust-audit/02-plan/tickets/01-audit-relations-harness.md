# Ticket 01: audit-relations-harness

**Model:** sonnet
**Model (Jev):** sonnet-5.5, confidence 0.3
**Effort:** high

**Acceptance Criteria:**
- [ ] New `crates/dna-sim/tests/physchem_audit.rs` with `#[ignore]`d tests; the header fixes the relations (charge balance with +1 e_aq for the thermalised primary, H and O budgets per channel) and the 4.5 sigma tolerance before any result is evaluated.
- [ ] The Geant4 side computes each relation from `physchem_prechem.csv` with the uncorrelated sum-of-variances bound; the limitation is printed in the table.
- [ ] The Rust side runs 5 extra master seeds at 1, 10, 100 and 900 keV and prints per-species mean +- seed spread and z versus 1/sqrt(N); no new public API, no production code touched.
- [ ] If 900 keV is too slow, 2 seeds are used there and the table says so.

**Files to Touch:**
- `crates/dna-sim/tests/physchem_audit.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-sim --test physchem_audit -- --ignored --nocapture
```

Expected:
Exit 0; a relation table and a multi-seed table for the four energies are printed; relation tolerances are in the file header and not edited after the run.

**Notes:**

Rust-run entry point and config are as in `physchem_reference.rs` (`master_seed` 20261017 is the existing seed; use other seeds). Never edit `validation/reference/`. Run `cargo fmt` and `clippy -D warnings` on the new file.
