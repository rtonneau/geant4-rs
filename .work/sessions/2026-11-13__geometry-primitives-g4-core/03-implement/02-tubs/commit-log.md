# Ticket 02: tubs

**Status:** ✅ Done

## Local Test Result

Run on 2026-11-13. Output is kept in
`.scratch/tests/2026-11-13__geometry-primitives-g4-core/evidence/02-*.txt`.

- `cargo test -p g4-core geometry` (Verification Step): exit 0, `test result: ok. 59 passed; 0 failed; 0 ignored; 49 filtered out` (24 from ticket 01, 30 in `geometry::tubs`, 4 in `geometry::phi_segment`, 1 new tolerance test in `geometry::tests`). The integration-test binaries ran 0 tests after filtering, 0 failed. Evidence: `02-test-geometry.txt`.
- `cargo fmt --all -- --check`: exit 0, no output. Evidence: `02-fmt.txt`.
- `cargo clippy -p g4-core --all-targets -- -D warnings`: exit 0, no warnings. Evidence: `02-clippy.txt`.
- `cargo test -p g4-core` (whole crate, regression): exit 0. Unit tests 108 passed, `committed_data` 3 passed, `thread_independence` 2 passed, doctests 0. Evidence: `02-test-g4-core.txt`.
- Stress run, `PROPTEST_CASES=1000000 PROPTEST_MAX_GLOBAL_REJECTS=100000 cargo test -p g4-core --release --lib geometry::tubs`: exit 0, 30 passed. Evidence: `02-proptest-1M.txt`.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p g4-core --no-deps`: exit 0.

`cargo test --workspace` was not run, because this ticket only touches `g4-core` and no other crate uses `Volume` yet. Ticket 04 runs it.

## Review Notes

Base commit: 5af2ccf956b882257878fcd339085b0dc2a7e99c.

Checked against the Geant4 11.4.1 source (read-only, `source/`):
- `global/management/src/G4GeometryTolerance.cc:46-51`: `fAngTolerance = 1E-9*rad` (49) and `fRadTolerance = 1E-9*mm` (50). `G4Tubs.cc:82-87` reads them and halves them. New constants: `RAD_TOLERANCE = 1e-3` nm, `HALF_RAD_TOLERANCE`, `ANG_TOLERANCE = 1e-9` rad and `HALF_ANG_TOLERANCE`, each with its source cited. A test checks `RAD_TOLERANCE` against `Length::from_mm(1e-9)`. `SetSurfaceTolerance` rescales `kRadTolerance` but not `kAngTolerance` (`G4GeometryTolerance.cc:96`). Only the defaults are used, as for `kCarTolerance`.
- `geometry/solids/CSG/src/G4Tubs.cc`: constructor checks (89-106), `Inside` (327-486), `DistanceToOut(p, v)` (1127-1543). `include/G4Tubs.icc`: `InitializeTrigonometry` (94-109), `CheckSPhiAngle` (111-127), `CheckDPhiAngle` (129-154), `CheckPhiAngles` (156-161).

Acceptance Criteria:
1. `Volume::tubs(centre, rmin, rmax, half_z, start_phi, delta_phi)` rejects non-finite values (`NonFinite`), `half_z <= 0` (new `NotPositive`), and `rmin < 0` or `rmin >= rmax` (new `InvalidRadii`), as `G4Tubs.cc:89-102` does. It also rejects `delta_phi` outside `(0, 2 pi]` (new `AngleOutOfRange`). The start angle is normalised as `G4Tubs.icc:111-161` does. A `delta_phi >= 2 pi - kAngTolerance/2` is a full circle with `start = 0`. Otherwise a non-zero start is reduced with `fmod` to `[0, 2 pi)`, then shifted by `-2 pi` if the segment would end beyond `2 pi`. Tests cover each branch, including -2 pi, 2 pi, -5 pi and 3 pi.
2. `TubsSolid::inside_local` follows `G4Tubs::Inside` branch for branch: the inner z/r limits with the full-phi, z-axis and phi checks, then the two "generous" branches. Those two are identical in Geant4 and are merged into `tolerant_shell`. The phi classification ports `G4Tubs.cc:359-392`, including the `fSPhi < 0` branch and the wrap for a segment ending at 2 pi. `distance_to_out_local` follows `DistanceToOut(p, v)`. It keeps the z-plane early returns, the `t1 > 0` guard around the radial and phi blocks, and the three radial branches with Geant4's root formulas: rmax exit when moving away from the axis, rmin hit or rmax far root when moving towards it, and the zero return on the tolerant surface. It also keeps the phi block (on-axis start, `fDPhi <= pi` and `> pi` plane tests, half-plane check against the centre direction, on-axis crossings tested against `vphi`), the ordering, and the final clamp below `kCarTolerance/2`. `kInfinity` (9e99) is `f64::INFINITY`; the `10*(fDz+fRMax)` test behaves the same.
3. Analytic tests cover radial exits (axis and off-axis), axial and oblique exits, exits through the rmin hole wall (head-on, chord, oblique), a ray tangent to rmin that crosses on to rmax, and phi-plane exits for openings below pi, above pi and across phi = 0, including an exit through the z axis where the planes meet. They also cover a start on the z axis of a segment, rays parallel to z (inside, along rmax, rmin and a phi plane) and rays parallel to the caps (including along a cap). Surface starts leaving through each boundary give 0, including tangent, grazing and inside or outside the tolerance shell. Surface starts moving inward cross the solid, and Outside starts give None. Inside is tested at every boundary at 0.9 and 1.1 of each tolerance, including the angular shell. Six proptests check: Inside start gives `d > 0`, the exit point is Surface and the chord is at most the axial-section diagonal; starts on rmax, rmin, a cap or a phi plane moving out give `Some(0)`; and the TOML round trip. These generators build their directions from the boundary normal instead of rejecting cases.

Design notes:
- New file `crates/g4-core/src/geometry/phi_segment.rs`, outside the ticket's Files to Touch. It holds `PhiSegment`, the shape-agnostic part of the phi range: the `CheckPhiAngles` normalisation (the same in `G4Sphere.icc:193-246`) and the start, end and centre sines and cosines. Ticket 03 can reuse it. The `Inside` and `DistanceToOut` phi branches of `G4Sphere.cc` differ from those of `G4Tubs` (checked at `G4Sphere.cc:255-340` and `2147-2290`), so they stay in `tubs.rs`. One difference for ticket 03: `G4Sphere::CheckDPhiAngle` does not reset `fSPhi` for a full circle. That has no effect, because full-phi solids skip all phi tests.
- Serialisation writes the requested `start_phi`/`delta_phi`, not the normalised values. The Geant4 normalisation is not idempotent in floating point (for example, a negative start goes through `2 pi - |s|`), so writing the normalised values would break the exact round trip. `TubsSolid::start_phi()`/`delta_phi()` return the normalised values, as the Geant4 getters do.
- `RawVolume::Tubs` requires all six fields (no defaults), so a full cylinder needs `delta_phi = 6.283185307179586`.
- `box_solid.rs` was not changed. Its `#[allow(irrefutable_let_patterns)]` attributes are now unnecessary but harmless; clippy does not flag them.

