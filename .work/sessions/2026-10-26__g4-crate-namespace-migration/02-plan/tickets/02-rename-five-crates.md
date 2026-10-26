# Ticket 02: rename-five-crates

**Model:** heavy
**Effort:** low

**Acceptance Criteria:**
- [ ] The only workspace packages are `g4-core`, `g4-data`, `g4-physics`, `g4-chemistry`, and `g4-sim`. Crate directories, path dependencies, dev profile keys, lockfile local identities, Rust imports and rustdoc paths use the new names. No third-party package version/checksum changes.
- [ ] Current examples, script fixture paths, CI commands and maintained docs use live `g4-*` paths; historical GPS/reviews/reference provenance stay unchanged. The guide includes an import migration table and documents intentional old schema/data names.
- [ ] `geant4-rs/dna-sim/run/v2` and `/v3`, `data/dna`, `G4RS_DATA_DIR`, CSV headers, species names and scientific data bytes remain unchanged. No algorithm, RNG, feature, dependency or compatibility-shim changes.
- [ ] Cargo metadata reports exactly the five new members; standalone physics and chemistry all-target checks, workspace fmt/Clippy/tests, and selected M2/M3/M6 gates pass with nonzero selected test counts.
- [ ] Fresh `g4-sim` physics and physchem captures compare against the archived S01 baselines in rename mode with exact scientific fields and verified lockfile-only local identity changes.

**Files to Touch:**
- `Cargo.toml`, `Cargo.lock`, `crates/dna-{core,data,physics,chemistry,sim}/` (renamed to corresponding `crates/g4-*`)
- `.github/workflows/ci.yml`, `scripts/check_crate_split.py`, `scripts/hand_physchem.py`, `scripts/hand_reactions.py` and other current executable references found by scoped search
- `README.md`, `docs/crate-separation.md` and current guides whose commands/imports must run after the rename

**Verification Step:**

Run:
```bash
cargo metadata --format-version 1 --no-deps --locked
cargo check -p g4-physics --all-targets --locked
cargo check -p g4-chemistry --all-targets --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test -p g4-sim --test geant4_reference --locked -- --ignored
cargo test -p g4-sim --test slowing_down_reference --locked -- --ignored
cargo test -p g4-sim --test physchem_reference --locked m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact
python -m unittest discover -s scripts -p test_check_crate_split.py
```

Expected:
Metadata contains exactly the five `g4-*` members. Every command exits 0; the maintained reference selectors execute 4 M2, 2 M3 and 1 M6 comparisons. Two new, exclusively created snapshot captures compare to the S01 physics/physchem baselines with no scientific difference.

**Notes:**

This is one coordinated rename; do not leave a half-renamed package set in a commit. Commit the coherent rename before final candidate capture so its recorded revision resolves to the renamed `Cargo.lock`. The snapshot checker defaults to `dna-sim`, so update its capture default and the retained strict tests when `g4-sim` becomes active. Do not mass-replace `dna` in schema IDs, data paths, model names, environment variables or historical records. Do not run extractors into existing fixture paths. S03, not this ticket, adds optional chemistry.
