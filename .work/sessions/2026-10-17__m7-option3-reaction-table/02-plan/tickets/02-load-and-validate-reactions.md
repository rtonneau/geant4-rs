# Ticket 02: load-and-validate-reactions

**Model:** opus
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] `ReactionParticipant` distinguishes the 15 mobile `Species` values from `H2O(B)`, `H3Op(B)`, `OHm(B)` and `NoneM`, and `Reaction` exposes its stable ID, ordered reactants/products, source class, kinetic intent/order, observed rate and source line.
- [ ] `ReactionTable::option3()` and `ReactionTable::from_toml(&str)` return an immutable table with `iter()`, stable-ID lookup and unordered mobile-pair lookup; no scheduler, radius, `k_diff` or `k_act` API is added.
- [ ] Loading rejects parse errors, noncanonical or unknown participants, missing/noncontiguous/duplicate IDs, non-finite or non-positive rates, wrong rate field/unit for the kinetic order, unordered duplicate mobile pairs and atom/charge imbalance.
- [ ] Conservation allows only an integral implicit-water residual with matching charge. Reaction 53 is accepted only when its ID, reactants and products exactly match the code-level Geant4 anomaly signature; a changed ID or signature fails.
- [ ] `option3_data_sha256()` includes the reaction TOML after species, dissociation channels and physicochemical parameters, and unit tests pin the new ordering.

**Files to Touch:**
- `crates/dna-chemistry/src/reactions.rs`
- `crates/dna-chemistry/src/lib.rs`
- `crates/dna-chemistry/src/species.rs`
- `crates/dna-chemistry/src/error.rs`
- `crates/dna-chemistry/Cargo.toml`

**Verification Step:**

Run:
```powershell
cargo test -p dna-chemistry reactions
```

Expected:
All reaction loader, lookup, malformed-input, conservation and fingerprint-order tests pass.

**Notes:**

Follow the private raw-record/public validated-type pattern used by `species.rs` and `channels.rs`. Keep source order for iteration and provenance. Canonicalize only the lookup key for mobile pairs. Numeric rates remain in their source units with explicit order; unit conversion and derived kinetic quantities belong to later M7 work.
