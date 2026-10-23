# Ticket 07: document-m7-two-body-irt

**Model:** fast
**Model (Jev):** haiku-5.5 (confidence 0.06)
**Effort:** high

**Acceptance Criteria:**
- [ ] Docs 03 and 04 describe the new module boundaries, formulas, outcome semantics, validation rules, and exact Geant4 11.4.1 source paths.
- [ ] Docs 08 and 09 record the delivered scope, V9 evidence, remaining risks, and deferrals without claiming the many-species scheduler, SBS, ODE, or Geant4 comparison is complete.
- [ ] Docs 10 and `reproducibility.md` record runtime data requirements, pre-registered regimes/thresholds, RNG ownership, and the verified 1/2/8-thread claim.
- [ ] The crate-level documentation no longer says diffusion and reactions are wholly absent and clearly limits what now exists.

**Files to Touch:**
- `crates/dna-chemistry/src/lib.rs`
- `docs/03-architecture-comparison.md`
- `docs/04-scientific-algorithms.md`
- `docs/08-roadmap.md`
- `docs/09-open-questions-and-risks.md`
- `docs/10-data-requirements.md`
- `docs/reproducibility.md`

**Verification Step:**

Run:
```bash
cargo test --workspace
```

Expected:
The complete workspace test suite and rustdoc compilation pass with the documentation matching the implemented M7 scope.

**Notes:**

Record deliberate differences from Geant4: explicit outcomes, typed invalid-domain/convergence errors, and open-(0,1) endpoint handling. Cite ADR 0003. Use TDD where doc tests apply and commit as `docs: record isolated two-body IRT validation`.
