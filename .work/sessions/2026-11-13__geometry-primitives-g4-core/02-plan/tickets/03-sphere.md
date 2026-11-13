# Ticket 03: sphere

**Model:** heavy
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] `Volume::sphere(centre, rmin, rmax, start_phi, delta_phi, start_theta, delta_theta)` validates ranges as Geant4 does (cited).
- [ ] `inside` and `distance_to_out` follow `G4Sphere.cc` (cited lines) for the rmin/rmax shells, the phi planes and the theta cones (including theta = pi/2 planes and the poles).
- [ ] Tests: analytic shell exits, the full-sphere chord length, phi- and theta-cone exits, surface starts, rays through the poles, and proptest invariants.
- [ ] Any intentional deviation from G4Sphere is documented.

**Files to Touch:**
- `crates/g4-core/src/geometry/sphere.rs`
- `crates/g4-core/src/geometry/mod.rs`
- `docs/04-scientific-algorithms.md`

**Verification Step:**

Run:
```bash
cargo test -p g4-core geometry
```

Expected:
All geometry tests pass (0 failed).

**Notes:**

This is the hardest port. Reuse the phi-segment helper from tubs where the Geant4 logic is the same.
