# Ticket 01: special-functions

**Model:** heavy
**Model (Jev):** opus-5.5 (confidence 0.39)
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] `erfc(x)` and `erfc_inv(p)` are finite and monotone on their documented domains, handle exact domain boundaries explicitly, and reject invalid/non-finite inputs through `ChemistryError`.
- [ ] Committed high-precision fixtures cover central values, tails, symmetry, and round trips with absolute or relative error at most `1e-14` wherever the reference value is representable.
- [ ] The implementation cites its algorithm and Geant4 11.4.1 `G4ErrorFunction` provenance; it does not use the rejected Numerical Recipes rational approximation or add a dependency.

**Files to Touch:**
- `crates/dna-chemistry/src/special.rs`
- `crates/dna-chemistry/src/error.rs`
- `crates/dna-chemistry/src/lib.rs`
- `crates/dna-chemistry/tests/special_functions.rs`
- `crates/dna-chemistry/tests/fixtures/special_functions.json`

**Verification Step:**

Run:
```bash
cargo test -p dna-chemistry --test special_functions
```

Expected:
The special-function fixture, boundary, monotonicity, and round-trip tests all pass.

**Notes:**

Implement only the special functions needed by the IRT formulas. Keep the API crate-private unless a later public interface requires it. Use TDD and commit as `feat: add high-accuracy IRT special functions`.
