//! Azimuthal (phi) segment shared by the solids with a phi range: its Geant4 normalisation,
//! the trigonometry of its bounding half-planes and the distance to exit through them.
//!
//! `G4Tubs` and `G4Sphere` normalise the phi range the same way (`G4Tubs.icc:111-161`,
//! `G4Sphere.icc:193-246`) and precompute the same sines and cosines (`G4Tubs.icc:94-109`,
//! `G4Sphere.icc:121-136`). Their `Inside` phi tests differ, so those stay with each shape.
//! [`PhiSegment::exit_distance`] is the phi block of `G4Tubs::DistanceToOut(p, v)`; the
//! sphere uses it too, in place of the older `G4Sphere` phi block (see `sphere.rs`).

use super::{CAR_TOLERANCE, GeometryError, HALF_ANG_TOLERANCE, HALF_CAR_TOLERANCE};
use crate::vec3::Vec3;
use std::f64::consts::{PI, TAU};

/// Phi range `[start, start + delta]` about the z axis, in rad, after Geant4 normalisation.
///
/// The requested values are kept as given, so that serialisation writes back what the user
/// wrote; the normalised values drive the algorithms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PhiSegment {
    requested_start: f64,
    requested_delta: f64,
    /// Geant4 `fSPhi`: in `[0, 2 pi)`, or in `(-2 pi, 0)` when the segment crosses phi = 0.
    start: f64,
    /// Geant4 `fDPhi`: in `(0, 2 pi]`.
    delta: f64,
    /// Geant4 `fPhiFullTube` / `fFullPhiSphere`.
    full: bool,
    sin_start: f64,
    cos_start: f64,
    sin_end: f64,
    cos_end: f64,
    sin_centre: f64,
    cos_centre: f64,
}

impl PhiSegment {
    /// Checks and normalises `(start_phi, delta_phi)` as `G4Tubs::CheckPhiAngles`
    /// (`G4Tubs.icc:156-161`) does.
    ///
    /// - `delta_phi` must lie in `(0, 2 pi]`. A value of at least `2 pi - kAngTolerance/2` is
    ///   a full circle: `delta = 2 pi` and `start = 0`, whatever `start_phi` is
    ///   (`G4Tubs.icc:129-154`). Geant4 silently resets a `delta_phi` above `2 pi` to `2 pi`;
    ///   it is rejected here so that a typing error in a configuration is not hidden.
    /// - Otherwise a non-zero `start_phi` is reduced with `fmod` to `[0, 2 pi)`, then shifted
    ///   by `-2 pi` if the segment would end beyond `2 pi` (`G4Tubs.icc:111-127`).
    ///
    /// Non-finite values are rejected, which Geant4 does not check.
    pub(super) fn new(start_phi: f64, delta_phi: f64) -> Result<Self, GeometryError> {
        for (field, value) in [("start_phi", start_phi), ("delta_phi", delta_phi)] {
            if !value.is_finite() {
                return Err(GeometryError::NonFinite { field, value });
            }
        }
        if delta_phi <= 0.0 || delta_phi > TAU {
            return Err(GeometryError::AngleOutOfRange {
                field: "delta_phi",
                value: delta_phi,
                range: "(0, 2 pi]",
            });
        }
        // CheckDPhiAngle, G4Tubs.icc:129-154.
        let full = delta_phi >= TAU - HALF_ANG_TOLERANCE;
        let delta = if full { TAU } else { delta_phi };
        // CheckPhiAngles, G4Tubs.icc:159: fSPhi stays 0 for a full circle or a zero start.
        let start = if !full && start_phi != 0.0 {
            normalised_start(start_phi, delta)
        } else {
            0.0
        };
        // InitializeTrigonometry, G4Tubs.icc:94-109.
        let centre = start + 0.5 * delta;
        let end = start + delta;
        Ok(Self {
            requested_start: start_phi,
            requested_delta: delta_phi,
            start,
            delta,
            full,
            sin_start: start.sin(),
            cos_start: start.cos(),
            sin_end: end.sin(),
            cos_end: end.cos(),
            sin_centre: centre.sin(),
            cos_centre: centre.cos(),
        })
    }

