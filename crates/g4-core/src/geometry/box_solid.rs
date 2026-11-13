//! Rectangular box, a port of Geant4 11.4.1 `G4Box`
//! (`source/geometry/solids/CSG/src/G4Box.cc`).

use super::{
    GeometryError, HALF_CAR_TOLERANCE, Inside, MIN_HALF_LENGTH, check_at_least, check_finite,
};
use crate::vec3::Vec3;

/// Axis-aligned box of half-lengths `(dx, dy, dz)` about its centre, nm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxSolid {
    centre: Vec3,
    half_lengths: Vec3,
}

impl BoxSolid {
    /// Checks the parameters as `G4Box::G4Box` does (`G4Box.cc:65-73`): each half-length must
    /// be at least `2*kCarTolerance` ([`MIN_HALF_LENGTH`]). Non-finite values are also
    /// rejected, which Geant4 does not check.
    pub(super) fn new(centre: Vec3, half_lengths: Vec3) -> Result<Self, GeometryError> {
        check_finite("centre", centre)?;
        check_at_least("half_lengths.x", half_lengths.x, MIN_HALF_LENGTH)?;
        check_at_least("half_lengths.y", half_lengths.y, MIN_HALF_LENGTH)?;
        check_at_least("half_lengths.z", half_lengths.z, MIN_HALF_LENGTH)?;
        Ok(Self {
            centre,
            half_lengths,
        })
    }

    /// Centre, nm.
    pub fn centre(&self) -> Vec3 {
        self.centre
    }

    /// Half-lengths `(dx, dy, dz)`, nm.
    pub fn half_lengths(&self) -> Vec3 {
        self.half_lengths
    }

    /// `G4Box::Inside` (`G4Box.cc:238-246`) for a point `q` relative to the centre: the
    /// largest signed distance to the three face pairs, compared with `±delta`.
    pub(super) fn inside_local(&self, q: Vec3) -> Inside {
        let h = self.half_lengths;
        let dist = (q.x.abs() - h.x).max(q.y.abs() - h.y).max(q.z.abs() - h.z);
        if dist > HALF_CAR_TOLERANCE {
            Inside::Outside
        } else if dist > -HALF_CAR_TOLERANCE {
            Inside::Surface
        } else {
            Inside::Inside
        }
    }
    /// `G4Box::DistanceToOut(p)` (`G4Box.cc:433-456`): the isotropic safety distance, the
    /// smallest distance from `q` (relative to the centre) to a face, clamped at 0.
    pub(super) fn safety_local(&self, q: Vec3) -> f64 {
        let h = self.half_lengths;
        let dist = (h.x - q.x.abs()).min(h.y - q.y.abs()).min(h.z - q.z.abs());
        if dist > 0.0 { dist } else { 0.0 }
    }

    /// `G4Box::DistanceToOut(p, v)` without normal (`G4Box.cc:381-391`) for a point `q`
    /// relative to the centre, which is not outside.
    ///
    /// On a face (within `delta`) and moving outward through it, the distance is 0
    /// (`G4Box.cc:383-385`). Otherwise it is the smallest distance to the far face of each
    /// slab; a zero direction component never limits it (`G4Box.cc:386-389`). A start on a
    /// face moving parallel to it therefore travels along the face.
    pub(super) fn distance_to_out_local(&self, q: Vec3, v: Vec3) -> f64 {
        let h = self.half_lengths;
        let leaving = |p: f64, d: f64, h: f64| p.abs() - h >= -HALF_CAR_TOLERANCE && p * d > 0.0;
        if leaving(q.x, v.x, h.x) || leaving(q.y, v.y, h.y) || leaving(q.z, v.z, h.z) {
            return 0.0;
        }
        let slab = |p: f64, d: f64, h: f64| {
            if d == 0.0 {
                f64::MAX
            } else {
                (h.copysign(d) - p) / d
            }
        };
        slab(q.x, v.x, h.x)
            .min(slab(q.y, v.y, h.y))
            .min(slab(q.z, v.z, h.z))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{
        centre, check_exit_invariants, fractions, size, unit_direction,
    };
    use super::super::{CAR_TOLERANCE, Direction, GeometryError, Inside, MIN_HALF_LENGTH, Volume};
    use crate::vec3::Vec3;
    use approx::assert_relative_eq;
    use proptest::prelude::*;

    const DELTA: f64 = 0.5 * CAR_TOLERANCE;

    fn v3(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z)
    }

