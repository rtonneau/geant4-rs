# Ticket 03: sphere

**Status:** ✅ Done

## Local Test Result

Run on 2026-11-13. Output is kept in
`.scratch/tests/2026-11-13__geometry-primitives-g4-core/evidence/03-*.txt`.

- `cargo test -p g4-core geometry` (Verification Step): exit 0, `test result: ok. 107 passed; 0 failed; 0 ignored; 49 filtered out`. That is 61 from tickets 01-02, all still passing after the phi-exit move, plus 46 new in `geometry::sphere`. The integration-test binaries ran 0 tests after filtering, 0 failed. Evidence: `03-test-geometry.txt`.
- `cargo fmt --all -- --check`: exit 0, no output. Evidence: `03-fmt.txt`.
- `cargo clippy -p g4-core --all-targets -- -D warnings`: exit 0, no warnings. Evidence: `03-clippy.txt`.
- `cargo test -p g4-core` (whole crate, regression): exit 0. Unit tests 156 passed, `committed_data` 3 passed, `thread_independence` 2 passed, doctests 0. Evidence: `03-test-g4-core.txt`.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p g4-core --no-deps`: exit 0. Evidence: `03-doc.txt`.
- Stress runs in release, with `PROPTEST_MAX_GLOBAL_REJECTS` set to the case count:
  - Sphere at 1M cases (`PROPTEST_CASES=1000000 cargo test -p g4-core --release --lib geometry::sphere`), final code apart from two later doc-comment edits: exit 0, 46 passed. Evidence: `03-proptest-1M.txt`.
  - Sphere at 10M cases: exit 0, 46 passed. This ran before rustfmt and two doc-comment edits, which change no code. Evidence: `03-proptest-10M.txt`.
  - Tubs at 1M cases, ticket 02's run repeated after the refactor: exit 0, 32 passed. Evidence: `03-tubs-proptest-1M.txt`.
- Mutation check: each Geant4 form behind a deviation was restored one at a time, then the files were restored byte for byte (checked with `cmp`). Each time the matching deviation test failed: 8 of 8 detected. Evidence: `03-mutation-check.txt`.

`cargo test --workspace` was not run. Only `g4-core` changed, and no other crate uses `Volume` yet. Ticket 04 runs it.

After correction pass 1, on the final code (evidence `03-c1-*.txt`):
- `cargo test -p g4-core geometry` (Verification Step): exit 0, `test result: ok. 113 passed; 0 failed; 0 ignored; 49 filtered out`. That is 61 from tickets 01-02, 1 new in `geometry::tubs` and 51 in `geometry::sphere`. The integration-test binaries ran 0 tests after filtering, 0 failed. Evidence: `03-c1-test-geometry.txt`.
- `cargo fmt --all -- --check`: exit 0, no output. Evidence: `03-c1-fmt.txt`.
- `cargo clippy -p g4-core --all-targets -- -D warnings`: exit 0, no warnings. Evidence: `03-c1-clippy.txt`.
- `cargo test -p g4-core`: exit 0. Unit tests 162 passed, `committed_data` 3 passed, `thread_independence` 2 passed. Evidence: `03-c1-test-g4-core.txt`.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p g4-core --no-deps`: exit 0. Evidence: `03-c1-doc.txt`.
- Stress runs in release:
  - Sphere at 1M cases: exit 0, 51 passed (`03-c1-proptest-1M.txt`).
  - Sphere at 10M cases: exit 0, 51 passed, 126 s (`03-c1-proptest-10M.txt`).
  - Tubs at 1M cases: exit 0, 33 passed (`03-c1-tubs-proptest-1M.txt`). This ran before the last test-helper fix, which touches only `sphere.rs`.
- Mutation check on the corrected code: 11 of 11 detected (`03-c1-mutation-check.txt`). These are the original eight minus the mm-unit clamp variant, plus four new ones: deviation 7 removed, deviation 7 without its sign test, the parallel root without the nappe test, and the tubs axial guard removed.

## Review Notes

