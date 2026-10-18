# Ticket 03: crosscheck-reaction-table

**Model:** sonnet
**Effort:** high

**Acceptance Criteria:**
- [ ] `scripts/hand_reactions.py` independently evaluates participant composition, charge, implicit-water completion, kinetic-order counts and the exact reaction-53 anomaly from `cpp_reactions.json`, producing a committed `hand_reactions.json` fixture.
- [ ] Rust integration tests compare all 72 loaded reactions against `cpp_reactions.json`, including IDs, ordered participants/products, rates, order/class/type intent and source lines.
- [ ] Rust integration tests compare loader conservation results with the independent hand fixture and prove that reaction 53 is the sole authorized imbalance.
- [ ] Both Python scripts support a check mode that fails when their committed fixtures are stale.

**Files to Touch:**
- `scripts/hand_reactions.py`
- `crates/dna-chemistry/tests/fixtures/hand_reactions.json`
- `crates/dna-chemistry/tests/cpp_crosscheck.rs`
- `crates/dna-chemistry/tests/common/mod.rs`

**Verification Step:**

Run:
```powershell
python scripts/hand_reactions.py --check crates/dna-chemistry/tests/fixtures/hand_reactions.json; if ($LASTEXITCODE -eq 0) { cargo test -p dna-chemistry --test cpp_crosscheck reaction }
```

Expected:
Both commands exit 0; the hand fixture is current and every reaction cross-check test passes.

**Notes:**

The hand evaluator must not import Rust code or reuse loader validation. It may consume the machine-extracted JSON, as `hand_physchem.py` does, but must implement composition and water-residual arithmetic independently.
