# Ticket 04: chemistry-semantic-identity

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] `PhysChemEngine::new` validates the species, channels and parameters it stores and exposes a stable SHA-256 identity over their canonical semantic values.
- [ ] A changed custom channel, species property, parameter or recombination policy changes identity. Equivalent inputs in a different map insertion order retain identity.
- [ ] Embedded option3 retains its source-file SHA-256 as an additional field; custom engines are never labeled with that default hash alone.
- [ ] Invalid custom chemistry returns `ChemistryError`; valid option3 conversion and RNG use remain unchanged.

**Files to Touch:**
- `crates/g4-chemistry/src/engine.rs`
- `crates/g4-chemistry/src/params.rs`
- `crates/g4-chemistry/src/reactions.rs`
- `crates/g4-chemistry/src/species_types.rs`
- `crates/g4-chemistry/src/lib.rs`
- `crates/g4-chemistry/src/engine.rs` (unit tests)

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry --locked
```

Expected:
All chemistry tests pass; mutation and insertion-order tests establish the identity contract without altering the default scientific outputs.

**Notes:**

Keep chemistry hashing and validation entirely in `g4-chemistry`. Specify canonical field order and numeric encoding in rustdoc so the identity is reproducible. The hash describes the constructed engine's actual values; `with_recombination_enabled` must update it.