Base commit: dc8dc9dc192110b44bdb9a9ed7bf75484e0a2ab1.

Checked against the Geant4 11.4.1 source (read-only, `source/`):
- `geometry/solids/CSG/src/G4Sphere.cc`: constructor (81-111), `Inside` (255-359), `DistanceToOut(p, v)` (1714-2603).
- `include/G4Sphere.icc`: `InitializePhiTrigonometry` (121-136), `InitializeThetaTrigonometry` (138-152), `CheckThetaAngles` (155-190), `CheckSPhiAngle`/`CheckDPhiAngle`/`CheckPhiAngles` (192-246, the same as G4Tubs, so `PhiSegment::new` is reused).
- `include/G4Sphere.hh`: `fEpsilon = 2.e-11` (267).
- `G4Tubs.cc:1320-1452`, the tubs phi block, for deviation 1.
- `externals/clhep/include/CLHEP/Units/SystemOfUnits.h:139,155`: `s = second = 1e9` (needed for deviation 3).

Acceptance Criteria:
1. `Volume::sphere(centre, rmin, rmax, start_phi, delta_phi, start_theta, delta_theta)` validates as Geant4 does (cited in its rustdoc):
   - `rmin < 0` or `rmin >= rmax` gives `InvalidRadii`, and `rmax < 1.1*kRadTolerance` gives `TooSmall { field: "rmax" }` (`G4Sphere.cc:95`).
   - The phi range goes through `PhiSegment::new`, as for tubs.
   - The theta range follows `CheckThetaAngles`: if `start + delta >= pi`, the opening becomes `pi - start`, and the end angle is then exactly pi. A test sweeps 2000 starts to show this, and a Python check of 3M random starts and six rounding-tie cases found no exception. `fFullThetaSphere` is ported as `delta - start >= pi`.
   - The radial tolerances are `max(kRadTolerance, 2e-11 r)`. Tests check 1e-3 nm for small spheres and 0.02 nm for a 1 m sphere.
2. `inside_local` ports `Inside` branch for branch: the centre special case, the radial shells, the sphere's own phi test (`phi_class`, which differs from G4Tubs) and the theta test. `distance_to_out_local` ports `DistanceToOut(p, v)`:
   - the rmax far root and the rmin near root that overrides it;
   - the start and end cones: z = 0 plane branches, parallel-ray branches, on-surface early returns and the quadratic root selection with its nappe tests;
   - candidates combined in Geant4's order. Unlike G4Tubs, there is no final clamp below `kCarTolerance/2`.

   The six documented deviations are listed below.
3. Tests (46 in `sphere.rs`):
   - Analytic: rmax exits from the centre and off-centre; full-sphere chords from the surface (`2R cos(alpha)`) and both ways through a point (`2 sqrt(R^2 - b^2)`); rmin hole-wall exits (head-on, chord, oblique, a ray that misses the hole, a ray tangent to rmin); exits through start and end cones, both convex and concave; the theta = pi/2 planes of both hemispheres; phi-plane exits (opening up to pi, above pi, across phi = 0).
   - Rays through the poles: along the z axis of a full sphere, a polar cap, a section reaching theta = pi, a section without poles, and the z axis of a phi segment. Also a ray through the apex.
   - Surface starts: 21 boundary starts moving out (every surface type, inside and outside the shells, tangent and grazing) give 0. Starts moving inward cross the solid. Outside starts give None. A centre offset is tested. `Inside` is tested at 0.9 and 1.1 of each tolerance, including the angular phi and theta shells, the poles and the 1 m sphere.
   - Seven proptests: exit point on the surface (with the near-axis/near-apex Cartesian check, as for tubs); chord symmetry of a full sphere; boundary starts on rmax, rmin, a phi plane or a theta cone moving out give `Some(0)`; TOML round trip.
