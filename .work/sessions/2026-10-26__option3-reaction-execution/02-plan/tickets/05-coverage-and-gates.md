# Ticket 05: coverage-and-gates

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] A reviewed matrix maps IDs 1–72 to pair/unary regime, constructor source line, and corresponding `G4DNAIRT` execution branch; tests assert descriptor coverage and ordered product identities for all IDs.
- [ ] Every channel's atom and charge balance is tested with explicit bulk exchange and implicit water; only the exact existing reaction-53 signature is excepted.
- [ ] Documentation states energy/chemistry applicability, source paths, rate units, literal-placement limitation, numerical errors, validation criteria/results, and R03's boundary before R04/R05.
- [ ] Formatting, Clippy with warnings denied, and the full workspace test suite pass.

**Files to Touch:**
- `crates/g4-chemistry/tests/reaction_coverage.rs` (new)
- `docs/04-scientific-algorithms.md`
- `docs/10-data-requirements.md`
- `docs/09-open-questions-and-risks.md`
- `docs/reviews/proposed-roadmap.md`
- `docs/validation/option3-reaction-coverage.md` (new)

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected:
All three commands exit 0, with every test suite passing.

**Notes:**

Do not change Geant4 C++ reference files, existing reference CSV/provenance, or generator modes. The coverage matrix cites exact Geant4 11.4.1 paths and per-reaction constructor lines from the embedded table. Preserve existing pair-kernel evidence; R04 will assess independent pair acceptance.
