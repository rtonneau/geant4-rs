# Ticket 05: provenance-envelope

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] Provenance records implementation revision and dirty state, crate version, `rustc -V`, target/platform, SHA-256 of `Cargo.lock`, Geant4 reference version, verified manifest and loaded-data identities, master seed, units and structured numerical/model policy identifiers.
- [ ] Build-time provenance explicitly represents unavailable Git or lockfile information instead of presenting a guessed value as verified.
- [ ] Paths do not enter semantic identity; the serialized envelope distinguishes verified and unverified inputs and documents same-platform reproducibility limits.
- [ ] Unit tests cover stable serialization keys and changed lockfile/policy identity; no new runtime dependency is added.

**Files to Touch:**
- `crates/g4-core/build.rs`
- `crates/g4-core/src/provenance.rs`
- `crates/g4-core/src/sha256.rs`
- `crates/g4-core/Cargo.toml`
- `docs/reproducibility.md`

**Verification Step:**

Run:
```bash
cargo test -p g4-core --locked
```

Expected:
All provenance tests pass; serialized metadata has the required fields and no absolute data-root path.

**Notes:**

Use the existing SHA-256 implementation and build script where possible. Preserve honest dirty-state semantics and capture the target actually built; do not infer these from the caller's manifest. Document any unavailable source-state evidence explicitly.