4. Deviations are documented in `sphere.rs` (module rustdoc), in `mod.rs` (summary) and in `docs/04-scientific-algorithms.md` D.5.1 (new section stub; ticket 04 adds the module overview). Each fixes a distance that is wrong beyond the tolerance, infinite or negative, and each has a test that failed in the mutation check:
   1. **Phi planes:** the G4Tubs phi block (`PhiSegment::exit_distance`) replaces `G4Sphere.cc:2149-2462`. G4Sphere compares the raw `atan2` direction angle with the segment range (2178-2181, 2213-2216, 2441-2442), so an on-axis start or a ray through the axis can be stopped wrongly. It also picks its cases from exact signs (2164, 2238, 2262), so a start on a plane, rounded just outside it, may not leave when the opening is above pi. Beyond the tolerance both blocks give the same result (checked case by case). Tests: `start_on_the_z_axis_of_a_phi_segment`, `ray_through_the_z_axis_with_a_direction_below_the_start_angle`, `start_just_outside_a_phi_plane_still_leaves`. The last one asserts the sign preconditions that send G4Sphere into its case 2.
   2. **Radial:** rmax is tested for every non-Outside start. In G4Sphere, `c < fRmaxTolerance*fRmax` (1773) and the negative `Rmin_minus` (1727, 1769) leave thin regions where the step is `kInfinity`. Test: `every_start_has_an_rmax_limit`.
   3. **Typo:** `G4Sphere.cc:1963` `std::fabs(s)`, where `s` is `CLHEP::s` = 1e9, is read as `sd`. The test case was found numerically: a start exactly on the cone has a first root of exactly 0. Test: `start_on_a_convex_start_cone_with_a_zero_first_root`, which asserts c = 0.
   4. **End-cone clamp:** the `|d2| < halfRmaxTolerance` clamp (2094) is dropped, because it compares an area with a length. In mm it misses the end cone of nm-sized spheres: for a 50 nm cap, 45.8 nm instead of 20. Ported literally in nm, it fails a 0.05 nm cap. The mutation check confirmed both.
   5. **Roots behind the start:** parallel-ray roots (1920, 2053) now pass the quadratic acceptance tests. Plane roots behind the start (1876, 2008) give 0. Geant4 gives -2.5e9 nm and -2e-3 nm in the two tests.
   6. **Apex start:** the direction's polar angle is compared with the theta range. At the apex Geant4 has t2 = 0: a convex cone always returns 0 and a concave one never limits the step. Test: `start_at_the_apex`.

   Constructor deviations: non-finite values are rejected, and so are `start_theta = pi` (Geant4 gives `fDTheta = 0`, an empty solid) and `delta_theta > pi` (Geant4 reduces it silently, like `delta_phi > 2 pi`).

Geant4 properties kept and documented (not deviations):
- Angular `Inside` shells versus Cartesian `DistanceToOut` tolerances: exits within about 10 nm of the axis or apex may classify as Outside, and Inside starts within the Cartesian tolerance can get 0.
- The z axis of a phi segment is Inside (`G4Sphere.cc:308`).
- The linearised inner rmax sliver (`tol^2/(8 rmax)`).
- Rays that touch rmin or pass through the apex stop there.
- Beyond 1e6 nm, Surface starts more than the Cartesian tolerance from their plane or cone are not recognised as leaving.

Design notes and changes outside Files to Touch:
- `phi_segment.rs` and `tubs.rs`: the tubs phi exit moved from a free function in `tubs.rs` into `PhiSegment::exit_distance`, so the sphere reuses it. One guard was added: an on-axis start moving along z is not limited (`G4Sphere.cc:2452-2455`). Correction (pass 1): this does change the tube, contrary to the claim first written here. A direction rounded just short of unit length, such as `Direction::new((0, 0, 49))`, has `t1 = 2.2e-16 > 0` and reaches the guard. It is now documented as a tubs deviation (see Correction pass 1). The tubs 1M stress run was repeated.
- Serialisation writes the requested angles (as tubs does). The getters return the normalised values (`delta_theta()` is after the clamp).
- New file `crates/g4-core/proptest-regressions/geometry/sphere.txt` (untracked): the seed of the 10M-run case below. It is replayed first on every run, and is now rejected by the generator's zone check. A fixed test replays the same case explicitly (`near_axis_start_on_a_phi_plane_is_inside_by_angle`). Proptest recommends committing the file, as ticket 02 did.
- No Geant4 generator was written or run. The Geant4 results quoted for the deviations are derived from the source, not executed.

