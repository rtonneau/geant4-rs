# Ticket 01: geometry-module-and-box

**Model:** heavy
**Effort:** high

**Acceptance Criteria:**
- [ ] `g4_core::geometry` exports `Volume`, `Inside {Inside, Surface, Outside}`, `GeometryError` (thiserror), and tolerance constants in nm derived from kCarTolerance = 1e-9 mm, with a source citation.
- [ ] `Volume::box_(centre, half_lengths)` rejects non-finite or non-positive values; serde deserialisation validates through the same path.
- [ ] `inside(p)` and `distance_to_out(p, v)` follow `G4Box.cc` semantics (cited lines). An Outside start gives None; a Surface start moving outward gives Some(0).
- [ ] Tests: analytic exits along the axes and diagonals, a centre offset, rays parallel to faces, surface starts going in and out, and proptest invariants (exit point is Surface; an Inside start gives d > 0).

**Files to Touch:**
- `crates/g4-core/src/geometry/mod.rs`
- `crates/g4-core/src/geometry/box_solid.rs`
- `crates/g4-core/src/lib.rs`
- `crates/g4-core/Cargo.toml`

**Verification Step:**

Run:
```bash
cargo test -p g4-core geometry
```

Expected:
All geometry tests pass (0 failed).

**Notes:**

Keep the shape fields private, and validate in the constructors. Put shared proptest generators (unit directions, points inside) in the test module so tickets 2 and 3 can reuse them. The direction must be a unit vector: document whether the function normalises it or requires a unit input, and reject a zero vector.
