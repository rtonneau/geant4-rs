# Ticket 01: reference-contract

**Acceptance Criteria:**
- [ ] Resolve 11.4.1 versus 11.4.3 explicitly; read model, stepper, reaction and example .cc sources and record paths and versions.
- [ ] Specify synchronization, diffusion, unary and pair reactions, contact handling, placement, finite search horizons and supported ranges.
- [ ] Predefine statistical endpoints, tolerances, multiplicity and resource limits; supersede ADR 0003 for production only; preserve historical evidence.

**Files to Touch:**
- `docs/08-roadmap.md`
- `docs/07-validation-strategy.md`
- `.work/adr/`
- `validation/geant4/README.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected:
All three commands exit 0. Source-review checklist has no unsupported semantic assumptions; all existing workspace checks pass.

**Notes:**
Add focused tests for the acceptance criteria above; register exact scientific gate commands as tests and reference consumers are introduced. No blanket ignored-test execution. Preserve existing reference data. Documentation-only ticket; resolve version provenance before implementation.