Correction pass 1 (after the independent review, which confirmed deviations 1-6):
- **[P2] Starts within `h` of a cone: new deviation 7** (`sphere.rs` cone functions, module rustdoc, docs/04 D.5.1 item 7).
  - The problem: Geant4's on-cone test uses the horizontal distance `rho - z tan(theta)`, which is the true distance divided by `|cos theta|`, while its root acceptance requires `sd > h`. A start whose true distance lies in `(h |cos theta|, h]` and that moves out therefore ignores the cone. In the reviewer's example (theta in [pi/4, pi], rmax 10, r = 5, 4.5e-4 nm inside, along `-theta_hat`) the step ran 8.66 nm through the removed cone.
  - The fix: the port returns 0 when a root on the cone's own nappe lies within `h` of the start, behind or ahead, and the direction leaves the solid by the sign test of the on-cone early return.
  - Beyond the request: I added that sign test and also accepted roots behind the start. Without the sign test, a Surface start just outside the cone moving in, whose entry root is within `h`, would get 0. The mutation check showed that without it three tests fail, including the existing `surface_start_moving_inward_crosses_the_solid`. The same check covers the parallel-ray root.
  - Tests: `start_within_h_of_a_cone_moving_out_leaves_at_once` covers all four cone types at 0.9 h, asserts that the horizontal distance exceeds h, and its first case is the reviewer's example. `surface_start_just_outside_a_cone_moving_in_crosses_the_solid` covers the sign test. The new proptest `start_near_a_theta_cone_moving_out_stops_at_it` builds starts at 0.05-0.99 h inside a cone or plane, moving out with normal component `a`, and requires `d <= 1.5 delta/a`. sphere.rs:25-28 is corrected.
  - `EDGE_CLEARANCE` is kept at 0.01 nm and its rustdoc says why. Tightening it cannot reach the band without letting Inside starts within the tolerance in, where 0 is a correct result that breaks `d > 0`. A random start falls in a band about 1e-4 nm wide almost never. The constructed proptest covers the band instead.
- **[P3-1] Cost of deviation 1:** documented in `sphere.rs` (deviation 1) and docs/04 item 1. The G4Tubs block ignores a plane where G4Sphere returns 0 in two cases: a start rounded outside and moving out with `|comp| < pDist/h` (`G4Tubs.cc:1356`, 1402), and, beyond 1e6 nm, a Surface start more than `h` outside a plane (1345-1348). The "beyond 1e6 nm" property bullet now attributes the phi case to the G4Tubs block. It keeps the theta-cone case as G4Sphere's own: outside `h` the near root lies behind and is skipped.
- **[P3-2] The axial guard changes the tube:** documented as a tubs deviation in the `PhiSegment::exit_distance` rustdoc, the `tubs.rs` module rustdoc, the `mod.rs` deviation list and docs/04 D.5. New test `start_on_the_z_axis_moving_along_it_with_a_rounded_direction`: phi in [pi/2, pi], start (0, 0, 5), `Direction::new((0, 0, 49))`. It asserts `v.x = v.y = 0` and `t1 > 0`, then 15 nm (G4Tubs gives 0). The "unchanged" claim in the design notes above is corrected.
- **[P3-3] Tests:**
  - (a) The near-axis branch of `exit_point_is_on_the_surface` now first requires `within_tolerance`: inside the tolerant radial shells and at most `kCarTolerance/2` outside the phi and theta ranges (new helper `angular_excess`). An rmax/rmin exit outside the angular ranges no longer passes.
  - (b) The same proptest samples 9 points along `[p, p + d v]` and requires each to be not Outside, or (near the axis or apex) within that Cartesian tolerance.
  - (c) New test `parallel_ray_crossing_the_other_nappe_goes_on`: theta in [pi/4, pi], from (0, 0, -5) along (1, 0, 1)/sqrt 2. It asserts the parallel branch is taken, then 12.89 nm (Geant4 3.54). Deviation 5's docs now name early stops on the other nappe.
  - The first 1M run after (a) failed in my new helper, not in the port. `angular_excess` moved phi up into the segment but not down, so for the segment [-4.51, 0.27] an exit at phi = 2.28 (inside, as 2.28 - 2 pi) was flagged. I fixed the helper, pinned it with `angular_excess_wraps_phi_into_the_segment`, and removed the seed that proptest saved for it from `sphere.txt`, which again holds only the documented angular-shell case. Both stress runs then passed.
