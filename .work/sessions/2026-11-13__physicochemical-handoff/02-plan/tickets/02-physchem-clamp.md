# Ticket 02: physchem-clamp

**Model:** heavy
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] With a volume, e_aq placement follows G4DNAOneStepThermalizationModel: if safety <= |d|, place at x + unit(d) * 0.8 * safety.
- [ ] With a volume, each dissociation product's displacement d = d_mother + d_product is shortened to min(0.8 * safety, |d|) (G4DNAMolecularDissociation::CheckNextStep); a product still outside is kept and counted.
- [ ] Per-species counters: shortened placements and species outside after placement.
- [ ] Tests: clamped point lies inside and on the ray; D2 (only counted products outside); D4 unbounded output and RNG stream unchanged.

**Files to Touch:**
- `crates/g4-chemistry/src/displacer.rs`
- `crates/g4-chemistry/src/engine.rs`
- `crates/g4-chemistry/tests/`

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry
```

Expected:
All tests pass, including the clamp and D4 tests.

**Notes:**

Verify against Geant4 11.4.1 .cc/.hpp (not only the 11.4.3 audit) and cite lines. The clamp draws no random number.
