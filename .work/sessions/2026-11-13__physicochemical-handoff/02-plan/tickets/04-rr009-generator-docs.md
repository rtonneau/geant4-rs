# Ticket 04: rr009-generator-docs

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] New `g4rs_ref finite_volume` mode: shape, size, energy, seed, histories and output path on the command line; outputs the docs/07 observables (E_escaped, primary escape, out-of-volume species counts at 1 ps and end of chemistry, OUTSIDE_OF_MOTHER_VOLUME count); existing modes untouched.
- [ ] Compile-checked (or the reason it could not be is stated).
- [ ] RR-009 logged in RUN_REQUESTS.md; README and BUILD_NOTES updated; docs/04 D.6/D.7 and docs/08 M12 progress updated.

**Files to Touch:**
- `validation/geant4/g4rs_ref/g4rs_ref.cc`
- `validation/geant4/README.md`
- `validation/geant4/BUILD_NOTES.md`
- `validation/geant4/RUN_REQUESTS.md`
- `docs/08-roadmap.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected:
All pass; generator compile log recorded.

**Notes:**

Agents never run the generator.
