# Ticket 01: optional-chemistry-simulation

**Model:** heavy
**Effort:** high

**Acceptance Criteria:**
- [ ] `g4-sim` defaults to `chemistry = ["dep:g4-chemistry"]`; the package-scoped no-default build compiles physical orchestration without chemistry.
- [ ] Existing physical `g4_sim::run` paths, raw `PhysChemEvent` recording and handoff CSV work in both profiles. Coupled paths, scoring, diagnostics and associated errors exist only with `chemistry`; default API and scientific output remain unchanged.
- [ ] A real `physics_only` test runs eight ordered 1 keV histories with seed 20261026, recorded raw handoff, conserved ledgers and unchanged CSV header. A required-feature `chemistry_feature` test exercises root species exports and coupled 1 ps output. Physical tests remain selected without the feature; chemistry tests run by default.
- [ ] Snapshot `--physics` works in both profiles; `--physchem` without chemistry exits nonzero with a clear unsupported-feature message. Default physics and physchem captures retain exact scientific payloads.

**Files to Touch:**
- `crates/g4-sim/Cargo.toml`
- `crates/g4-sim/src/lib.rs`
- `crates/g4-sim/src/error.rs`
- `crates/g4-sim/src/run.rs`
- `crates/g4-sim/src/run/chemistry.rs` (new)
- `crates/g4-sim/src/physchem_scoring.rs`
- `crates/g4-sim/src/spatial_diagnostic.rs`
- `crates/g4-sim/examples/crate_split_snapshot.rs`
- `crates/g4-sim/tests/physics_only.rs` (new)
- `crates/g4-sim/tests/chemistry_feature.rs` (new)
- `crates/g4-sim/tests/physchem_a.rs`, `physchem_b.rs`, `physchem_audit.rs`, `physchem_reference.rs`, `spatial_diagnostic.rs` (target gating in manifest)

**Verification Step:**

Run:
```bash
cargo test -p g4-sim --no-default-features --all-targets --locked
cargo test -p g4-sim --test chemistry_feature --locked
cargo test --workspace --locked
```

Expected:
All exit 0. The no-default run executes physical tests, including eight-history `physics_only`; the default run executes `chemistry_feature` and the existing coupled tests. No test family silently disappears. Compare new default captures with the immutable pre-S01 baseline using `scripts/check_crate_split.py --compare` under the same toolchain/platform.

**Notes:**

Use the exact public paths and test parameters in Task 4 of `docs/superpowers/plans/2026-10-26-physics-chemistry-crate-separation.md`. Move only coupled functions and `PhysChemRunOutput` to the private child module; retain existing private shared helpers, batch order and RNG streams. Do not add a chemistry dev-dependency or change numerical behavior. Commit this tested slice before ticket 2.
