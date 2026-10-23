# Ticket 03: reaction-parameters

**Model:** heavy
**Model (Jev):** opus-5.5 (confidence 0.27)
**Effort:** high

**Acceptance Criteria:**
- [ ] Validated runtime parameters are derived from `SpeciesTable` and `ReactionTable` for mobile two-body reactions using the exact option3 rules for A+B and A+A.
- [ ] TDC derivation covers effective radius and the charged Onsager/effective-distance data; PDC derivation covers physical contact radius, `k_diff`, `k_act`, and stored contact probability with `Rs = 0.29 nm`.
- [ ] PDC rejects `k_obs >= k_diff`; all zero-total-diffusivity, non-finite, non-positive-radius, unsupported-order, bulk-participant, and arithmetic-domain failures are typed errors.
- [ ] Source-backed tests reproduce the exact parameters for the three pre-registered reactions within stated floating-point tolerances.

**Files to Touch:**
- `crates/dna-chemistry/src/reaction_parameters.rs`
- `crates/dna-chemistry/src/error.rs`
- `crates/dna-chemistry/src/lib.rs`
- `crates/dna-chemistry/tests/reaction_parameters.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-chemistry --test reaction_parameters
```

Expected:
All source-backed A+A, neutral/charged TDC, neutral PDC, and invalid-domain tests pass.

**Notes:**

Keep observed rates distinct from derived activation-controlled rates. Cite the exact Geant4 11.4.1 construction paths and preserve reaction-table source data. Use TDD and commit as `feat: derive option3 two-body reaction parameters`.
