//! Cylindrical tube segment, a port of Geant4 11.4.1 `G4Tubs`
//! (`source/geometry/solids/CSG/src/G4Tubs.cc`, `include/G4Tubs.icc`).
//!
//! The solid is bounded by the cylinders `rho = rmin` and `rho = rmax` about the z axis, the
//! planes `z = ±half_z` and, unless the phi range is a full circle, the two half-planes
//! `phi = start` and `phi = start + delta`. With `rmin = 0` it is a solid cylinder (segment).
//!
//! Geant4 uses three tolerances here: [`HALF_CAR_TOLERANCE`] for the z planes and for the
//! distances to the phi planes in `DistanceToOut`, [`HALF_RAD_TOLERANCE`] for the cylinders,
//! and [`HALF_ANG_TOLERANCE`] (an angle) for the phi planes in `Inside`. For `Inside` the
//! phi shell at radius `r` is therefore `r * 5e-10` nm thick on each side of the plane:
//! thinner than the Cartesian shell within 1e6 nm of the axis, wider beyond. This is the
//! Geant4 behaviour and is kept. A consequence, also present in Geant4: within a few nm of
//! the z axis the rounding of a computed exit point (about 1e-9 nm for coordinates near
//! 1e6 nm) exceeds the angular shell, so a point that `distance_to_out` puts on a phi plane
//! there may classify as Outside.
//!
//! One deviation from `G4Tubs::DistanceToOut(p, v)`: a start on the z axis of a phi segment
//! moving along the axis, with a direction rounded just short of unit length (`t1 > 0`), is
//! not limited by the phi planes. `G4Tubs` gives 0 there unless phi = 0 lies in the segment
//! (`G4Tubs.cc:1437-1450`). See [`PhiSegment::exit_distance`].

use super::phi_segment::PhiSegment;
use super::{
    GeometryError, HALF_ANG_TOLERANCE, HALF_CAR_TOLERANCE, HALF_RAD_TOLERANCE, Inside,
    RAD_TOLERANCE, check_finite,
};
use crate::vec3::Vec3;
use std::f64::consts::TAU;

/// Tube segment about the z axis through its centre: radii `rmin < rmax`, half-length
/// `half_z` along z (nm) and a phi range (rad).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TubsSolid {
    centre: Vec3,
    rmin: f64,
    rmax: f64,
    half_z: f64,
    phi: PhiSegment,
}

impl TubsSolid {
    /// Checks the parameters as `G4Tubs::G4Tubs` does (`G4Tubs.cc:89-106`): `half_z > 0`,
    /// `0 <= rmin < rmax`, and the phi range of [`PhiSegment::new`]. Non-finite values are
    /// also rejected, which Geant4 does not check.
    pub(super) fn new(
        centre: Vec3,
        rmin: f64,
        rmax: f64,
        half_z: f64,
        start_phi: f64,
        delta_phi: f64,
    ) -> Result<Self, GeometryError> {
        check_finite("centre", centre)?;
        for (field, value) in [("rmin", rmin), ("rmax", rmax), ("half_z", half_z)] {
            if !value.is_finite() {
                return Err(GeometryError::NonFinite { field, value });
            }
        }
        if half_z <= 0.0 {
            return Err(GeometryError::NotPositive {
                field: "half_z",
                value: half_z,
            });
        }
        if rmin >= rmax || rmin < 0.0 {
            return Err(GeometryError::InvalidRadii { rmin, rmax });
        }
        let phi = PhiSegment::new(start_phi, delta_phi)?;
        Ok(Self {
            centre,
            rmin,
            rmax,
            half_z,
            phi,
        })
    }

    /// Centre, nm.
    pub fn centre(&self) -> Vec3 {
        self.centre
    }

    /// Inner radius, nm (0 for a solid cylinder).
    pub fn rmin(&self) -> f64 {
        self.rmin
    }

    /// Outer radius, nm.
    pub fn rmax(&self) -> f64 {
        self.rmax
    }

    /// Half-length along z, nm.
    pub fn half_z(&self) -> f64 {
        self.half_z
    }

    /// Start angle after Geant4 normalisation (`G4Tubs::GetStartPhiAngle`), rad: in
    /// `[0, 2 pi)`, in `(-2 pi, 0)` for a segment crossing phi = 0, and 0 for a full circle.
    pub fn start_phi(&self) -> f64 {
        self.phi.start()
    }

    /// Opening angle after Geant4 normalisation (`G4Tubs::GetDeltaPhiAngle`), rad: exactly
    /// `2 pi` for a full circle.
    pub fn delta_phi(&self) -> f64 {
        self.phi.delta()
    }

    /// The `start_phi` given to the constructor, rad.
    pub(super) fn requested_start_phi(&self) -> f64 {
        self.phi.requested_start()
    }

    /// The `delta_phi` given to the constructor, rad.
    pub(super) fn requested_delta_phi(&self) -> f64 {
        self.phi.requested_delta()
    }

    /// `G4Tubs::Inside` (`G4Tubs.cc:327-486`) for a point `q` relative to the centre.
    ///
    /// Well inside the z planes (`G4Tubs.cc:332-396`), a point within the narrow radial
    /// limits is classified by phi, except on the z axis of a solid segment, which is
    /// Surface. Otherwise, and between the tolerant z limits (`G4Tubs.cc:441-484`), only the
    /// tolerant shell can give Surface.
    pub(super) fn inside_local(&self, q: Vec3) -> Inside {
        let r2 = q.x * q.x + q.y * q.y;
        let abs_z = q.z.abs();
        if abs_z <= self.half_z - HALF_CAR_TOLERANCE {
            let tol_rmin = if self.rmin != 0.0 {
                self.rmin + HALF_RAD_TOLERANCE
            } else {
                0.0
            };
            let tol_rmax = self.rmax - HALF_RAD_TOLERANCE;
            if r2 >= tol_rmin * tol_rmin && r2 <= tol_rmax * tol_rmax {
                if self.phi.is_full() {
                    Inside::Inside
                } else if tol_rmin == 0.0
                    && q.x.abs() <= HALF_CAR_TOLERANCE
                    && q.y.abs() <= HALF_CAR_TOLERANCE
                {
                    Inside::Surface
                } else {
                    phi_class(&self.phi, q.x, q.y)
                }
            } else {
                self.tolerant_shell(q, r2)
            }
        } else if abs_z <= self.half_z + HALF_CAR_TOLERANCE {
            self.tolerant_shell(q, r2)
        } else {
            Inside::Outside
        }
    }