    fn dir(x: f64, y: f64, z: f64) -> Direction {
        Direction::new(v3(x, y, z)).unwrap()
    }

    /// Box of half-lengths (10, 20, 30) nm at the origin.
    fn unit_box() -> Volume {
        Volume::box_(Vec3::ZERO, v3(10.0, 20.0, 30.0)).unwrap()
    }

    // --- construction ---

    #[test]
    fn accessors_return_constructor_values() {
        let b = Volume::box_(v3(1.0, 2.0, 3.0), v3(4.0, 5.0, 6.0)).unwrap();
        assert_eq!(b.centre(), v3(1.0, 2.0, 3.0));
        #[allow(irrefutable_let_patterns)]
        let Volume::Box(solid) = b else {
            unreachable!("box strategy yields boxes")
        };
        assert_eq!(solid.half_lengths(), v3(4.0, 5.0, 6.0));
    }

    #[test]
    fn rejects_non_positive_half_lengths() {
        for h in [0.0, -1.0, -0.0] {
            let err = Volume::box_(Vec3::ZERO, v3(1.0, h, 1.0)).unwrap_err();
            assert!(
                matches!(
                    err,
                    GeometryError::TooSmall {
                        field: "half_lengths.y",
                        ..
                    }
                ),
                "{err}"
            );
        }
    }

    #[test]
    fn rejects_half_lengths_below_two_car_tolerances() {
        let just_below = MIN_HALF_LENGTH * (1.0 - 1e-12);
        assert!(Volume::box_(Vec3::ZERO, v3(1.0, 1.0, just_below)).is_err());
        assert!(Volume::box_(Vec3::ZERO, v3(1.0, 1.0, MIN_HALF_LENGTH)).is_ok());
    }

