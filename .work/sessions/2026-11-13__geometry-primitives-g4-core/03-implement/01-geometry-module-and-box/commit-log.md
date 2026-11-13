# Ticket 01: geometry-module-and-box

**Status:** ✅ Done

## Local Test Result

Verification Step and the extra gates were run on 2026-11-13. Output is kept in
`.scratch/tests/2026-11-13__geometry-primitives-g4-core/evidence/01-*.txt`.

- `cargo test -p g4-core geometry`: exit 0, `test result: ok. 24 passed; 0 failed; 0 ignored; 49 filtered out` (the integration-test binaries ran 0 tests after filtering, 0 failed). Evidence: `01-test-geometry.txt`.
- `cargo fmt --all -- --check`: exit 0, no output. Evidence: `01-fmt.txt`.
- `cargo clippy -p g4-core --all-targets -- -D warnings`: exit 0, no warnings. Evidence: `01-clippy.txt`.
- `cargo test -p g4-core` (whole crate, as a regression check): exit 0. Unit tests 73 passed, `committed_data` 3 passed, `thread_independence` 2 passed, doctests 0. Evidence: `01-test-g4-core.txt`.

The workspace-wide `cargo test --workspace` was not run, because this ticket only touches `g4-core`. Ticket 04 runs it as its own Verification Step.

## Review Notes

Base commit: 101f91c093cea2412d5e20ce95ae87f556d77007.

Checked against the Geant4 11.4.1 source (read-only, `source/`):
- `global/management/src/G4GeometryTolerance.cc:46-51` sets `fCarTolerance = 1E-9*mm`, which `geometry/management/src/G4VSolid.cc:62` copies into `kCarTolerance`. `SetSurfaceTolerance` (lines 92-107) can rescale it from the world extent; the Rust code uses only the default, and the module doc says so.
- `geometry/solids/CSG/src/G4Box.cc`: `delta = 0.5*kCarTolerance` (64), the constructor's `< 2*kCarTolerance` rejection (65-73), `Inside` (238-246) and `DistanceToOut(p, v)` without normal (381-391).

Acceptance Criteria:
1. `g4_core::geometry` exports `Volume`, `Inside {Inside, Surface, Outside}`, `GeometryError` (thiserror; variants `NonFinite`, `TooSmall`, `InvalidDirection`), and the constants `CAR_TOLERANCE = 1e-3` nm, `HALF_CAR_TOLERANCE` and `MIN_HALF_LENGTH = 2*kCarTolerance`, with source citations. A test checks `CAR_TOLERANCE` against `Length::from_mm(1e-9)`. It also exports `BoxSolid` (private fields, read-only accessors) and `Direction`.
2. `Volume::box_(centre, half_lengths)` rejects non-finite centre or half-length components. It also rejects any half-length below `2*kCarTolerance` = 2e-3 nm, which covers zero, -0.0 and negative values, as `G4Box.cc:65-73` does. Serde uses `#[serde(try_from = "RawVolume", into = "RawVolume")]`, so deserialisation calls the same constructor. `RawVolume` is internally tagged by `shape` with `deny_unknown_fields`, and tickets 02 and 03 add their variants to it. Tests cover TOML and JSON rejection, unknown shapes, unknown fields, missing fields, a TOML round trip, and a TOML round-trip proptest.
3. `inside` and `distance_to_out` port the G4Box lines listed above. `Volume` applies the centre translation once for all shapes, maps NaN points to Outside, and returns `None` for an Outside start before calling the shape code. A Surface start moving outward returns `Some(0.0)`, from G4Box's `px*vx > 0` test. A Surface start moving inward or parallel to the face gets the distance to the far boundary.
4. Tests cover analytic axis exits from the centre and from an off-centre start, diagonals in a cube and in a non-cube box, a centre offset, rays parallel to faces (including along a face and along an edge), surface starts going out (face, inside or outside the tolerance shell, grazing, corner) and going in, and outside starts. Three proptests check that the exit point is Surface and an Inside start gives 0 < d <= the space diagonal; that a start on the face a ray leaves through is Surface and gives `Some(0)`; and the serde round trip.

Design decisions for later tickets:
- The direction is the `Direction` newtype. `Direction::new` normalises any non-zero finite vector and rejects zero or non-finite vectors with `GeometryError::InvalidDirection`, so the ray methods can rely on `|v| = 1` without checking it on every call.
- Shared proptest helpers live in `geometry::test_support` (`#[cfg(test)]`, `pub(super)`): `size`, `centre`, `fractions`, `unit_direction` (isotropic, plus axis-aligned directions one time in four) and `check_exit_invariants`. Sizes are bounded to [1, 1e6) nm, offsets to (-1e6, 1e6) nm and inside-point fractions to (-0.99, 0.99). With these bounds rounding stays around 1e-9 nm, well below `delta` = 5e-4 nm, and inside points stay at least 0.01 nm from the boundary.
- Shapes implement `inside_local` and `distance_to_out_local` in their own frame. `Volume` dispatches with `match`, so a new shape needs a new variant, a new `RawVolume` variant and two match arms.

Deviations from Geant4, documented in the module rustdoc:
- An Outside start returns `None`, where Geant4 does not check and returns an unspecified distance.
- NaN points are classified as Outside.
- Non-finite parameters are rejected, which Geant4 does not check.

`crates/g4-core/Cargo.toml` was not changed: `serde`, `thiserror` and `toml` were already dependencies, and `proptest`, `approx` and `serde_json` were already dev-dependencies.

## Blockers / Challenges

None.

## Commits

- 747f847 feat(g4-core): geometry module with Volume, Inside and box solid (ticket 01)

## Time Spent

8m (ticket-start.js to ticket-complete.js)
