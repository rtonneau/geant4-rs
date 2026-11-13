# Ticket 02: tubs

**Model:** heavy
**Effort:** high

**Acceptance Criteria:**
- [ ] `Volume::tubs(centre, rmin, rmax, half_z, start_phi, delta_phi)` validates 0 <= rmin < rmax, half_z > 0, 0 < delta_phi <= 2 pi (Geant4 normalisation of start_phi, cited).
- [ ] `inside` and `distance_to_out` follow `G4Tubs.cc` (cited lines), including the rmin surface, phi planes and the full-phi case.
- [ ] Tests: analytic radial and axial exits, exits through the rmin hole wall, phi-plane exits, rays parallel to the z axis or the end caps, surface starts, and proptest invariants.

**Files to Touch:**
- `crates/g4-core/src/geometry/tubs.rs`
- `crates/g4-core/src/geometry/mod.rs`

**Verification Step:**

Run:
```bash
cargo test -p g4-core geometry
```

Expected:
All geometry tests pass (0 failed).

**Notes:**

Read `G4Tubs.cc` Inside and DistanceToOut(p, v, ...) before writing anything. Record kRadTolerance and kAngTolerance with their source.