    /// `start_phi` as passed to [`PhiSegment::new`], rad.
    pub(super) fn requested_start(&self) -> f64 {
        self.requested_start
    }

    /// `delta_phi` as passed to [`PhiSegment::new`], rad.
    pub(super) fn requested_delta(&self) -> f64 {
        self.requested_delta
    }

    /// Normalised start angle (Geant4 `fSPhi`), rad.
    pub(super) fn start(&self) -> f64 {
        self.start
    }

    /// Normalised opening angle (Geant4 `fDPhi`), rad.
    pub(super) fn delta(&self) -> f64 {
        self.delta
    }

    /// End angle `start + delta`, rad.
    pub(super) fn end(&self) -> f64 {
        self.start + self.delta
    }

    /// True for a full circle, which has no phi boundary.
    pub(super) fn is_full(&self) -> bool {
        self.full
    }

    /// `(sin, cos)` of the start angle (Geant4 `sinSPhi`, `cosSPhi`).
    pub(super) fn start_trig(&self) -> (f64, f64) {
        (self.sin_start, self.cos_start)
    }

    /// `(sin, cos)` of the end angle (Geant4 `sinEPhi`, `cosEPhi`).
    pub(super) fn end_trig(&self) -> (f64, f64) {
        (self.sin_end, self.cos_end)
    }
    /// Safety to the phi planes, the phi block of `G4Tubs::DistanceToOut(p)`
    /// (`G4Tubs.cc:1583-1596`) and `G4Sphere::DistanceToOut(p)` (`G4Sphere.cc:2646-2665`):
    /// the distance to the nearer full plane, chosen by the side of the centre direction.
    /// Not clamped (the caller clamps the overall minimum at 0). Only for a cut segment.
    pub(super) fn safety(&self, x: f64, y: f64) -> f64 {
        if y * self.cos_centre - x * self.sin_centre <= 0.0 {
            -(x * self.sin_start - y * self.cos_start)
        } else {
            x * self.sin_end - y * self.cos_end
        }
    }

    /// `(sin, cos)` of the centre angle `start + delta/2` (Geant4 `sinCPhi`, `cosCPhi`).
    pub(super) fn centre_trig(&self) -> (f64, f64) {
        (self.sin_centre, self.cos_centre)
    }