Deviations from Geant4, documented in rustdoc:
- `delta_phi > 2 pi` is rejected, where Geant4 silently resets it to 2 pi.
- Non-finite parameters are rejected, which Geant4 does not check.

Known Geant4 property, documented and not a deviation: `Inside` uses an angular phi shell (half-width `r * 5e-10` nm at radius `r`). Within a few nm of the z axis, the rounding of a computed exit point (about 1e-9 nm at coordinates near 1e6 nm) is larger than this shell. A phi-plane exit there can then classify as Outside. The 1M-case stress run found one such case: an exit 1.5e-3 nm from the axis, 4e-11 nm from the end plane in Cartesian terms but 2.6e-8 rad from it in angle. The exit invariant therefore checks the Cartesian distance to a phi plane (or a cap) inside a 10 nm zone around the axis of a segment, and the strict Surface check everywhere else. Proptest saved that case's seed in `crates/g4-core/proptest-regressions/geometry/tubs.txt` (new, untracked). Every test run replays it first, so the near-axis branch stays exercised; proptest recommends committing the file.

Correction pass 1 (after an independent review that found three P3 test gaps; only `tubs.rs` changed):
1. The relaxed near-axis check in `exit_point_is_on_the_surface` measured the distance to the full start and end planes. A new test helper, `half_plane_distances`, now counts a plane only when the point is on its half-plane side: `q.x cos + q.y sin >= -kCarTolerance` for that plane's angle. The check also requires `rho <= rmax + kRadTolerance/2` and `|q.z| <= half_z + kCarTolerance/2`.
2. New fixed test `near_axis_phi_exit_is_on_the_half_plane_but_outside_in_angle` replays the regression seed explicitly. It asserts that the exit is less than 2e-3 nm from the axis, within `kCarTolerance/2` of the end half-plane, more than `kAngTolerance/2` from the end angle, and that `inside` gives Outside. The `proptest-regressions` file is kept.
3. New fixed test `ray_crossing_the_z_axis_inside_the_phi_range`: `three_quarter()` (rmax = 10 nm, phi in [0, 3 pi/2]), from (-5, -5, 0) along (1, 1, 0)/sqrt 2. The ray crosses the axis at t = 5 sqrt 2 with its direction at phi = pi/4, inside the range, so the end-plane crossing there is not an exit (`G4Tubs.cc:1407-1418`). It then reaches rmax at a further 10 nm: expected and measured 5 sqrt 2 + 10, to 1e-14 relative.

Re-verification: `cargo test -p g4-core geometry` exit 0, 61 passed, 0 failed (`02-c1-test-geometry.txt`). `cargo fmt --all -- --check` exit 0 (`02-c1-fmt.txt`). `cargo clippy -p g4-core --all-targets -- -D warnings` exit 0, no warnings (`02-c1-clippy.txt`). 1M-case tubs proptest run exit 0, 32 passed (`02-c1-proptest-1M.txt`).

## Blockers / Challenges

- At 100k proptest cases, the boundary-start tests that used `prop_assume` hit proptest's 1024 global-reject cap. At the default 256 cases the rmin test accepted only about 27% of cases, close to that cap. I rewrote those generators to build the direction from the boundary normal, so they no longer reject cases.
- The 1M-case stress run found the near-axis angular-tolerance case described in Review Notes. The Cartesian check confirmed the exit point is on the plane, so the cause is Geant4's tolerance model and not the port. I narrowed the invariant to that zone and documented why.

## Commits

- 510c6a0 feat(g4-core): G4Tubs solid with phi-segment helper (ticket 02)

## Time Spent

25m (ticket-start.js to ticket-complete.js)
