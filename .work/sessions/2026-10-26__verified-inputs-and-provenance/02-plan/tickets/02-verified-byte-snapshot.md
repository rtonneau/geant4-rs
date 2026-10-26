# Ticket 02: verified-byte-snapshot

**Model:** heavy
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] A new immutable verified data snapshot loads every declared manifest file once, checks size and SHA-256 against the manifest, and parses model tables from those same captured bytes. Missing, unlisted, wrong-manifest and checksum-mismatched files fail before model construction.
- [ ] The production option2 `ProcessTable` is constructed through this snapshot and retains its verified identity. An explicitly named unchecked/synthetic construction path remains available for tests and cannot be mistaken for verified production input.
- [ ] Altered data bytes cannot retain the old verified identity; relocating an identical manifest and dataset preserves identity. Mutating a source file after snapshot construction does not change or relabel the frozen model.
- [ ] Current default model windows, cross sections and sampling inputs are unchanged.

**Files to Touch:**
- `crates/g4-data/src/datadir.rs`
- `crates/g4-data/src/lib.rs`
- `crates/g4-core/src/manifest.rs`
- `crates/g4-physics/src/table.rs`
- `crates/g4-physics/src/models/*.rs` (only constructors that load option2 data)
- `crates/g4-data/tests/real_tables.rs`
- `crates/g4-physics/tests/windows.rs`

**Verification Step:**

Run:
```bash
cargo test -p g4-core -p g4-data -p g4-physics --locked
```

Expected:
All tests pass, including altered-byte, relocation and frozen-snapshot regressions; no unchecked file read participates in production option2 construction.

**Notes:**

Prefer one `VerifiedDataSet::load(dir: &DataDir, manifest: &Manifest) -> Result<VerifiedDataSet, DataError>` seam in `g4-data`, with private checked bytes and read-only access to parsed tables. Add `VerifiedProcessTable::option2_electron(&VerifiedDataSet, ElasticRotation)` in `g4-physics`; it owns a `ProcessTable` and the verified identity. Keep the existing unchecked constructor temporarily so this ticket compiles; ticket 6 gives it an explicit unverified name and migrates callers. Preserve manifest-file order in identity records; do not include absolute roots.
