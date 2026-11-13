# Ticket 01: volume-safety

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] `Volume::safety(p)` returns Geant4's DistanceToOut(p) for box, tube segment and sphere section (0 for Surface/Outside starts, as Geant4).
- [ ] Analytic tests per solid and a proptest: `safety(p) <= distance_to_out(p, v)` for any direction.
- [ ] Rustdoc cites G4Box.cc, G4Tubs.cc, G4Sphere.cc lines; docs/04 D.5 drops "no safety distances".

**Files to Touch:**
- `crates/g4-core/src/geometry/mod.rs`
- `crates/g4-core/src/geometry/box_solid.rs`
- `crates/g4-core/src/geometry/tubs.rs`
- `crates/g4-core/src/geometry/sphere.rs`
- `docs/04-scientific-algorithms.md`

**Verification Step:**

Run:
```bash
cargo test -p g4-core geometry
```

Expected:
All geometry tests pass, including the new safety tests.

**Notes:**

Read the 11.4.1 source for DistanceToOut(const G4ThreeVector&) of each solid before porting.
