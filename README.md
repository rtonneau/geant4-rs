# geant4-rs

A native Rust implementation of selected Geant4-DNA electron transport, physicochemistry and isolated chemistry kernels in liquid water. It is a research implementation, not a Geant4 binding or a complete radiolysis application. Proton transport, multispecies chemistry scheduling, time-dependent G-values and a supported CLI remain to be implemented.

## Layout

- `crates/dna-core` — units, reproducible RNG streams, shared physicochemical event records and provenance types.
- `crates/dna-data` — readers and interpolation for the committed G4EMLOW 8.8 DNA tables.
- `crates/dna-physics` — option2 electron models, secondary cascades and energy accounting.
- `crates/dna-chemistry` — chemical species, the 1 ps physicochemical stage, option3 reaction data, diffusion and isolated pair kernels.
- `crates/dna-sim` — independent-history orchestration, scoring and JSON output.
- `data/` — committed Geant4 tables with SHA-256 manifest; `validation/reference/` — committed Geant4 comparison fixtures and provenance.
- `validation/gates.json` — versioned maintained-gate inventory; `scripts/run_validation_gates.py` — fail-closed evidence preflight and comparison runner.

## Checks and validation status

The supported compiler policy is the pinned Rust 1.97.0 toolchain in `rust-toolchain.toml`. CI is configured for Linux and Windows to run format, Clippy, workspace tests and the required-data tier. Local Windows checks do not verify Linux CI; platform results are unverified until their jobs run.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
python scripts/run_validation_gates.py --tier required
```

The required tier verifies the committed data and reference fixtures, including identities and SHA-256 hashes; missing required evidence fails. The maintained statistical selection comprises four M2 reference tests, two M3 reference tests and the M6 RR-001 pre-IRT 1 ps comparison. Run it explicitly with `python scripts/run_validation_gates.py --tier statistical`; it is not part of ordinary CI. The inventory records historical exclusions, including the post-zero-time M6 comparisons and scratch-producing spatial diagnostics. Do not run all ignored tests as a substitute for this selection.

M6 has a qualified historical closure. RR-002's five-seed spatial study reached its precision target and did not reproduce the earlier RMS excess, but its interval includes equality and does not prove equivalence. Its large CSV and provenance are local-only under `.scratch/rr-002-80/`, not committed validation fixtures or a CI dependency. See `docs/08-roadmap.md` and `docs/07-validation-strategy.md` for model limits, criteria and evidence details.
