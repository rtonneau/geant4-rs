# Ticket 06: document-oracles-and-open-gate

**Model:** medium
**Model (Jev):** sonnet-5.5 (confidence 0.22)
**Effort:** high

**Acceptance Criteria:**
- [ ] Document the source-faithful SBS algorithm, V10 Gillespie/LSODA system, pinned `rsoda` provenance, Geant4 mode and schema, statistical criteria, and the equality-boundary distinction.
- [ ] Update M7 status, risks, data requirements, and reproducibility records to mark local oracle tests complete and the external Geant4 two-body comparison `awaiting data`.
- [ ] Documentation points to the open run request and later `m7-sbs-close-gate`; it does not claim that the external comparison passed.
- [ ] Formatting, Clippy, and the full workspace test suite pass.

**Files to Touch:**
- `docs/03-architecture-comparison.md`
- `docs/04-scientific-algorithms.md`
- `docs/06-dependencies.md`
- `docs/07-validation-strategy.md`
- `docs/08-roadmap.md`
- `docs/09-open-questions-and-risks.md`
- `docs/10-data-requirements.md`
- `docs/reproducibility.md`

**Verification Step:**

Run:
```powershell
cargo fmt --all -- --check; if ($LASTEXITCODE -eq 0) { cargo clippy --workspace --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test --workspace }
```

Expected:
All three commands exit 0 with no formatting differences, Clippy warnings, or test failures.

**Notes:**

Keep the distinction between completed local oracle validation and the unrun external Geant4 gate explicit throughout.