    #[test]
    fn rejects_non_finite_values() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let err = Volume::box_(Vec3::ZERO, v3(bad, 1.0, 1.0)).unwrap_err();
            assert!(matches!(
                err,
                GeometryError::NonFinite {
                    field: "half_lengths.x",
                    ..
                }
            ));
            let err = Volume::box_(v3(0.0, 0.0, bad), v3(1.0, 1.0, 1.0)).unwrap_err();
            assert!(matches!(
                err,
                GeometryError::NonFinite {
                    field: "centre",
                    ..
                }
            ));
        }
    }

    // --- serde ---

    #[test]
    fn toml_round_trip() {
        let b = Volume::box_(v3(1.0, -2.0, 3.5), v3(4.0, 5.0, 6.0)).unwrap();
        let text = toml::to_string(&b).unwrap();
        let back: Volume = toml::from_str(&text).unwrap();
        assert_eq!(back, b);
        assert!(text.contains("shape = \"box\""), "{text}");
    }

    #[test]
    fn deserialisation_validates_through_the_constructor() {
        let err = toml::from_str::<Volume>(
            "shape = \"box\"\ncentre = [0.0, 0.0, 0.0]\nhalf_lengths = [1.0, 0.0, 1.0]\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("half_lengths.y"), "{err}");
        // JSON can carry no NaN, but a negative half-length must fail the same way.
        let err = serde_json::from_str::<Volume>(
            r#"{"shape":"box","centre":[0,0,0],"half_lengths":[1,1,-1]}"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("half_lengths.z"), "{err}");
    }

    #[test]
    fn deserialisation_rejects_unknown_shapes_and_fields() {
        assert!(toml::from_str::<Volume>("shape = \"cone\"\ncentre = [0.0, 0.0, 0.0]\n").is_err());
        assert!(
            toml::from_str::<Volume>(
                "shape = \"box\"\ncentre = [0.0, 0.0, 0.0]\nhalf_lengths = [1.0, 1.0, 1.0]\n\
                 radius = 2.0\n"
            )
            .is_err()
        );
        assert!(toml::from_str::<Volume>("shape = \"box\"\ncentre = [0.0, 0.0, 0.0]\n").is_err());
    }

    // --- inside ---

    #[test]
    fn inside_classifies_with_half_tolerance_shell() {
        let b = unit_box();
        assert_eq!(b.inside(Vec3::ZERO), Inside::Inside);
        assert_eq!(b.inside(v3(10.0, 0.0, 0.0)), Inside::Surface);
        assert_eq!(b.inside(v3(0.0, -20.0, 0.0)), Inside::Surface);
        assert_eq!(b.inside(v3(10.0, 20.0, 30.0)), Inside::Surface); // corner
        assert_eq!(b.inside(v3(10.0 + 0.9 * DELTA, 0.0, 0.0)), Inside::Surface);
        assert_eq!(b.inside(v3(10.0 - 0.9 * DELTA, 0.0, 0.0)), Inside::Surface);
        assert_eq!(b.inside(v3(10.0 + 1.1 * DELTA, 0.0, 0.0)), Inside::Outside);
        assert_eq!(b.inside(v3(10.0 - 1.1 * DELTA, 0.0, 0.0)), Inside::Inside);
        assert_eq!(b.inside(v3(0.0, 0.0, -31.0)), Inside::Outside);
        assert_eq!(b.inside(v3(11.0, 25.0, 0.0)), Inside::Outside);
    }

    #[test]
    fn inside_follows_the_centre_offset() {
        let b = Volume::box_(v3(100.0, -50.0, 7.0), v3(1.0, 2.0, 3.0)).unwrap();
        assert_eq!(b.inside(v3(100.0, -50.0, 7.0)), Inside::Inside);
        assert_eq!(b.inside(v3(101.0, -50.0, 7.0)), Inside::Surface);
        assert_eq!(b.inside(v3(100.0, -50.0, 10.0)), Inside::Surface);
        assert_eq!(b.inside(Vec3::ZERO), Inside::Outside);
    }

    #[test]
    fn non_finite_points_are_outside() {
        let b = unit_box();
        assert_eq!(b.inside(v3(f64::NAN, 0.0, 0.0)), Inside::Outside);
        assert_eq!(b.inside(v3(0.0, f64::INFINITY, 0.0)), Inside::Outside);
        assert_eq!(
            b.distance_to_out(v3(0.0, 0.0, f64::NAN), dir(1.0, 0.0, 0.0)),
            None
        );
    }

    // --- distance_to_out ---

    #[test]
    fn axis_exits_from_the_centre() {
        let b = unit_box();
        let cases = [
            (dir(1.0, 0.0, 0.0), 10.0),
            (dir(-1.0, 0.0, 0.0), 10.0),
            (dir(0.0, 1.0, 0.0), 20.0),
            (dir(0.0, -1.0, 0.0), 20.0),
            (dir(0.0, 0.0, 1.0), 30.0),
            (dir(0.0, 0.0, -1.0), 30.0),
        ];
        for (v, expected) in cases {
            assert_eq!(b.distance_to_out(Vec3::ZERO, v), Some(expected), "{v:?}");
        }
    }

    #[test]
    fn axis_exits_from_an_off_centre_start() {
        let b = unit_box();
        let p = v3(4.0, -5.0, 6.0);
        assert_eq!(b.distance_to_out(p, dir(1.0, 0.0, 0.0)), Some(6.0));
        assert_eq!(b.distance_to_out(p, dir(-1.0, 0.0, 0.0)), Some(14.0));
        assert_eq!(b.distance_to_out(p, dir(0.0, 1.0, 0.0)), Some(25.0));
        assert_eq!(b.distance_to_out(p, dir(0.0, -1.0, 0.0)), Some(15.0));
        assert_eq!(b.distance_to_out(p, dir(0.0, 0.0, 1.0)), Some(24.0));
        assert_eq!(b.distance_to_out(p, dir(0.0, 0.0, -1.0)), Some(36.0));
    }

    #[test]
    fn diagonal_exits() {
        // Cube: the main diagonal reaches the corner, length h sqrt(3).
        let cube = Volume::box_(Vec3::ZERO, v3(5.0, 5.0, 5.0)).unwrap();
        for (sx, sy, sz) in [(1.0, 1.0, 1.0), (-1.0, 1.0, -1.0), (1.0, -1.0, -1.0)] {
            let d = cube.distance_to_out(Vec3::ZERO, dir(sx, sy, sz)).unwrap();
            assert_relative_eq!(d, 5.0 * 3f64.sqrt(), max_relative = 1e-14);
        }
        // Box (10, 20, 30): along (1,1,1)/sqrt(3) the x face is hit first, at t = 10 sqrt(3).
        let d = unit_box()
            .distance_to_out(Vec3::ZERO, dir(1.0, 1.0, 1.0))
            .unwrap();
        assert_relative_eq!(d, 10.0 * 3f64.sqrt(), max_relative = 1e-14);
        // In the xy plane at 45 degrees from (0, 15, 0): y face at t = 5 sqrt(2).
        let d = unit_box()
            .distance_to_out(v3(0.0, 15.0, 0.0), dir(1.0, 1.0, 0.0))
            .unwrap();
        assert_relative_eq!(d, 5.0 * 2f64.sqrt(), max_relative = 1e-14);
    }

    #[test]
    fn exits_with_a_centre_offset() {
        let c = v3(1.0e3, -2.0e3, 5.0e2);
        let b = Volume::box_(c, v3(10.0, 20.0, 30.0)).unwrap();
        assert_eq!(b.distance_to_out(c, dir(1.0, 0.0, 0.0)), Some(10.0));
        assert_eq!(
            b.distance_to_out(c + v3(0.0, 5.0, 0.0), dir(0.0, -1.0, 0.0)),
            Some(25.0)
        );
        let d = b.distance_to_out(c, dir(1.0, 1.0, 1.0)).unwrap();
        assert_relative_eq!(d, 10.0 * 3f64.sqrt(), max_relative = 1e-12);
        // The untranslated position of the same start is outside.
        assert_eq!(b.distance_to_out(Vec3::ZERO, dir(1.0, 0.0, 0.0)), None);
    }

    #[test]
    fn rays_parallel_to_faces() {
        let b = unit_box();
        // Inside, moving in the yz plane: x faces never limit the step.
        let d = b
            .distance_to_out(v3(9.0, 0.0, 0.0), dir(0.0, 3.0, 4.0))
            .unwrap();
        assert_relative_eq!(d, 20.0 / 0.6, max_relative = 1e-14); // y = 20 at t = 33.3
        // On the +x face, moving along it: travels the face to the z face.
        assert_eq!(
            b.distance_to_out(v3(10.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)),
            Some(30.0)
        );
        // On an edge (x = 10, y = 20), moving along the edge.
        assert_eq!(
            b.distance_to_out(v3(10.0, 20.0, -30.0), dir(0.0, 0.0, 1.0)),
            Some(60.0)
        );
    }

    #[test]
    fn surface_start_moving_outward_gives_zero() {
        let b = unit_box();
        assert_eq!(
            b.distance_to_out(v3(10.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)),
            Some(0.0)
        );
        assert_eq!(
            b.distance_to_out(v3(0.0, -20.0, 0.0), dir(0.0, -1.0, 0.0)),
            Some(0.0)
        );
        // Slightly outward through a face at a grazing angle.
        assert_eq!(
            b.distance_to_out(v3(0.0, 0.0, 30.0), dir(1.0, 0.0, 1e-6)),
            Some(0.0)
        );
        // Within the tolerance shell, on either side of the face.
        let outer = v3(10.0 + 0.9 * DELTA, 0.0, 0.0);
        let inner = v3(10.0 - 0.9 * DELTA, 0.0, 0.0);
        assert_eq!(b.distance_to_out(outer, dir(1.0, 0.0, 0.0)), Some(0.0));
        assert_eq!(b.distance_to_out(inner, dir(1.0, 0.0, 0.0)), Some(0.0));
        // At a corner, leaving through any of its faces.
        assert_eq!(
            b.distance_to_out(v3(10.0, 20.0, 30.0), dir(-1.0, -1.0, 1.0)),
            Some(0.0)
        );
    }

    #[test]
    fn surface_start_moving_inward_crosses_the_box() {
        let b = unit_box();
        assert_eq!(
            b.distance_to_out(v3(10.0, 0.0, 0.0), dir(-1.0, 0.0, 0.0)),
            Some(20.0)
        );
        assert_eq!(
            b.distance_to_out(v3(0.0, 0.0, -30.0), dir(0.0, 0.0, 1.0)),
            Some(60.0)
        );
        let d = b
            .distance_to_out(v3(-10.0, -20.0, -30.0), dir(1.0, 1.0, 1.0))
            .unwrap();
        assert_relative_eq!(d, 20.0 * 3f64.sqrt(), max_relative = 1e-14);
        // Within the tolerance shell outside the face, moving inward.
        let outer = v3(10.0 + 0.9 * DELTA, 0.0, 0.0);
        let d = b.distance_to_out(outer, dir(-1.0, 0.0, 0.0)).unwrap();
        assert_relative_eq!(d, 20.0 + 0.9 * DELTA, max_relative = 1e-14);
    }

    #[test]
    fn outside_start_gives_none() {
        let b = unit_box();
        assert_eq!(
            b.distance_to_out(v3(11.0, 0.0, 0.0), dir(-1.0, 0.0, 0.0)),
            None
        );
        assert_eq!(
            b.distance_to_out(v3(10.0 + 1.1 * DELTA, 0.0, 0.0), dir(-1.0, 0.0, 0.0)),
            None
        );
        assert_eq!(
            b.distance_to_out(v3(0.0, 100.0, 0.0), dir(0.0, 1.0, 0.0)),
            None
        );
    }

    // --- invariants ---

    fn box_strategy() -> impl Strategy<Value = Volume> {
        (centre(), size(), size(), size())
            .prop_map(|(c, x, y, z)| Volume::box_(c, v3(x, y, z)).unwrap())
    }

    proptest! {
        #[test]
        fn exit_point_is_on_the_surface(
            volume in box_strategy(),
            f in fractions(),
            v in unit_direction(),
        ) {
            #[allow(irrefutable_let_patterns)]
        let Volume::Box(solid) = &volume else {
            unreachable!("box strategy yields boxes")
        };
            let h = solid.half_lengths();
            let p = volume.centre() + v3(f.x * h.x, f.y * h.y, f.z * h.z);
            check_exit_invariants(&volume, p, v)?;
            // A chord of the box is never longer than its space diagonal.
            let d = volume.distance_to_out(p, v).unwrap();
            prop_assert!(d <= 2.0 * h.norm() * (1.0 + 1e-12));
        }

        #[test]
        fn surface_start_leaving_a_face_gives_zero(
            volume in box_strategy(),
            f in fractions(),
            v in unit_direction(),
            axis in 0usize..3,
        ) {
            #[allow(irrefutable_let_patterns)]
        let Volume::Box(solid) = &volume else {
            unreachable!("box strategy yields boxes")
        };
            let h = solid.half_lengths();
            // Move the start onto the face that `v` leaves through along `axis`.
            let mut local = [f.x * h.x, f.y * h.y, f.z * h.z];
            let dir = [v.as_vec3().x, v.as_vec3().y, v.as_vec3().z];
            prop_assume!(dir[axis] != 0.0);
            let half = [h.x, h.y, h.z];
            local[axis] = half[axis].copysign(dir[axis]);
            let p = volume.centre() + v3(local[0], local[1], local[2]);
            prop_assert_eq!(volume.inside(p), Inside::Surface);
            prop_assert_eq!(volume.distance_to_out(p, v), Some(0.0));
        }

        #[test]
        fn serde_round_trip_preserves_the_box(volume in box_strategy()) {
            let text = toml::to_string(&volume).unwrap();
            let back: Volume = toml::from_str(&text).unwrap();
            prop_assert_eq!(back, volume);
        }
    }

    // --- safety (`G4Box::DistanceToOut(p)`) ---

    #[test]
    fn safety_is_the_distance_to_the_nearest_face() {
        let b = Volume::box_(v3(1.0, 1.0, 1.0), v3(10.0, 20.0, 30.0)).unwrap();
        assert_relative_eq!(b.safety(v3(4.0, 2.0, 3.0)).unwrap(), 7.0);
        assert_relative_eq!(b.safety(v3(1.0, 1.0, 1.0)).unwrap(), 10.0);
        assert_eq!(b.safety(v3(11.0, 1.0, 1.0)), Some(0.0));
        assert_eq!(b.safety(v3(12.0, 1.0, 1.0)), None);
    }

    proptest! {
        #[test]
        fn safety_never_exceeds_the_exit_distance(
            volume in box_strategy(),
            f in fractions(),
            v in unit_direction(),
        ) {
            #[allow(irrefutable_let_patterns)]
            let Volume::Box(solid) = &volume else {
                unreachable!("box strategy yields boxes")
            };
            let h = solid.half_lengths();
            let p = volume.centre() + v3(f.x * h.x, f.y * h.y, f.z * h.z);
            let safety = volume.safety(p).unwrap();
            let d = volume.distance_to_out(p, v).unwrap();
            prop_assert!(safety >= 0.0);
            prop_assert!(safety <= d + CAR_TOLERANCE, "safety {} > exit {}", safety, d);
        }
    }
}