    /// Distance along `v` from `p` (relative to the solid's centre) to the phi planes, the
    /// phi block of `G4Tubs::DistanceToOut(p, v)` (`G4Tubs.cc:1320-1452`): `f64::INFINITY`
    /// if they do not limit the step, 0 for a start on a plane moving out through it.
    ///
    /// `pDistS`/`pDistE` are the signed distances to the full start/end planes (negative
    /// inside), and `compS`/`compE` the direction components along their inward normals
    /// (negative when heading out). A plane crossing counts only if it lies on the segment's
    /// half-plane, tested against the centre direction `c` with `yi cos c - xi sin c`. A
    /// crossing on the z axis counts only if the direction leaves the phi range. A start on
    /// the z axis leaves at once if its direction is outside the phi range. Moving along the
    /// axis (`v.x = v.y = 0`) it is not limited, as in `G4Sphere.cc:2439-2455`.
    ///
    /// That last guard is a deviation for the tube. `G4Tubs` skips its phi block only for
    /// `t1 = 1 - v.z^2 = 0` (`G4Tubs.cc:1204`). A direction along z that rounding leaves just
    /// short of unit length, such as `Direction::new((0, 0, 49))` with `v.z = 1 - 1.1e-16`
    /// and `t1 = 2.2e-16`, reaches its on-axis test with `atan2(0, 0) = 0`
    /// (`G4Tubs.cc:1437-1450`). That returns 0 unless phi = 0 lies in the segment, although
    /// the ray runs along the edge and does not leave through a phi plane.
    ///
    /// The on-axis start-plane crossing can return a distance down to `-kCarTolerance/2`;
    /// `G4Tubs` clamps it to 0 at the end (`G4Tubs.cc:1540`) and the sphere clamps it too.
    pub(super) fn exit_distance(&self, p: Vec3, v: Vec3) -> f64 {
        let (start, end, delta) = (self.start, self.end(), self.delta);
        // Direction angle in the range of the segment, G4Tubs.cc:1325-1328.
        let mut vphi = v.y.atan2(v.x);
        if vphi < start - HALF_ANG_TOLERANCE {
            vphi += TAU;
        } else if vphi > end + HALF_ANG_TOLERANCE {
            vphi -= TAU;
        }
        let v_in_range = start - HALF_ANG_TOLERANCE <= vphi && vphi <= end + HALF_ANG_TOLERANCE;

        if p.x == 0.0 && p.y == 0.0 {
            // On the z axis, G4Tubs.cc:1437-1452 and G4Sphere.cc:2434-2456; the v along z
            // guard is a deviation for the tube (see above).
            return if (v.x == 0.0 && v.y == 0.0) || v_in_range {
                f64::INFINITY
            } else {
                0.0
            };
        }

        let (sin_s, cos_s) = self.start_trig();
        let (sin_e, cos_e) = self.end_trig();
        let (sin_c, cos_c) = self.centre_trig();
        let p_dist_s = p.x * sin_s - p.y * cos_s;
        let p_dist_e = -p.x * sin_e + p.y * cos_e;
        let comp_s = -sin_s * v.x + cos_s * v.y;
        let comp_e = sin_e * v.x - cos_e * v.y;

        // Inside both full planes for a segment up to pi, either one above (G4Tubs.cc:1345-1348).
        let within_planes = if delta <= PI {
            p_dist_s <= HALF_CAR_TOLERANCE && p_dist_e <= HALF_CAR_TOLERANCE
        } else {
            p_dist_s <= HALF_CAR_TOLERANCE || p_dist_e <= HALF_CAR_TOLERANCE
        };
        if !within_planes {
            return f64::INFINITY; // G4Tubs.cc:1432-1435
        }
        let on_axis = |xi: f64, yi: f64| xi.abs() <= CAR_TOLERANCE && yi.abs() <= CAR_TOLERANCE;
        let on_segment_side = |xi: f64, yi: f64| yi * cos_c - xi * sin_c >= 0.0;

        // Start plane, G4Tubs.cc:1352-1394.
        let mut sphi = f64::INFINITY;
        if comp_s < 0.0 {
            let s = p_dist_s / comp_s;
            if s >= -HALF_CAR_TOLERANCE {
                let (xi, yi) = (p.x + s * v.x, p.y + s * v.y);
                sphi = if on_axis(xi, yi) {
                    if v_in_range { f64::INFINITY } else { s }
                } else if on_segment_side(xi, yi) {
                    f64::INFINITY // crosses the start plane's extension beyond the axis
                } else if p_dist_s > -HALF_CAR_TOLERANCE {
                    0.0 // on the start plane, leaving
                } else {
                    s
                };
            }
        }

        // End plane, only if before the start-plane crossing, G4Tubs.cc:1396-1430.
        if comp_e < 0.0 {
            let s2 = p_dist_e / comp_e;
            if s2 > -HALF_CAR_TOLERANCE && s2 < sphi {
                let (xi, yi) = (p.x + s2 * v.x, p.y + s2 * v.y);
                let leaves_by_end = if on_axis(xi, yi) {
                    !v_in_range
                } else {
                    on_segment_side(xi, yi)
                };
                if leaves_by_end {
                    sphi = if p_dist_e <= -HALF_CAR_TOLERANCE {
                        s2
                    } else {
                        0.0
                    };
                }
            }
        }
        sphi
    }
}

