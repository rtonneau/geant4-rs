# Ticket 04: resolve-m6-gate-or-request-diagnostics

**Model:** opus
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] The final M6 gate result is recorded with the unchanged pre-registered count and RMS tolerances.
- [ ] Every remaining Rust/Geant4 difference is listed in `docs/09-open-questions-and-risks.md` with source paths, observed effect, and whether it is intentional, a generator-semantic mismatch, or unresolved.
- [ ] `docs/08-roadmap.md` marks M6 closed only if the gate is scientifically valid and passes; otherwise it states the exact evidence-backed open condition.
- [ ] If aggregate data cannot resolve that condition, a new, separately named Geant4 diagnostic mode and ignored consuming test are added, compile-checked, documented, and logged in `validation/geant4/RUN_REQUESTS.md`; the session ends `awaiting data`.
- [ ] Workspace formatting, linting, and tests pass.

**Files to Touch:**
- `docs/08-roadmap.md`
- `docs/09-open-questions-and-risks.md`
- `validation/geant4/g4rs_ref/g4rs_ref.cc`
- `validation/geant4/README.md`
- `validation/geant4/BUILD_NOTES.md`
- `validation/geant4/RUN_REQUESTS.md`
- `crates/dna-sim/tests/physchem_reference.rs`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected:
All three commands exit 0. The ignored Geant4 gate is reported separately as passing, evidence-backed open, or awaiting newly requested data.

**Notes:**

Never loosen a tolerance, overwrite a reference file, or run a Geant4 generator. A new mode must accept output path, seed, and statistics on the command line and leave every existing mode unchanged. If no new data are needed, do not create speculative generator code.
