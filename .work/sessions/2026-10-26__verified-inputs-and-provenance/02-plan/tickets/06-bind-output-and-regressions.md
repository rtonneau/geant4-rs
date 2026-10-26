# Ticket 06: bind-output-and-regressions

**Model:** heavy
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] Production `run`/`run_with_physchem` and output-writing APIs accept verified model/input context rather than a separately supplied manifest. Existing callers migrate at compile time.
- [ ] A separately named synthetic/unverified test path produces output with an explicit unverified marker and cannot claim a verified manifest identity. Custom chemistry output uses its actual semantic hash and embedded option3 source hash only when applicable.
- [ ] Changed bytes cannot produce a result with an old verified identity. Identical inputs relocated to another root produce the same semantic identity. Valid default summary values and RNG consumption match the pre-R02 baseline.
- [ ] Documentation explains changed library constructors, identity fields, model limits and deliberate Geant4 deviations. No chemistry dependency leaks into `g4-core`, `g4-data`, `g4-physics` or the physics-only simulation profile.
- [ ] Formatting, Clippy, full workspace tests, physics-only checks and the seven R01 maintained scientific gates pass.

**Files to Touch:**
- `crates/g4-sim/src/run.rs`
- `crates/g4-sim/src/run/chemistry.rs`
- `crates/g4-sim/src/lib.rs`
- `crates/g4-sim/tests/determinism.rs`
- `crates/g4-sim/tests/*.rs` (existing callers of output APIs)
- `crates/g4-sim/examples/*.rs` (existing callers of output APIs)
- `docs/reproducibility.md`
- `docs/03-architecture-comparison.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check -p g4-sim --no-default-features --all-targets --locked
cargo test -p g4-sim --no-default-features --all-targets --locked
python scripts/run_validation_gates.py --tier required
```

Expected:
Every command exits 0; the runner reports the four M2, two M3 and one M6 maintained gates passing, with no skipped required data.

**Notes:**

Change the run schema version when its provenance shape changes. Keep an explicitly named unverified escape hatch for synthetic tests only; its JSON must not look like a verified result. Preserve source-backed model coverage, default scoring fields and scientific reference fixtures. Record any intentional difference from Geant4 in the existing limitations documentation.
