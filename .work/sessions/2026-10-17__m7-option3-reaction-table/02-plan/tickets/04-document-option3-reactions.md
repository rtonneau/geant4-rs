# Ticket 04: document-option3-reactions

**Model:** haiku
**Effort:** medium

**Acceptance Criteria:**
- [ ] `data/chemistry/README.md` documents the fourth embedded file, its schema, concatenation order and Geant4 v11.4.1 provenance without inventing a source commit.
- [ ] `docs/02-cpp-reference-inventory.md` records the audited table, rate/type semantics, line ranges, bulk participants and known source discrepancies in the B.7.6-B.7.7 context.
- [ ] `docs/04-scientific-algorithms.md` clarifies the D.2 boundary and updates D.3 with the validated reaction representation and deferred derived kinetics; `docs/10-data-requirements.md`, `docs/08-roadmap.md` and `docs/reproducibility.md` describe the completed table and four-file fingerprint.
- [ ] Formatting, linting and the full workspace test suite pass.

**Files to Touch:**
- `data/chemistry/README.md`
- `docs/02-cpp-reference-inventory.md`
- `docs/04-scientific-algorithms.md`
- `docs/08-roadmap.md`
- `docs/10-data-requirements.md`
- `docs/reproducibility.md`

**Verification Step:**

Run:
```powershell
cargo fmt --all -- --check; if ($LASTEXITCODE -eq 0) { cargo clippy --workspace --all-targets -- -D warnings }; if ($LASTEXITCODE -eq 0) { cargo test --workspace }
```

Expected:
All three commands exit 0 with no formatting differences, Clippy warnings or test failures.

**Notes:**

Document executable literals as authoritative and list the comment mismatches as source anomalies, not corrections. State explicitly that reaction execution, radii, `k_diff`, `k_act`, probability, diffusion and scheduling remain for later M7 milestones.
