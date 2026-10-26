# Ticket 01: compile-network

**Model:** heavy
**Effort:** high

**Acceptance Criteria:**
- [ ] `ReactionTable::option3()` compiles exactly 52 pair and 20 unary/background entries to immutable descriptors, preserving stable IDs, source order, ordered products, and bulk participant identity; no entry is skipped.
- [ ] Compilation rejects unsupported intent/order combinations, unknown products, invalid diffusion/radius/rate values, arithmetic overflow, and inconsistent source signatures before any sampling.
- [ ] `ReactionTable::from_toml` accepts zero only for first- or pseudo-first-order custom entries and compiles an explicit disabled unary descriptor; `option3()` still rejects altered embedded literals and every invalid negative/non-finite rate.
- [ ] Existing `ReactionTable` and isolated two-body tests continue to pass.

**Files to Touch:**
- `crates/g4-chemistry/src/reactions.rs`
- `crates/g4-chemistry/src/reaction_parameters.rs`
- `crates/g4-chemistry/src/error.rs`
- `crates/g4-chemistry/src/lib.rs`
- `crates/g4-chemistry/src/reaction_network.rs` (new)
- `crates/g4-chemistry/tests/network_compilation.rs` (new)

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry --test network_compilation && cargo test -p g4-chemistry --test reaction_parameters
```

Expected:
Both test binaries report all tests passed and exit 0.

**Notes:**

Use an internal enum for pair versus unary execution; no plugin interface. Keep a distinction between an absent channel and a present disabled unary channel. `from_toml` remains a full 72-entry, Geant4-provenance table path, not an arbitrary reaction DSL. Source table: `G4EmDNAChemistry_option3::ConstructReactionTable`.