    /// The "generous boundaries" branches of `G4Tubs::Inside` (`G4Tubs.cc:397-439` and
    /// `441-484`, identical): Surface inside the tolerant radial and phi limits, else Outside.
    fn tolerant_shell(&self, q: Vec3, r2: f64) -> Inside {
        let tol_rmin = (self.rmin - HALF_RAD_TOLERANCE).max(0.0);
        let tol_rmax = self.rmax + HALF_RAD_TOLERANCE;
        if r2 < tol_rmin * tol_rmin || r2 > tol_rmax * tol_rmax {
            return Inside::Outside;
        }
        // Continuous in phi, or on the z axis.
        if self.phi.is_full() || r2 <= HALF_RAD_TOLERANCE * HALF_RAD_TOLERANCE {
            return Inside::Surface;
        }
        match phi_class(&self.phi, q.x, q.y) {
            Inside::Outside => Inside::Outside,
            Inside::Inside | Inside::Surface => Inside::Surface,
        }
    }

    /// `G4Tubs::DistanceToOut(p, v)` without normal (`G4Tubs.cc:1127-1543`) for a point `p`
    /// relative to the centre, which is not outside.
    ///
    /// The candidates are the z plane ahead (`G4Tubs.cc:1143-1184`), the cylinder exit
    /// (`1197-1316`) and the phi exit (`1320-1458`); the smallest wins (`1453-1463`), and a
    /// distance below `kCarTolerance/2` is returned as 0 (`1540`). A start within tolerance
    /// of a boundary and moving out through it returns 0. A ray parallel to the z axis
    /// (`v.z = ±1`) is limited by the z planes only, and a ray parallel to the end caps
    /// (`v.z = 0`) by the cylinders and phi planes only.
    pub(super) fn distance_to_out_local(&self, p: Vec3, v: Vec3) -> f64 {
        let (rmin, rmax, dz) = (self.rmin, self.rmax, self.half_z);

        // Z planes, G4Tubs.cc:1143-1184.
        let mut snxt = if v.z > 0.0 {
            let pdist = dz - p.z;
            if pdist <= HALF_CAR_TOLERANCE {
                return 0.0;
            }
            pdist / v.z
        } else if v.z < 0.0 {
            let pdist = dz + p.z;
            if pdist <= HALF_CAR_TOLERANCE {
                return 0.0;
            }
            -pdist / v.z
        } else {
            f64::INFINITY
        };

        // Cylinders, G4Tubs.cc:1197-1316: (t1 t^2 + 2 t2 t + t3 = R^2) in the xy plane.
        let t1 = 1.0 - v.z * v.z;
        let t2 = p.x * v.x + p.y * v.y;
        let t3 = p.x * p.x + p.y * p.y;
        // G4Tubs.cc:1204: the radial and phi intersections are only computed off the z
        // direction; parallel to z, the z plane is the exit.
        if t1 > 0.0 {
            // Radius squared where the ray meets the z plane ahead (G4Tubs.cc:1201-1202).
            let roi2 = if snxt > 10.0 * (dz + rmax) {
                2.0 * rmax * rmax
            } else {
                snxt * snxt * t1 + 2.0 * snxt * t2 + t3
            };
            let reaches_rmax = roi2 > rmax * (rmax + RAD_TOLERANCE);
            let b = t2 / t1;
            // Far root -b + sqrt(b^2 - c) of the cylinder of radius r, if it exists.
            let far_root = |r: f64| {
                let d2 = b * b - (t3 - r * r) / t1;
                (d2 >= 0.0).then(|| -b + d2.sqrt())
            };
            let mut srd = f64::INFINITY;
            if t2 >= 0.0 && reaches_rmax {
                // Moving away from the axis: leaving via rmax (G4Tubs.cc:1208-1239).
                let delta_r = t3 - rmax * rmax;
                if delta_r < -RAD_TOLERANCE * rmax {
                    let c = delta_r / t1;
                    let d2 = b * b - c;
                    srd = if d2 >= 0.0 { c / (-b - d2.sqrt()) } else { 0.0 };
                } else {
                    // On the tolerant rmax surface, heading out or along it.
                    return 0.0;
                }
            } else if t2 < 0.0 {
                // Moving towards the axis: possible rmin intersection (G4Tubs.cc:1240-1316).
                let ro_min2 = t3 - t2 * t2 / t1; // smallest rho^2 along the ray
                if rmin != 0.0 && ro_min2 < rmin * (rmin - RAD_TOLERANCE) {
                    let delta_r = t3 - rmin * rmin;
                    let c = delta_r / t1;
                    let d2 = b * b - c;
                    if d2 >= 0.0 {
                        if delta_r > RAD_TOLERANCE * rmin {
                            srd = c / (-b + d2.sqrt()); // near root of rmin
                        } else {
                            // On the tolerant rmin surface, heading into the hole.
                            return 0.0;
                        }
                    } else {
                        match far_root(rmax) {
                            Some(d) => srd = d,
                            None => return 0.0,
                        }
                    }
                } else if reaches_rmax {
                    // No rmin intersection: leaving via rmax (G4Tubs.cc:1292-1315).
                    match far_root(rmax) {
                        Some(d) => srd = d,
                        None => return 0.0,
                    }
                }
            }

            if !self.phi.is_full() {
                let sphi = self.phi.exit_distance(p, v);
                if sphi < snxt {
                    snxt = sphi;
                }
            }
            if srd < snxt {
                snxt = srd;
            }
        }
        if snxt < HALF_CAR_TOLERANCE { 0.0 } else { snxt }
    }
}

