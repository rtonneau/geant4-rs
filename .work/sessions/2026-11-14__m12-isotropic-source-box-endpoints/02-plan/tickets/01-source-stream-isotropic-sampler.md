# Ticket 01: source-stream-isotropic-sampler

**Model:** medium
**Effort:** medium

**Acceptance Criteria:**
- [ ] `Stage::Source = 3` exists in `g4_core::rng`, stream number `(history_id << 2) | 3`; golden-value test pins its first words.
- [ ] Isotropic sampler returns a unit vector from exactly two uniforms; formula and Geant4 source (G4RandomDirection.hh, version 11.4.3) cited in rustdoc.
- [ ] Pre-registered tests (alpha written in the test before running): KS on cos theta vs U(-1,1) and phi vs U(0,2pi), N = 100000, reject at p < 0.001; unit norm to 1e-12.

**Files to Touch:**
- `crates/g4-core/src/rng.rs`
- `crates/g4-core/src/` (sampler module or vec3 helper)

**Verification Step:**

Run:
```bash
cargo test -p g4-core
```

Expected:
All tests pass, including the new isotropy and stream tests.

**Notes:**

Check the Geant4 v11.4.3 implementation of G4RandomDirection in geant4SourcePath before writing the rustdoc; if it uses a rejection method, document that the distribution, not the sequence, is reproduced.