- **[P3-4] Docs:** docs/04 D.5.1 now gives the phi shell as `rho * 5e-10` and the theta shell as `r * 5e-10`, as does the `sphere.rs` tolerance paragraph. Item 5 mentions early stops on the other nappe and the 3.54 vs 12.89 example. `mod.rs` says the sphere scales its radial tolerance as `max(kRadTolerance, 2e-11 r)`, and says the angular tolerance also applies to theta.

## Blockers / Challenges

- G4Sphere's `DistanceToOut(p, v)` is older code than G4Tubs and has real defects: a unit typo, a unit-dependent clamp, unchecked parallel roots, raw `atan2` comparisons, exact-sign case selection, and no direction test at the apex. A literal port would reproduce wrong, infinite or negative distances. I fixed only those, each as a documented deviation with a test that the mutation check showed fails under the Geant4 form. Everything at tolerance level was kept as a documented Geant4 property. The reviewer should confirm this policy, especially deviation 1, which swaps in the G4Tubs phi block wholesale.
- The 1M run first failed on proptest's local-reject cap: a `prop_filter` in the theta-cone generator rejected too many draws. No geometry case failed. I rewrote the generator so it never rejects.
- The 10M run found one start, built on a phi plane 0.033 nm from the axis at a 1e6 nm offset, that classified Inside instead of Surface. The coordinates round to 1.2e-10 nm, so the point lies 1.8e-11 nm from the plane but 5.4e-10 rad from it in angle, just beyond `kAngTolerance/2`. This is the documented angular-shell property, not a porting bug. Generated boundary starts now stay outside the same 10 nm zone the exit check already uses, and a fixed test pins the case. The 10M rerun passed.
- Correction pass 1: in deviation 7 as literally requested ("root in [0, h] gives 0"), a Surface start just outside a cone and moving in gets 0. I added the early return's sign test, and the mutation check confirms it is needed. The first 1M run after the review fixes failed on a phi-wrapping bug in my new test helper `angular_excess`, not in the port. I fixed it and pinned it with a test, and the 1M and 10M reruns passed.

## Commits

- aec246d feat(g4-core): G4Sphere solid with documented G4Sphere deviations (ticket 03)

## Time Spent

1h 22m (ticket-start.js to ticket-complete.js)

## Parent acceptance notes

- Independent review round 1: 6 deviations judged justified; 1 P2 and 4 P3 findings, all resolved in correction pass 1.
- Re-review: deviation 7 (sign test, roots behind the start) judged correct and minimal. 3 residual P3 items were fixed inline by the parent:
  - the residual-case wording in sphere.rs and docs/04;
  - the 9-point check wording ("never more than kCarTolerance/2 outside");
  - the new test `surface_start_with_cone_root_behind_moving_out_leaves_at_once` (θs = π/2 − 1e-4, r = 1e3 nm, start 3e-10 rad outside, along −θ̂; expects 0).
- Not done: a mutation check of that new test against an ahead-only `sd` filter.
- Parent re-verification: `cargo test -p g4-core geometry` 114 passed, 0 failed; clippy -p g4-core -D warnings clean; fmt applied.
- Accepted: the criteria are met, and the deviations are documented in sphere.rs and docs/04 D.5.