/// Phi classification of `(x, y)` in `G4Tubs::Inside` (`G4Tubs.cc:359-392`), with the
/// angular tolerance: Inside between the inner tolerant phi limits, Surface between the
/// outer ones, else Outside. The "outer limits only" tests of `G4Tubs.cc:405-435` and
/// `449-482` are the same function with Inside read as Surface. `atan2` gives phi in
/// `[-pi, pi]`, moved to `[-tol, 2 pi - tol)`; a segment ending at 2 pi also accepts phi
/// close to 0.
fn phi_class(seg: &PhiSegment, x: f64, y: f64) -> Inside {
    let h = HALF_ANG_TOLERANCE;
    let (start, end) = (seg.start(), seg.end());
    let mut p_phi = y.atan2(x);
    if p_phi < -h {
        p_phi += TAU;
    }
    if start >= 0.0 {
        if p_phi.abs() < h && (end - TAU).abs() < h {
            p_phi += TAU;
        }
        if p_phi >= start + h && p_phi <= end - h {
            Inside::Inside
        } else if p_phi >= start - h && p_phi <= end + h {
            Inside::Surface
        } else {
            Inside::Outside
        }
    } else if p_phi <= start + TAU - h && p_phi >= end + h {
        // The segment crosses phi = 0: the excluded arc is (end, start + 2 pi).
        Inside::Outside
    } else if p_phi <= start + TAU + h && p_phi >= end - h {
        Inside::Surface
    } else {
        Inside::Inside
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{
        FRACTION, centre, check_exit_invariants, size, unit_direction,
    };
    use super::super::{
        CAR_TOLERANCE, Direction, GeometryError, HALF_ANG_TOLERANCE, HALF_CAR_TOLERANCE,
        HALF_RAD_TOLERANCE, Inside, Volume,
    };
    use crate::vec3::Vec3;
    use approx::assert_relative_eq;
    use proptest::prelude::*;
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

    const HRT: f64 = HALF_RAD_TOLERANCE;
    const HCT: f64 = HALF_CAR_TOLERANCE;

    fn v3(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z)
    }

    fn dir(x: f64, y: f64, z: f64) -> Direction {
        Direction::new(v3(x, y, z)).unwrap()
    }

    fn tubs(rmin: f64, rmax: f64, half_z: f64, start: f64, delta: f64) -> Volume {
        Volume::tubs(Vec3::ZERO, rmin, rmax, half_z, start, delta).unwrap()
    }

    /// Solid cylinder, radius 10 nm, half-length 20 nm.
    fn cylinder() -> Volume {
        tubs(0.0, 10.0, 20.0, 0.0, TAU)
    }

    /// Hollow cylinder, radii 4 and 10 nm, half-length 20 nm.
    fn hollow() -> Volume {
        tubs(4.0, 10.0, 20.0, 0.0, TAU)
    }

    /// Solid quarter cylinder, phi in [0, pi/2].
    fn quarter() -> Volume {
        tubs(0.0, 10.0, 20.0, 0.0, FRAC_PI_2)
    }

    /// Solid three-quarter cylinder, phi in [0, 3 pi/2] (opening above pi).
    fn three_quarter() -> Volume {
        tubs(0.0, 10.0, 20.0, 0.0, 1.5 * PI)
    }

    fn solid(volume: &Volume) -> &super::TubsSolid {
        match volume {
            Volume::Tubs(t) => t,
            other => unreachable!("not a tubs: {other:?}"),
        }
    }

    fn d(volume: &Volume, p: Vec3, v: Direction) -> f64 {
        volume.distance_to_out(p, v).unwrap()
    }

    // --- construction ---

    #[test]
    fn accessors_return_constructor_values() {
        let t = Volume::tubs(v3(1.0, 2.0, 3.0), 4.0, 5.0, 6.0, 0.5, 1.0).unwrap();
        assert_eq!(t.centre(), v3(1.0, 2.0, 3.0));
        let s = solid(&t);
        assert_eq!(
            (s.rmin(), s.rmax(), s.half_z(), s.start_phi(), s.delta_phi()),
            (4.0, 5.0, 6.0, 0.5, 1.0)
        );
    }

    #[test]
    fn start_phi_is_normalised_and_ignored_for_a_full_circle() {
        let t = tubs(0.0, 1.0, 1.0, -FRAC_PI_4, FRAC_PI_2);
        assert_relative_eq!(solid(&t).start_phi(), -FRAC_PI_4, epsilon = 1e-15);
        let t = tubs(0.0, 1.0, 1.0, 2.5 * PI, 1.0);
        assert_relative_eq!(solid(&t).start_phi(), 0.5 * PI, epsilon = 1e-14);
        let t = tubs(0.0, 1.0, 1.0, 1.0, TAU);
        assert_eq!((solid(&t).start_phi(), solid(&t).delta_phi()), (0.0, TAU));
    }

    #[test]
    fn rejects_invalid_sizes_and_radii() {
        for half_z in [0.0, -0.0, -1.0] {
            let err = Volume::tubs(Vec3::ZERO, 0.0, 1.0, half_z, 0.0, TAU).unwrap_err();
            assert!(
                matches!(
                    err,
                    GeometryError::NotPositive {
                        field: "half_z",
                        ..
                    }
                ),
                "{err}"
            );
        }
        for (rmin, rmax) in [(2.0, 1.0), (1.0, 1.0), (-0.5, 1.0), (0.0, 0.0), (0.0, -1.0)] {
            let err = Volume::tubs(Vec3::ZERO, rmin, rmax, 1.0, 0.0, TAU).unwrap_err();
            assert_eq!(err, GeometryError::InvalidRadii { rmin, rmax });
        }
        // A tiny but positive size is accepted, as in Geant4.
        assert!(Volume::tubs(Vec3::ZERO, 0.0, 1e-6, 1e-6, 0.0, TAU).is_ok());
    }

    #[test]
    fn rejects_out_of_range_angles() {
        for delta in [0.0, -1.0, TAU + 1e-9] {
            let err = Volume::tubs(Vec3::ZERO, 0.0, 1.0, 1.0, 0.0, delta).unwrap_err();
            assert!(
                matches!(
                    err,
                    GeometryError::AngleOutOfRange {
                        field: "delta_phi",
                        ..
                    }
                ),
                "{err}"
            );
        }
    }

    #[test]
    fn rejects_non_finite_values() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let args = [
                ("rmin", [bad, 1.0, 1.0, 0.0, 1.0]),
                ("rmax", [0.0, bad, 1.0, 0.0, 1.0]),
                ("half_z", [0.0, 1.0, bad, 0.0, 1.0]),
                ("start_phi", [0.0, 1.0, 1.0, bad, 1.0]),
                ("delta_phi", [0.0, 1.0, 1.0, 0.0, bad]),
            ];
            for (name, [a, b, c, e, f]) in args {
                let err = Volume::tubs(Vec3::ZERO, a, b, c, e, f).unwrap_err();
                assert!(
                    matches!(err, GeometryError::NonFinite { field, .. } if field == name),
                    "{name}: {err}"
                );
            }
            let err = Volume::tubs(v3(bad, 0.0, 0.0), 0.0, 1.0, 1.0, 0.0, 1.0).unwrap_err();
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
    fn toml_round_trip_keeps_the_requested_angles() {
        let t = Volume::tubs(v3(1.0, -2.0, 3.5), 1.5, 4.0, 6.0, -0.5, 1.0).unwrap();
        let text = toml::to_string(&t).unwrap();
        assert!(text.contains("shape = \"tubs\""), "{text}");
        assert!(text.contains("start_phi = -0.5"), "{text}");
        let back: Volume = toml::from_str(&text).unwrap();
        assert_eq!(back, t);
    }

    #[test]
    fn deserialisation_validates_through_the_constructor() {
        let base = "shape = \"tubs\"\ncentre = [0.0, 0.0, 0.0]\nhalf_z = 1.0\n\
                    start_phi = 0.0\ndelta_phi = 1.0\n";
        let err = toml::from_str::<Volume>(&format!("{base}rmin = 2.0\nrmax = 1.0\n")).unwrap_err();
        assert!(err.to_string().contains("rmin"), "{err}");
        let err = serde_json::from_str::<Volume>(
            r#"{"shape":"tubs","centre":[0,0,0],"rmin":0,"rmax":1,"half_z":1,
                "start_phi":0,"delta_phi":7}"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("delta_phi"), "{err}");
        // A missing field or an unknown one is an error.
        assert!(toml::from_str::<Volume>(&format!("{base}rmax = 1.0\n")).is_err());
        assert!(
            toml::from_str::<Volume>(&format!("{base}rmin = 0.0\nrmax = 1.0\ndz = 1.0\n")).is_err()
        );
        assert!(toml::from_str::<Volume>(&format!("{base}rmin = 0.0\nrmax = 1.0\n")).is_ok());
    }

    // --- inside ---

    #[test]
    fn inside_full_cylinder() {
        let t = cylinder();
        assert_eq!(t.inside(Vec3::ZERO), Inside::Inside);
        assert_eq!(t.inside(v3(10.0, 0.0, 0.0)), Inside::Surface);
        assert_eq!(t.inside(v3(0.0, -10.0, 5.0)), Inside::Surface);
        assert_eq!(t.inside(v3(10.0 + 0.9 * HRT, 0.0, 0.0)), Inside::Surface);
        assert_eq!(t.inside(v3(10.0 - 0.9 * HRT, 0.0, 0.0)), Inside::Surface);
        assert_eq!(t.inside(v3(10.0 + 1.1 * HRT, 0.0, 0.0)), Inside::Outside);
        assert_eq!(t.inside(v3(10.0 - 1.1 * HRT, 0.0, 0.0)), Inside::Inside);
        assert_eq!(t.inside(v3(0.0, 0.0, 20.0)), Inside::Surface);
        assert_eq!(t.inside(v3(1.0, 1.0, -20.0 - 0.9 * HCT)), Inside::Surface);
        assert_eq!(t.inside(v3(1.0, 1.0, -20.0 - 1.1 * HCT)), Inside::Outside);
        assert_eq!(t.inside(v3(1.0, 1.0, 20.0 - 1.1 * HCT)), Inside::Inside);
        assert_eq!(t.inside(v3(10.0, 0.0, 20.0)), Inside::Surface); // rim
        assert_eq!(t.inside(v3(10.5, 0.0, 20.0)), Inside::Outside);
        assert_eq!(t.inside(v3(8.0, 8.0, 0.0)), Inside::Outside); // rho = 11.3
    }

    #[test]
    fn inside_hollow_cylinder() {
        let t = hollow();
        assert_eq!(t.inside(v3(7.0, 0.0, 0.0)), Inside::Inside);
        assert_eq!(t.inside(v3(0.0, 4.0, 0.0)), Inside::Surface);
        assert_eq!(t.inside(v3(4.0 + 1.1 * HRT, 0.0, 0.0)), Inside::Inside);
        assert_eq!(t.inside(v3(4.0 - 0.9 * HRT, 0.0, 0.0)), Inside::Surface);
        assert_eq!(t.inside(v3(4.0 - 1.1 * HRT, 0.0, 0.0)), Inside::Outside);
        assert_eq!(t.inside(Vec3::ZERO), Inside::Outside); // in the hole
        assert_eq!(t.inside(v3(2.0, 0.0, 0.0)), Inside::Outside);
        assert_eq!(t.inside(v3(2.0, 0.0, 20.0)), Inside::Outside); // hole, at the cap
        assert_eq!(t.inside(v3(7.0, 0.0, 20.0)), Inside::Surface);
    }

    #[test]
    fn inside_phi_segment_up_to_pi() {
        let t = quarter();
        assert_eq!(t.inside(v3(5.0, 5.0, 0.0)), Inside::Inside);
        assert_eq!(t.inside(v3(5.0, 0.0, 0.0)), Inside::Surface); // start plane
        assert_eq!(t.inside(v3(0.0, 5.0, 0.0)), Inside::Surface); // end plane
        assert_eq!(t.inside(Vec3::ZERO), Inside::Surface); // z axis
        assert_eq!(t.inside(v3(0.0, 0.0, 20.0)), Inside::Surface);
        assert_eq!(t.inside(v3(-5.0, 5.0, 0.0)), Inside::Outside);
        assert_eq!(t.inside(v3(5.0, -5.0, 0.0)), Inside::Outside);
        assert_eq!(t.inside(v3(-5.0, -5.0, 0.0)), Inside::Outside);
        // The phi shell is angular: half-width kAngTolerance/2 about the plane.
        let at = |phi: f64| v3(5.0 * phi.cos(), 5.0 * phi.sin(), 0.0);
        assert_eq!(t.inside(at(-0.9 * HALF_ANG_TOLERANCE)), Inside::Surface);
        assert_eq!(t.inside(at(-1.1 * HALF_ANG_TOLERANCE)), Inside::Outside);
        assert_eq!(t.inside(at(1.1 * HALF_ANG_TOLERANCE)), Inside::Inside);
        assert_eq!(
            t.inside(at(FRAC_PI_2 + 1.1 * HALF_ANG_TOLERANCE)),
            Inside::Outside
        );
        // On rmax, within the phi range.
        assert_eq!(t.inside(at(0.5) * 2.0), Inside::Surface);
    }

    #[test]
    fn inside_phi_segment_above_pi_and_across_zero() {
        let t = three_quarter();
        assert_eq!(t.inside(v3(-5.0, -5.0, 0.0)), Inside::Inside);
        assert_eq!(t.inside(v3(-5.0, 5.0, 0.0)), Inside::Inside);
        assert_eq!(t.inside(v3(5.0, -5.0, 0.0)), Inside::Outside);
        assert_eq!(t.inside(v3(5.0, 0.0, 0.0)), Inside::Surface);
        assert_eq!(t.inside(v3(0.0, -5.0, 0.0)), Inside::Surface);

        // [-pi/4, pi/4]: normalised start is negative.
        let t = tubs(0.0, 10.0, 20.0, -FRAC_PI_4, FRAC_PI_2);
        assert_eq!(t.inside(v3(5.0, 0.0, 0.0)), Inside::Inside);
        assert_eq!(t.inside(v3(5.0, 1.0, 0.0)), Inside::Inside);
        assert_eq!(t.inside(v3(5.0, -1.0, 0.0)), Inside::Inside);
        assert_eq!(t.inside(v3(4.0, 4.0, 0.0)), Inside::Surface);
        assert_eq!(t.inside(v3(4.0, -4.0, 0.0)), Inside::Surface);
        assert_eq!(t.inside(v3(-5.0, 0.0, 0.0)), Inside::Outside);
        assert_eq!(t.inside(v3(0.0, 5.0, 0.0)), Inside::Outside);

        // [pi, 2 pi]: a point at phi = 0 is on the end plane.
        let t = tubs(0.0, 10.0, 20.0, PI, PI);
        assert_eq!(t.inside(v3(0.0, -5.0, 0.0)), Inside::Inside);
        assert_eq!(t.inside(v3(5.0, 0.0, 0.0)), Inside::Surface);
        assert_eq!(t.inside(v3(-5.0, 0.0, 0.0)), Inside::Surface);
        assert_eq!(t.inside(v3(5.0, 1.0, 0.0)), Inside::Outside);
    }

    #[test]
    fn inside_follows_the_centre_offset() {
        let c = v3(100.0, -50.0, 7.0);
        let t = Volume::tubs(c, 0.0, 10.0, 20.0, 0.0, TAU).unwrap();
        assert_eq!(t.inside(c), Inside::Inside);
        assert_eq!(t.inside(c + v3(10.0, 0.0, 0.0)), Inside::Surface);
        assert_eq!(t.inside(Vec3::ZERO), Inside::Outside);
        assert_eq!(t.inside(v3(f64::NAN, 0.0, 0.0)), Inside::Outside);
    }

    // --- distance_to_out: radial and axial ---

    #[test]
    fn radial_exits_of_a_solid_cylinder() {
        let t = cylinder();
        for v in [
            dir(1.0, 0.0, 0.0),
            dir(-1.0, 0.0, 0.0),
            dir(0.0, 1.0, 0.0),
            dir(0.0, -1.0, 0.0),
            dir(1.0, 1.0, 0.0),
            dir(-3.0, 4.0, 0.0),
        ] {
            assert_relative_eq!(d(&t, Vec3::ZERO, v), 10.0, max_relative = 1e-14);
        }
        // From (3, 4, 0) along +x: x^2 + 16 = 100.
        let got = d(&t, v3(3.0, 4.0, 0.0), dir(1.0, 0.0, 0.0));
        assert_relative_eq!(got, 84f64.sqrt() - 3.0, max_relative = 1e-14);
        let got = d(&t, v3(3.0, 4.0, 0.0), dir(-1.0, 0.0, 0.0));
        assert_relative_eq!(got, 84f64.sqrt() + 3.0, max_relative = 1e-14);
    }

    #[test]
    fn axial_and_oblique_exits() {
        let t = cylinder();
        assert_eq!(d(&t, Vec3::ZERO, dir(0.0, 0.0, 1.0)), 20.0);
        assert_eq!(d(&t, v3(0.0, 0.0, 5.0), dir(0.0, 0.0, -1.0)), 25.0);
        // (0.6, 0, 0.8): rmax at t = 16.67 before z = 20 at t = 25.
        let got = d(&t, Vec3::ZERO, dir(0.6, 0.0, 0.8));
        assert_relative_eq!(got, 10.0 / 0.6, max_relative = 1e-14);
        // (0.3, 0, 0.954): z = 20 at t = 20.96 before rmax at t = 33.3.
        let vz = (1.0f64 - 0.09).sqrt();
        let got = d(&t, Vec3::ZERO, dir(0.3, 0.0, vz));
        assert_relative_eq!(got, 20.0 / vz, max_relative = 1e-14);
    }

    #[test]
    fn exits_through_the_rmin_hole_wall() {
        let t = hollow();
        assert_relative_eq!(d(&t, v3(7.0, 0.0, 0.0), dir(-1.0, 0.0, 0.0)), 3.0);
        assert_relative_eq!(d(&t, v3(7.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)), 3.0);
        assert_relative_eq!(d(&t, v3(0.0, -6.0, 0.0), dir(0.0, 1.0, 0.0)), 2.0);
        // Chord through the hole at y = 2: rmin at x = sqrt(12).
        let got = d(&t, v3(7.0, 2.0, 0.0), dir(-1.0, 0.0, 0.0));
        assert_relative_eq!(got, 7.0 - 12f64.sqrt(), max_relative = 1e-14);
        // Oblique, towards the hole and down: the rmin wall is reached before z = -20.
        let got = d(&t, v3(7.0, 0.0, 0.0), dir(-0.6, 0.0, -0.8));
        assert_relative_eq!(got, 3.0 / 0.6, max_relative = 1e-14);
        // Missing the hole: from (7, 0) along +y to rmax at y = sqrt(51).
        let got = d(&t, v3(7.0, 0.0, 0.0), dir(0.0, 1.0, 0.0));
        assert_relative_eq!(got, 51f64.sqrt(), max_relative = 1e-14);
        // Tangent to rmin (y = 4) is not an exit: the ray crosses to rmax at x = -sqrt(84).
        let got = d(&t, v3(7.0, 4.0, 0.0), dir(-1.0, 0.0, 0.0));
        assert_relative_eq!(got, 7.0 + 84f64.sqrt(), max_relative = 1e-14);
    }

    // --- distance_to_out: phi planes ---

    #[test]
    fn phi_plane_exits_up_to_pi() {
        let t = quarter();
        let p = v3(5.0, 5.0, 0.0);
        assert_relative_eq!(d(&t, p, dir(0.0, -1.0, 0.0)), 5.0, max_relative = 1e-14); // y = 0
        assert_relative_eq!(d(&t, p, dir(-1.0, 0.0, 0.0)), 5.0, max_relative = 1e-14); // x = 0
        let got = d(&t, p, dir(1.0, 1.0, 0.0));
        assert_relative_eq!(got, 10.0 - 50f64.sqrt(), max_relative = 1e-14);
        // Oblique onto the start plane: y = 0 at t = 5 / 0.6.
        let got = d(&t, v3(2.0, 5.0, 0.0), dir(0.0, -0.6, 0.8));
        assert_relative_eq!(got, 5.0 / 0.6, max_relative = 1e-14);
        // Through the edge where the two planes meet, the z axis.
        let got = d(&t, p, dir(-1.0, -1.0, 0.0));
        assert_relative_eq!(got, 50f64.sqrt(), max_relative = 1e-12);
    }

    #[test]
    fn phi_plane_exits_above_pi_and_across_zero() {
        let t = three_quarter();
        // From phi = 3 pi/4 along -y: crosses phi = pi (inside) and leaves through rmax.
        let got = d(&t, v3(-5.0, 5.0, 0.0), dir(0.0, -1.0, 0.0));
        assert_relative_eq!(got, 5.0 + 75f64.sqrt(), max_relative = 1e-14);
        // From phi = 5 pi/4 along +x: end plane phi = 3 pi/2 at x = 0.
        assert_relative_eq!(
            d(&t, v3(-5.0, -5.0, 0.0), dir(1.0, 0.0, 0.0)),
            5.0,
            max_relative = 1e-14
        );
        // From phi = pi/4 along -y: start plane at y = 0.
        assert_relative_eq!(
            d(&t, v3(5.0, 5.0, 0.0), dir(0.0, -1.0, 0.0)),
            5.0,
            max_relative = 1e-14
        );

        let t = tubs(0.0, 10.0, 20.0, -FRAC_PI_4, FRAC_PI_2);
        assert_relative_eq!(
            d(&t, v3(5.0, 0.0, 0.0), dir(0.0, 1.0, 0.0)),
            5.0,
            max_relative = 1e-14
        );
        assert_relative_eq!(
            d(&t, v3(5.0, 0.0, 0.0), dir(0.0, -1.0, 0.0)),
            5.0,
            max_relative = 1e-14
        );
        assert_relative_eq!(
            d(&t, v3(5.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)),
            5.0,
            max_relative = 1e-14
        );
    }

    #[test]
    fn ray_crossing_the_z_axis_inside_the_phi_range() {
        // From phi = 5 pi/4 along (1, 1)/sqrt 2: through the axis at t = 5 sqrt 2, then on
        // at phi = pi/4 (inside [0, 3 pi/2]) to rmax = 10. The crossing on the axis is not
        // an exit because the direction is inside the phi range (G4Tubs.cc:1407-1418).
        let got = d(&three_quarter(), v3(-5.0, -5.0, 0.0), dir(1.0, 1.0, 0.0));
        assert_relative_eq!(got, 5.0 * 2f64.sqrt() + 10.0, max_relative = 1e-14);
    }

    /// The case proptest saved in `proptest-regressions/geometry/tubs.txt`: an exit through
    /// the end half-plane 1.5e-3 nm from the z axis. The exit point is on the half-plane to
    /// within the Cartesian tolerance, but the rounding of its coordinates at a 6e5 nm
    /// offset puts its angle beyond kAngTolerance/2 of the end angle, so the angular phi
    /// shell of `Inside` classifies it as Outside (a Geant4 property, see the module doc).
    #[test]
    fn near_axis_phi_exit_is_on_the_half_plane_but_outside_in_angle() {
        let (start, delta) = (0.41402592255895654, 4.185937526460339);
        let volume = Volume::tubs(
            v3(-621521.552021449, 0.0, 0.0),
            0.0,
            5030.883888057993,
            1.0,
            start,
            delta,
        )
        .unwrap();
        let p = point(&volume, 0.7803339153237773, 0.6516024423563239, 0.0);
        let v = dir(1.0, 0.0, 0.0);
        assert_eq!(volume.inside(p), Inside::Inside);
        let dist = d(&volume, p, v);
        let exit = p + v.as_vec3() * dist;
        let q = exit - volume.centre();
        assert!(q.x.hypot(q.y) < 2e-3, "{q:?}");
        let (_, to_end) = half_plane_distances(solid(&volume), q);
        assert!(to_end <= HCT, "{to_end} nm from the end half-plane");
        let mut phi = q.y.atan2(q.x);
        if phi < 0.0 {
            phi += TAU;
        }
        let end = start + delta;
        assert!(
            (phi - end).abs() > HALF_ANG_TOLERANCE,
            "angle {phi} vs end {end}"
        );
        assert_eq!(volume.inside(exit), Inside::Outside);
    }

    #[test]
    fn start_on_the_z_axis_of_a_segment() {
        let t = quarter();
        assert_eq!(t.inside(Vec3::ZERO), Inside::Surface);
        // Direction inside the phi range: rmax limits the step.
        assert_relative_eq!(
            d(&t, Vec3::ZERO, dir(1.0, 1.0, 0.0)),
            10.0,
            max_relative = 1e-14
        );
        assert_relative_eq!(
            d(&t, Vec3::ZERO, dir(1.0, 0.0, 0.0)),
            10.0,
            max_relative = 1e-14
        ); // along a plane
        // Direction outside the phi range: leaves at once.
        assert_eq!(d(&t, Vec3::ZERO, dir(-1.0, 0.0, 0.0)), 0.0);
        assert_eq!(d(&t, Vec3::ZERO, dir(-1.0, -1.0, 0.5)), 0.0);
        // Along the axis: the z plane.
        assert_eq!(d(&t, Vec3::ZERO, dir(0.0, 0.0, 1.0)), 20.0);
    }

    /// The deviation of the shared phi exit for the tube: on the z axis of phi in
    /// [pi/2, pi], along a z direction that rounds to just below unit length. `t1 = 2.2e-16`
    /// lets the phi block run (G4Tubs.cc:1204), where `atan2(0, 0) = 0` lies outside the
    /// segment, so G4Tubs returns 0 (G4Tubs.cc:1437-1450). The ray runs along the edge, so
    /// the port lets the z plane limit it.
    #[test]
    fn start_on_the_z_axis_moving_along_it_with_a_rounded_direction() {
        let t = tubs(0.0, 10.0, 20.0, FRAC_PI_2, FRAC_PI_2);
        let v = Direction::new(v3(0.0, 0.0, 49.0)).unwrap();
        let w = v.as_vec3();
        assert_eq!((w.x, w.y), (0.0, 0.0));
        assert!(
            w.z < 1.0 && 1.0 - w.z * w.z > 0.0,
            "t1 = {}",
            1.0 - w.z * w.z
        );
        let p = v3(0.0, 0.0, 5.0);
        assert_eq!(t.inside(p), Inside::Surface);
        assert_relative_eq!(d(&t, p, v), 15.0, max_relative = 1e-14);
    }

    // --- distance_to_out: parallel rays ---

    #[test]
    fn rays_parallel_to_the_z_axis() {
        assert_eq!(d(&hollow(), v3(7.0, 0.0, -5.0), dir(0.0, 0.0, 1.0)), 25.0);
        // Along the rmax and rmin surfaces and along a phi plane.
        assert_eq!(
            d(&cylinder(), v3(10.0, 0.0, 0.0), dir(0.0, 0.0, -1.0)),
            20.0
        );
        assert_eq!(d(&hollow(), v3(0.0, 4.0, 10.0), dir(0.0, 0.0, 1.0)), 10.0);
        assert_eq!(d(&quarter(), v3(5.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)), 20.0);
        assert_eq!(d(&quarter(), v3(0.0, 5.0, 0.0), dir(0.0, 0.0, -1.0)), 20.0);
    }

    #[test]
    fn rays_parallel_to_the_end_caps() {
        let t = cylinder();
        assert_relative_eq!(d(&t, v3(0.0, 0.0, 15.0), dir(1.0, 0.0, 0.0)), 10.0);
        // On a cap, moving along it.
        assert_relative_eq!(d(&t, v3(0.0, 0.0, 20.0), dir(1.0, 0.0, 0.0)), 10.0);
        let got = d(&t, v3(3.0, 0.0, -20.0), dir(0.0, 1.0, 0.0));
        assert_relative_eq!(got, 91f64.sqrt(), max_relative = 1e-14);
        // Phi segment, on a cap, moving along it to the start plane.
        assert_relative_eq!(
            d(&quarter(), v3(5.0, 5.0, 20.0), dir(0.0, -1.0, 0.0)),
            5.0,
            max_relative = 1e-14
        );
    }

    // --- distance_to_out: surface starts ---

    #[test]
    fn surface_start_moving_outward_gives_zero() {
        let cases = [
            (cylinder(), v3(10.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)), // rmax
            (cylinder(), v3(10.0, 0.0, 0.0), dir(1.0, 1.0, 0.3)),
            (cylinder(), v3(10.0, 0.0, 0.0), dir(0.0, 1.0, 0.0)), // tangent to rmax
            (
                cylinder(),
                v3(10.0 + 0.9 * HRT, 0.0, 0.0),
                dir(1.0, 0.0, 0.0),
            ),
            (
                cylinder(),
                v3(10.0 - 0.9 * HRT, 0.0, 0.0),
                dir(1.0, 0.0, 0.0),
            ),
            (cylinder(), v3(1.0, 1.0, 20.0), dir(0.0, 0.0, 1.0)), // + cap
            (cylinder(), v3(1.0, 1.0, -20.0), dir(1.0, 0.0, -1e-6)), // - cap, grazing
            (cylinder(), v3(10.0, 0.0, 20.0), dir(-1.0, 0.0, 1.0)), // rim, via the cap
            (hollow(), v3(4.0, 0.0, 0.0), dir(-1.0, 0.0, 0.0)),   // rmin, into the hole
            (hollow(), v3(0.0, -4.0, 0.0), dir(0.3, 1.0, 0.2)),
            (hollow(), v3(4.0 - 0.9 * HRT, 0.0, 0.0), dir(-1.0, 0.0, 0.0)),
            (quarter(), v3(5.0, 0.0, 0.0), dir(0.0, -1.0, 0.0)), // start plane
            (quarter(), v3(5.0, 0.0, 0.0), dir(1.0, -1.0, 0.5)),
            (quarter(), v3(0.0, 5.0, 0.0), dir(-1.0, 0.0, 0.0)), // end plane
            (three_quarter(), v3(0.0, -5.0, 0.0), dir(1.0, 0.0, 0.0)), // end plane
        ];
        for (t, p, v) in cases {
            assert_ne!(t.inside(p), Inside::Outside, "{p:?}");
            assert_eq!(t.distance_to_out(p, v), Some(0.0), "{p:?} {v:?}");
        }
    }

    #[test]
    fn surface_start_moving_inward_crosses_the_solid() {
        assert_relative_eq!(
            d(&cylinder(), v3(10.0, 0.0, 0.0), dir(-1.0, 0.0, 0.0)),
            20.0
        );
        assert_eq!(
            d(&cylinder(), v3(1.0, 1.0, -20.0), dir(0.0, 0.0, 1.0)),
            40.0
        );
        assert_relative_eq!(d(&hollow(), v3(4.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)), 6.0);
        let got = d(&quarter(), v3(5.0, 0.0, 0.0), dir(0.0, 1.0, 0.0));
        assert_relative_eq!(got, 75f64.sqrt(), max_relative = 1e-14);
        assert_relative_eq!(
            d(&quarter(), v3(0.0, 5.0, 0.0), dir(1.0, 0.0, 0.0)),
            75f64.sqrt(),
            max_relative = 1e-14
        );
        let outer = v3(10.0 + 0.9 * HRT, 0.0, 0.0);
        let got = d(&cylinder(), outer, dir(-1.0, 0.0, 0.0));
        assert_relative_eq!(got, 20.0 + 0.9 * HRT, max_relative = 1e-14);
    }

    #[test]
    fn outside_start_gives_none() {
        let v = dir(-1.0, 0.0, 0.0);
        assert_eq!(cylinder().distance_to_out(v3(11.0, 0.0, 0.0), v), None);
        assert_eq!(hollow().distance_to_out(v3(2.0, 0.0, 0.0), v), None);
        assert_eq!(quarter().distance_to_out(v3(5.0, -5.0, 0.0), v), None);
        assert_eq!(
            cylinder().distance_to_out(v3(0.0, 0.0, 20.0 + 1.1 * HCT), v),
            None
        );
    }

    #[test]
    fn exits_with_a_centre_offset() {
        let c = v3(1.0e3, -2.0e3, 5.0e2);
        let t = Volume::tubs(c, 4.0, 10.0, 20.0, 0.0, FRAC_PI_2).unwrap();
        assert_relative_eq!(
            d(&t, c + v3(7.0, 1.0, 0.0), dir(0.0, -1.0, 0.0)),
            1.0,
            max_relative = 1e-12
        );
        assert_relative_eq!(
            d(&t, c + v3(5.0, 5.0, 0.0), dir(1.0, 1.0, 0.0)),
            10.0 - 50f64.sqrt(),
            max_relative = 1e-12
        );
        assert_eq!(
            t.distance_to_out(v3(7.0, 1.0, 0.0), dir(0.0, -1.0, 0.0)),
            None
        );
    }

    // --- invariants ---

    /// Radius about the axis (nm) within which a phi-plane exit point is checked against the
    /// Cartesian rather than the angular tolerance: coordinates up to 2e6 nm round to about
    /// 1e-9 nm, which is `kAngTolerance/2` at 2 nm from the axis.
    const AXIS_ZONE: f64 = 10.0;

    /// Opening angles of the generated phi segments, rad.
    const SEGMENT: std::ops::Range<f64> = 0.2..(TAU - 0.01);

    /// Tube segment with rmax and half_z in [1, 1e6) nm, `rmin = rmin_fraction * rmax`, an
    /// opening `delta` and a start in [-4 pi, 4 pi) rad.
    fn tubs_with(
        rmin_fraction: impl Strategy<Value = f64>,
        delta: impl Strategy<Value = f64>,
    ) -> impl Strategy<Value = Volume> {
        (
            centre(),
            size(),
            rmin_fraction,
            size(),
            -2.0 * TAU..2.0 * TAU,
            delta,
        )
            .prop_map(|(c, rmax, f, half_z, start, delta)| {
                Volume::tubs(c, f * rmax, rmax, half_z, start, delta).unwrap()
            })
    }

    /// rmin = 0 (one case in four) or up to 0.9 rmax, and a full circle (one case in four)
    /// or a [`SEGMENT`] opening.
    fn tubs_strategy() -> impl Strategy<Value = Volume> {
        tubs_with(
            prop_oneof![1 => Just(0.0), 3 => 0.0..0.9],
            prop_oneof![1 => Just(TAU), 3 => SEGMENT],
        )
    }

    /// Point at radius fraction `fr` of [rmin, rmax], phi fraction `fphi` of the requested
    /// range and z fraction `fz` of half_z.
    fn point(volume: &Volume, fr: f64, fphi: f64, fz: f64) -> Vec3 {
        let t = solid(volume);
        let r = t.rmin() + fr * (t.rmax() - t.rmin());
        let phi = t.requested_start_phi() + fphi * t.requested_delta_phi();
        point_at(volume, r, phi, fz * t.half_z())
    }

    fn point_at(volume: &Volume, r: f64, phi: f64, z: f64) -> Vec3 {
        volume.centre() + v3(r * phi.cos(), r * phi.sin(), z)
    }

    /// Unit vector with component `a` along the outward normal `n`; the rest is split
    /// between `u` and `w` (orthonormal, both orthogonal to `n`) by the angle `psi`.
    fn leaving(n: Vec3, u: Vec3, w: Vec3, a: f64, psi: f64) -> Direction {
        let b = (1.0 - a * a).max(0.0).sqrt();
        Direction::new(n * a + (u * psi.cos() + w * psi.sin()) * b).unwrap()
    }

    /// Radial and azimuthal unit vectors at `phi`.
    fn radial(phi: f64) -> (Vec3, Vec3) {
        (
            v3(phi.cos(), phi.sin(), 0.0),
            v3(-phi.sin(), phi.cos(), 0.0),
        )
    }

    const Z: Vec3 = Vec3::new(0.0, 0.0, 1.0);

    /// Cartesian distances (nm) of the local point `q` from the start and end half-planes
    /// of `t`. A point on the far side of the z axis from a half-plane (by more than
    /// kCarTolerance along it) is infinitely far from that half-plane.
    fn half_plane_distances(t: &super::TubsSolid, q: Vec3) -> (f64, f64) {
        let distance = |phi: f64| {
            let (sin, cos) = phi.sin_cos();
            if q.x * cos + q.y * sin >= -CAR_TOLERANCE {
                (q.x * sin - q.y * cos).abs()
            } else {
                f64::INFINITY
            }
        };
        let start = t.start_phi();
        (distance(start), distance(start + t.delta_phi()))
    }

    proptest! {
        #[test]
        fn exit_point_is_on_the_surface(
            volume in tubs_strategy(),
            fr in 0.05..0.95f64,
            fphi in 0.02..0.98f64,
            fz in FRACTION,
            v in unit_direction(),
        ) {
            let p = point(&volume, fr, fphi, fz);
            // Very close to the axis of a narrow segment the point can be on the surface.
            prop_assume!(volume.inside(p) == Inside::Inside);
            let t = solid(&volume);
            let d = volume.distance_to_out(p, v).unwrap();
            let q = p + v.as_vec3() * d - volume.centre();
            if t.delta_phi() < TAU && q.x.hypot(q.y) < AXIS_ZONE {
                // Near the axis the angular phi shell is thinner than the rounding of the
                // exit point, which may then classify as Outside, as in Geant4. Require it
                // to be within the Cartesian tolerance of a phi plane instead.
                prop_assert!(d > 0.0 && d.is_finite());
                prop_assert!(q.x.hypot(q.y) <= t.rmax() + HRT, "exit {:?} beyond rmax", q);
                prop_assert!(q.z.abs() <= t.half_z() + HCT, "exit {:?} beyond a cap", q);
                let (to_start, to_end) = half_plane_distances(t, q);
                let on_cap = (q.z.abs() - t.half_z()).abs() <= HCT;
                prop_assert!(
                    to_start.min(to_end) <= HCT
                        || on_cap
                        || volume.inside(p + v.as_vec3() * d) == Inside::Surface,
                    "near-axis exit {:?} is {} / {} nm from the phi half-planes",
                    q, to_start, to_end
                );
            } else {
                check_exit_invariants(&volume, p, v)?;
            }
            // No chord is longer than the diagonal of the cylinder's axial section.
            let diagonal = 2.0 * (t.rmax().powi(2) + t.half_z().powi(2)).sqrt();
            prop_assert!(d <= diagonal * (1.0 + 1e-12), "{} > {}", d, diagonal);
        }

        /// The outward radial component is at least 0.1, so that the ray leaves the
        /// tolerant rmax shell before the z plane (`roi2` test, G4Tubs.cc:1208).
        #[test]
        fn leaving_through_rmax_gives_zero(
            volume in tubs_strategy(),
            fphi in 0.02..0.98f64,
            fz in FRACTION,
            a in 0.1..=1.0f64,
            psi in 0.0..TAU,
        ) {
            let t = solid(&volume);
            let phi = t.requested_start_phi() + fphi * t.requested_delta_phi();
            let p = point_at(&volume, t.rmax(), phi, fz * t.half_z());
            let (rho, phi_hat) = radial(phi);
            let v = leaving(rho, phi_hat, Z, a, psi);
            prop_assert_eq!(volume.inside(p), Inside::Surface);
            prop_assert_eq!(volume.distance_to_out(p, v), Some(0.0));
        }

        /// rmin is at least 0.1 rmax and the inward radial component at least 0.2, so that
        /// the ray is not tangent to rmin within kRadTolerance (G4Tubs.cc:1244).
        #[test]
        fn leaving_through_rmin_gives_zero(
            volume in tubs_with(0.1..0.9, prop_oneof![Just(TAU), SEGMENT]),
            fphi in 0.02..0.98f64,
            fz in FRACTION,
            a in 0.2..=1.0f64,
            psi in 0.0..TAU,
        ) {
            let t = solid(&volume);
            let phi = t.requested_start_phi() + fphi * t.requested_delta_phi();
            let p = point_at(&volume, t.rmin(), phi, fz * t.half_z());
            let (rho, phi_hat) = radial(phi);
            let v = leaving(rho * -1.0, phi_hat, Z, a, psi);
            prop_assert_eq!(volume.inside(p), Inside::Surface);
            prop_assert_eq!(volume.distance_to_out(p, v), Some(0.0));
        }

        /// Any direction with a z component towards the cap, down to grazing.
        #[test]
        fn leaving_through_an_end_cap_gives_zero(
            volume in tubs_strategy(),
            fr in 0.05..0.95f64,
            fphi in 0.02..0.98f64,
            up in any::<bool>(),
            a in 1e-6..=1.0f64,
            psi in 0.0..TAU,
        ) {
            let t = solid(&volume);
            let mid = point(&volume, fr, fphi, 0.0);
            prop_assume!(volume.inside(mid) == Inside::Inside);
            let n = if up { Z } else { Z * -1.0 };
            let p = mid + n * t.half_z();
            let v = leaving(n, v3(1.0, 0.0, 0.0), v3(0.0, 1.0, 0.0), a, psi);
            prop_assert_eq!(volume.inside(p), Inside::Surface);
            prop_assert_eq!(volume.distance_to_out(p, v), Some(0.0));
        }

        /// A start on the start or end plane, with an outward normal component of at least
        /// 0.1 so that the plane-crossing distance `pDist/comp` stays within tolerance.
        #[test]
        fn leaving_through_a_phi_plane_gives_zero(
            volume in tubs_with(prop_oneof![1 => Just(0.0), 3 => 0.0..0.9], SEGMENT),
            fr in 0.05..0.95f64,
            fz in FRACTION,
            end_plane in any::<bool>(),
            a in 0.1..=1.0f64,
            psi in 0.0..TAU,
        ) {
            let t = solid(&volume);
            let phi = t.requested_start_phi()
                + if end_plane { t.requested_delta_phi() } else { 0.0 };
            let r = t.rmin() + fr * (t.rmax() - t.rmin());
            let p = point_at(&volume, r, phi, fz * t.half_z());
            // Outward normals: (sin s, -cos s, 0) at the start, (-sin e, cos e, 0) at the end.
            let (rho, phi_hat) = radial(phi);
            let n = if end_plane { phi_hat } else { phi_hat * -1.0 };
            let v = leaving(n, rho, Z, a, psi);
            prop_assert_eq!(volume.inside(p), Inside::Surface);
            prop_assert_eq!(volume.distance_to_out(p, v), Some(0.0));
        }

        #[test]
        fn serde_round_trip_preserves_the_tubs(volume in tubs_strategy()) {
            let text = toml::to_string(&volume).unwrap();
            let back: Volume = toml::from_str(&text).unwrap();
            prop_assert_eq!(back, volume);
        }
    }
}
