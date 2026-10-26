# Ticket 01: capture-pre-refactor-evidence

**Model:** heavy
**Effort:** high

**Acceptance Criteria:**
- [ ] Add `crates/dna-sim/examples/crate_split_snapshot.rs` with `--physics` and `--physchem` profiles, schema `geant4-rs/crate-split-snapshot/v1`, seed `20261026`, history IDs 3–10, energies 100 eV, 1 keV and 10 keV, one thread and batch size 7. Record ordered physical ledgers, secondaries, radial and `PhysChemEvent` fields; the coupled profile also records species, statistics, summary JSON and CSV headers/rows. Encode scientific floats as exact `to_bits()` hex strings.
- [ ] Add `scripts/check_crate_split.py` with `--capture <new-directory> --package dna-sim` and `--compare <baseline.json> <candidate.json>`. Capture both profiles into a new exclusive ignored `target/crate-split-baseline/<revision>-<unique-run-id>/` directory. Compare exact scientific fields and input/toolchain identity; allow only individually named, checkout-validated build/revision metadata differences.
- [ ] Add checker tests: equal pair and allowlisted revision change pass; one-bit change, species order/header change, and input/toolchain change fail. Document commands, hashes, revision and capture location in `docs/crate-separation.md`.
- [ ] Run the baseline workspace gate and named maintained M2/M3/M6 ignored reference selectors. If a gate fails unexpectedly, record the failure and stop before species migration. Commit the evidence slice without changing runtime behavior or reference fixtures.

**Files to Touch:**
- `crates/dna-sim/examples/crate_split_snapshot.rs`
- `scripts/check_crate_split.py`
- `scripts/test_check_crate_split.py`
- `docs/crate-separation.md`

**Verification Step:**

Run:
```bash
python -m unittest discover -s scripts -p test_check_crate_split.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test -p dna-sim --test geant4_reference --locked -- --ignored
cargo test -p dna-sim --test slowing_down_reference --locked -- --ignored
cargo test -p dna-sim --test physchem_reference --locked m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact
```

Expected:
All commands exit 0 with expected cases selected. A capture of each profile succeeds and creates nonempty JSON in a new directory; the guide records revision, toolchain and input hashes.

**Notes:**

Read patterns in `crates/dna-sim/tests/determinism.rs` and `physchem_a.rs` without changing their expectations. Capture uses production entry points. Scientific changes are never allowlisted. Preserve data bytes and historical provenance; do not run Geant4 generators.