/// `CheckSPhiAngle` (`G4Tubs.icc:111-127`): reduce `start_phi` to `[0, 2 pi)`, then to
/// `(-2 pi, 0)` if the segment of opening `delta` would end beyond `2 pi`. Rust's `%` on
/// `f64` is C `fmod`.
fn normalised_start(start_phi: f64, delta: f64) -> f64 {
    let mut start = if start_phi < 0.0 {
        TAU - start_phi.abs() % TAU
    } else {
        start_phi % TAU
    };
    if start + delta > TAU {
        start -= TAU;
    }
    start
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f64::consts::{FRAC_PI_2, PI};

    #[test]
    fn full_circle_resets_the_start() {
        for (start, delta) in [
            (0.0, TAU),
            (1.0, TAU),
            (-3.0, TAU - 0.4 * HALF_ANG_TOLERANCE),
        ] {
            let s = PhiSegment::new(start, delta).unwrap();
            assert!(s.is_full(), "{start} {delta}");
            assert_eq!(s.start(), 0.0);
            assert_eq!(s.delta(), TAU);
            assert_eq!(s.requested_start(), start);
            assert_eq!(s.requested_delta(), delta);
        }
        // Just below the full-circle threshold is a segment.
        let s = PhiSegment::new(0.0, TAU - 2.0 * HALF_ANG_TOLERANCE).unwrap();
        assert!(!s.is_full());
    }

    #[test]
    fn start_is_reduced_as_in_geant4() {
        let cases = [
            // (start, delta, normalised start)
            (0.0, FRAC_PI_2, 0.0),
            (1.0, 1.0, 1.0),
            (3.0 * PI, FRAC_PI_2, PI),    // fmod(3 pi, 2 pi)
            (-FRAC_PI_2, PI, -FRAC_PI_2), // 2 pi - pi/2, then crosses 2 pi: -pi/2
            (-FRAC_PI_2, 1.0, 1.5 * PI),  // 2 pi - pi/2 = 3 pi/2, ends before 2 pi
            (1.5 * PI, PI, -FRAC_PI_2),   // ends at 5 pi/2 > 2 pi
            (-5.0 * PI, FRAC_PI_2, PI),   // 2 pi - fmod(5 pi, 2 pi) = pi
            (TAU, 1.0, 0.0),              // fmod(2 pi, 2 pi) = 0
            (-TAU, 1.0, 0.0),             // 2 pi - 0, then crosses 2 pi: 0
        ];
        for (start, delta, expected) in cases {
            let s = PhiSegment::new(start, delta).unwrap();
            assert!(!s.is_full());
            assert_relative_eq!(s.start(), expected, epsilon = 1e-14);
            assert_eq!(s.delta(), delta);
            assert_relative_eq!(s.end(), expected + delta, epsilon = 1e-14);
            assert!(s.start() > -TAU && s.start() < TAU);
            assert!(s.end() <= TAU);
        }
    }

    #[test]
    fn trigonometry_matches_the_angles() {
        let s = PhiSegment::new(0.25, 1.0).unwrap();
        assert_eq!(s.start_trig(), (0.25f64.sin(), 0.25f64.cos()));
        assert_eq!(s.end_trig(), (1.25f64.sin(), 1.25f64.cos()));
        assert_eq!(s.centre_trig(), (0.75f64.sin(), 0.75f64.cos()));
    }

    #[test]
    fn rejects_out_of_range_and_non_finite_angles() {
        for delta in [0.0, -0.0, -1.0, TAU * (1.0 + 1e-15), 7.0] {
            assert!(
                matches!(
                    PhiSegment::new(0.0, delta),
                    Err(GeometryError::AngleOutOfRange {
                        field: "delta_phi",
                        ..
                    })
                ),
                "{delta}"
            );
        }
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                PhiSegment::new(bad, 1.0),
                Err(GeometryError::NonFinite {
                    field: "start_phi",
                    ..
                })
            ));
            assert!(matches!(
                PhiSegment::new(0.0, bad),
                Err(GeometryError::NonFinite {
                    field: "delta_phi",
                    ..
                })
            ));
        }
    }
}
