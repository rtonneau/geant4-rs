//! Spherical shell section, a port of Geant4 11.4.1 `G4Sphere`
//! (`source/geometry/solids/CSG/src/G4Sphere.cc`, `include/G4Sphere.icc`,
//! `include/G4Sphere.hh`).
//!
//! The solid is bounded by the spheres `r = rmin` and `r = rmax` about its centre, the two
//! half-planes of its phi range unless that is a full circle, and the cones of its polar
//! (theta) range, theta being measured from +z: `theta = start` unless the start is 0 and
//! `theta = end` unless the end is pi. A cone at theta = pi/2 is the plane z = 0. With
//! `rmin = 0`, a full phi circle and the full theta range it is a full sphere.
//!
//! # Tolerances
//!
//! The radial shells are `fRmaxTolerance = max(kRadTolerance, 2e-11 rmax)` and
//! `fRminTolerance = max(kRadTolerance, 2e-11 rmin)` thick (`G4Sphere.cc:104-105`, `fEpsilon`
//! in `G4Sphere.hh:267`), so they grow with the radius beyond 5e7 nm (5 cm): 0.02 nm for a
//! 1 m sphere. `Inside` tests phi and theta against [`HALF_ANG_TOLERANCE`], an angle.
//! `DistanceToOut` tests the phi planes against [`HALF_CAR_TOLERANCE`] and the theta cones
//! against `h = fRmaxTolerance/2`, both lengths. The angular shell is therefore
//! `rho * 5e-10` nm thick on each side of a phi plane at a distance `rho` from the z axis,
//! and `r * 5e-10` nm on each side of a theta cone at a distance `r` from the centre: thinner
//! than the Cartesian shell within 1e6 nm of the axis (phi) or of the centre (theta), wider
//! beyond. These Geant4 properties are kept:
//!
//! - An exit point that `distance_to_out` puts on a phi plane within a few nm of the z axis,
//!   or on a theta cone within a few nm of the centre, may classify as Outside: its rounding
//!   exceeds the angular shell there (as for `G4Tubs`, see `tubs.rs`).
//! - A start that is Inside by angle but on a boundary in the Cartesian sense can get a zero
//!   distance. For a phi plane that means within `kCarTolerance/2` of it (the `G4Tubs` phi
//!   block, deviation 1). For a theta cone, Geant4's on-cone test uses the horizontal
//!   distance `rho - z tan(theta)` (`G4Sphere.cc:1886`, 2023), which is the true distance
//!   divided by `|cos(theta)|`: it covers starts within `h |cos(theta)|` of the cone, and
//!   deviation 7 extends this to `h` for a start whose ray leaves through the cone. The z
//!   axis of a phi segment is also such a boundary: `Inside` skips the phi test there
//!   (`G4Sphere.cc:308`), so such a point is Inside, yet leaves at once if its direction is
//!   outside the phi range.
//! - Beyond 1e6 nm from the axis or the centre, a Surface start can lie farther than the
//!   Cartesian tolerance outside the plane or cone it is on, and is then not recognised as
//!   leaving through it. For a phi plane this comes from the `G4Tubs` phi block (deviation
//!   1); `G4Sphere`'s own block would return 0. For a theta cone it is `G4Sphere`'s own
//!   behaviour, kept by deviation 7 whenever the near root lies more than `h` behind the
//!   start along the ray: a Surface start beyond `h`, or within `h` but moving out so
//!   obliquely that `delta / cos(alpha) > h` (possible only for r > 1e6 |cos(theta)| nm).
//! - `DistanceToOut` tests the rmax shell in the linearised form
//!   `r^2 - rmax^2 > -fRmaxTolerance*rmax` (`G4Sphere.cc:1786`). In the innermost
//!   `fRmaxTolerance^2/(8 rmax)` of the shell (1.25e-7 nm at rmax = 1 nm, thinner for larger
//!   spheres), a point that is Inside then gets 0 when moving outward.
//! - A ray that touches rmin, or passes through the apex of a theta cone (the centre), stops
//!   there: these are boundary points (`G4Sphere.cc:1820-1829`, 1955-1988).
//!
//! # Deviations from `G4Sphere::DistanceToOut(p, v)`
//!
//! Each fixes a case where Geant4 returns a distance that is wrong by more than the
//! tolerance, infinite or negative. A test in this file covers each one.
//!
//! 1. **Phi planes.** The phi block of `G4Tubs` ([`PhiSegment::exit_distance`],
//!    `G4Tubs.cc:1320-1452`) replaces the older one of `G4Sphere` (`G4Sphere.cc:2149-2462`).
//!    Away from the tolerance shell both give the same distance, but `G4Sphere` (a) compares
//!    the `atan2` angle of the direction, in `[-pi, pi]`, with the segment range
//!    (`G4Sphere.cc:2178-2181`, 2213-2216, 2441-2442) without first moving it into that
//!    range, as `G4Tubs.cc:1322-1328` does: for a segment such as `[pi/2, 2 pi]`, a start on
//!    the z axis, or a ray through it, with its direction at `-pi/4` inside the segment is
//!    stopped on the axis. It also (b) picks its cases from the exact signs of `pDistS` and
//!    `pDistE` (`G4Sphere.cc:2164`, 2238, 2262): for an opening above pi, a start on a plane
//!    whose rounded position falls just outside it can fail to leave.
//!    The cost, within the tolerance: the `G4Tubs` block ignores a plane where `G4Sphere`
//!    returns 0 for a start outside it moving out, in two cases. A start rounded just
//!    outside a plane, moving out so obliquely that `|comp| < pDist/(kCarTolerance/2)`, has
//!    its crossing more than `kCarTolerance/2` behind it (`G4Tubs.cc:1356`, 1402). Beyond
//!    1e6 nm from the axis, a Surface start can lie more than `kCarTolerance/2` outside a
//!    plane (`G4Tubs.cc:1345-1348`). Such a start goes on outside the segment.
//! 2. **Radial shells.** Every start that `Inside` accepts is tested against rmax.
//!    `G4Sphere` skips that test in the outermost `fRmaxTolerance^2/(8 rmax)` of the shell,
//!    where its linearised `c < fRmaxTolerance*fRmax` fails (`G4Sphere.cc:1773`), and within
//!    `halfRminTolerance - rmin` of the centre when `rmin < halfRminTolerance`, because its
//!    `Rmin_minus` goes negative (`G4Sphere.cc:1727`, 1769). The step is then `kInfinity`
//!    (Geant4 warns about it, `G4Sphere.cc:2578-2600`).
//! 3. **Convex start cone (start > pi/2).** `G4Sphere.cc:1963` tests `std::fabs(s)`, where
//!    `s` is not declared in the function and resolves to the unit `CLHEP::s` (1e9), so that
//!    test is always false. The port tests `sd`, as the matching end-cone test does
//!    (`G4Sphere.cc:2106`). With the typo, a start exactly on that cone moving inward, whose
//!    first root is exactly 0, ignores the cone.
//! 4. **End cone.** The clamp `|d2| < halfRmaxTolerance => d2 = 0` (`G4Sphere.cc:2094`) is
//!    dropped. It compares an area with a length, so its effect depends on the unit system:
//!    in Geant4's mm it merges roots up to `2 sqrt(5e-10) mm = 45 nm` apart, and the end cone
//!    of a nm-sized sphere is missed. The start cone has no such clamp.
//! 5. **Roots behind the start or on the other nappe.** For a ray parallel to a cone
//!    (`|t1| < kAngTolerance/2`, `G4Sphere.cc:1888-1922`, 2025-2060) Geant4 takes the root
//!    of the linear equation without checking that it lies ahead or on the cone's own
//!    nappe. That can give a negative distance (`-inf` for `t2 = 0`), or stop the ray early
//!    where it crosses the other nappe, which is not a boundary. The port accepts it only
//!    under the tests of the quadratic roots of the same cone. For the z = 0 plane
//!    (`G4Sphere.cc:1863-1879`, 1995-2016) a root behind the start (a Surface start beyond
//!    the Cartesian tolerance, farther than 1e6 nm from the z axis) is negative in Geant4
//!    and 0 here.
//! 6. **Start at the apex (the centre, `rmin = 0`).** There `t2 = 0`, so the cone tests
//!    ignore the direction: a convex cone always returns 0 and a concave one never limits
//!    the step, even for a direction outside the theta range. The port compares the polar
//!    angle of the direction with `[start - kAngTolerance/2, end + kAngTolerance/2]`, as
//!    `G4Sphere` does for the phi range on the z axis (`G4Sphere.cc:2439-2451`).
//! 7. **Starts within `h` of a cone.** Geant4's on-cone early return uses the horizontal
//!    distance `|rho - z tan(theta)| < h` (`G4Sphere.cc:1926`, 2064), true distance
//!    `< h |cos(theta)|`, while it accepts only roots beyond `h` (1968, 1983, 2111, 2130). A
//!    start whose true distance lies in `(h |cos(theta)|, h]`, moving out, has its exit root
//!    within `h`: the root is rejected and the cone ignored, so the step runs on through the
//!    removed region (for theta in `[pi/4, pi]`, rmax 10 nm, a start at r = 5 nm 4.5e-4 nm
//!    inside the start cone and moving along `-theta_hat` gets 8.66 nm instead of 0). The
//!    port returns 0 when a root on the cone's own nappe lies within `h` of the start,
//!    behind or ahead, and the direction leaves the solid by the sign test of the early
//!    return. That sign test keeps a Surface start just outside the cone moving in from
//!    getting 0.

use super::phi_segment::PhiSegment;
use super::{
    ANG_TOLERANCE, GeometryError, HALF_ANG_TOLERANCE, Inside, RAD_TOLERANCE, check_finite,
};
use crate::vec3::Vec3;
use std::f64::consts::{FRAC_PI_2, PI, TAU};

/// `fEpsilon` (`G4Sphere.hh:267`): the radial tolerance is at least this fraction of the
/// radius.
const RADIAL_EPSILON: f64 = 2.0e-11;

/// Smallest accepted outer radius, `1.1*kRadTolerance` (`G4Sphere.cc:95`), nm.
const MIN_RMAX: f64 = 1.1 * RAD_TOLERANCE;

/// A theta cone with `|tan theta|` above `5/kAngTolerance` is the plane z = 0
/// (`G4Sphere.cc:1863`, 1995).
const PLANE_TAN: f64 = 5.0 / ANG_TOLERANCE;

/// Spherical shell section about its centre: radii `rmin < rmax` (nm), a phi range about the
/// z axis and a theta range from +z (rad).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereSolid {
    centre: Vec3,
    rmin: f64,
    rmax: f64,
    /// `fRminTolerance`: 0 for `rmin = 0`.
    rmin_tol: f64,
    /// `fRmaxTolerance`.
    rmax_tol: f64,
    phi: PhiSegment,
    theta: ThetaSegment,
}

impl SphereSolid {
    /// Checks the parameters as `G4Sphere::G4Sphere` does (`G4Sphere.cc:95-110`):
    /// `0 <= rmin < rmax`, `rmax >= 1.1*kRadTolerance`, the phi range of
    /// [`PhiSegment::new`] and the theta range of [`ThetaSegment::new`]. Non-finite values
    /// are also rejected, which Geant4 does not check.
    pub(super) fn new(
        centre: Vec3,
        rmin: f64,
        rmax: f64,
        start_phi: f64,
        delta_phi: f64,
        start_theta: f64,
        delta_theta: f64,
    ) -> Result<Self, GeometryError> {
        check_finite("centre", centre)?;
        for (field, value) in [("rmin", rmin), ("rmax", rmax)] {
            if !value.is_finite() {
                return Err(GeometryError::NonFinite { field, value });
            }
        }
        if rmin >= rmax || rmin < 0.0 {
            return Err(GeometryError::InvalidRadii { rmin, rmax });
        }
        if rmax < MIN_RMAX {
            return Err(GeometryError::TooSmall {
                field: "rmax",
                value: rmax,
                min: MIN_RMAX,
            });
        }
        let phi = PhiSegment::new(start_phi, delta_phi)?;
        let theta = ThetaSegment::new(start_theta, delta_theta)?;
        // G4Sphere.cc:104-105.
        let rmin_tol = if rmin != 0.0 {
            RAD_TOLERANCE.max(RADIAL_EPSILON * rmin)
        } else {
            0.0
        };
        let rmax_tol = RAD_TOLERANCE.max(RADIAL_EPSILON * rmax);
        Ok(Self {
            centre,
            rmin,
            rmax,
            rmin_tol,
            rmax_tol,
            phi,
            theta,
        })
    }

    /// Centre, nm.
    pub fn centre(&self) -> Vec3 {
        self.centre
    }

    /// Inner radius, nm (0 for a solid sphere).
    pub fn rmin(&self) -> f64 {
        self.rmin
    }

    /// Outer radius, nm.
    pub fn rmax(&self) -> f64 {
        self.rmax
    }

    /// Start of the phi range after Geant4 normalisation (`G4Sphere::GetStartPhiAngle`),
    /// rad: in `[0, 2 pi)`, in `(-2 pi, 0)` for a segment crossing phi = 0, and 0 for a full
    /// circle.
    pub fn start_phi(&self) -> f64 {
        self.phi.start()
    }

    /// Phi opening after Geant4 normalisation (`G4Sphere::GetDeltaPhiAngle`), rad: exactly
    /// `2 pi` for a full circle.
    pub fn delta_phi(&self) -> f64 {
        self.phi.delta()
    }

    /// Start of the theta range (`G4Sphere::GetStartThetaAngle`), rad, in `[0, pi)`.
    pub fn start_theta(&self) -> f64 {
        self.theta.start
    }

    /// Theta opening after Geant4 normalisation (`G4Sphere::GetDeltaThetaAngle`), rad: at
    /// most `pi - start_theta`.
    pub fn delta_theta(&self) -> f64 {
        self.theta.delta
    }

    /// The `start_phi` given to the constructor, rad.
    pub(super) fn requested_start_phi(&self) -> f64 {
        self.phi.requested_start()
    }

    /// The `delta_phi` given to the constructor, rad.
    pub(super) fn requested_delta_phi(&self) -> f64 {
        self.phi.requested_delta()
    }

    /// The `start_theta` given to the constructor, rad.
    pub(super) fn requested_start_theta(&self) -> f64 {
        self.theta.requested_start
    }

    /// The `delta_theta` given to the constructor, rad.
    pub(super) fn requested_delta_theta(&self) -> f64 {
        self.theta.requested_delta
    }

    /// `G4Sphere::Inside` (`G4Sphere.cc:255-359`) for a point `q` relative to the centre.
    ///
    /// The centre itself is Outside with an inner radius, Surface for a phi or theta
    /// segment and Inside otherwise (`G4Sphere.cc:274-286`). Elsewhere the radial shells
    /// give Inside, Surface or Outside (`288-304`), and the phi (`308-323`) and theta
    /// (`327-357`) tests can only lower that class, with the angular tolerance. The phi test
    /// is skipped on the z axis, so the z axis of a phi segment is Inside.
    pub(super) fn inside_local(&self, q: Vec3) -> Inside {
        let rho2 = q.x * q.x + q.y * q.y;
        let rad2 = rho2 + q.z * q.z;
        if rad2 == 0.0 {
            return if self.rmin > 0.0 {
                Inside::Outside
            } else if !self.phi.is_full() || !self.theta.full {
                Inside::Surface
            } else {
                Inside::Inside
            };
        }

        let half_rmax_tol = 0.5 * self.rmax_tol;
        let half_rmin_tol = 0.5 * self.rmin_tol;
        let rmax_minus = self.rmax - half_rmax_tol;
        let rmin_plus = if self.rmin > 0.0 {
            self.rmin + half_rmin_tol
        } else {
            0.0
        };
        let mut class = if rad2 <= rmax_minus * rmax_minus && rad2 >= rmin_plus * rmin_plus {
            Inside::Inside
        } else {
            let tol_rmax = self.rmax + half_rmax_tol;
            let tol_rmin = (self.rmin - half_rmin_tol).max(0.0);
            if rad2 <= tol_rmax * tol_rmax && rad2 >= tol_rmin * tol_rmin {
                Inside::Surface
            } else {
                return Inside::Outside;
            }
        };

        if !self.phi.is_full() && rho2 != 0.0 {
            class = phi_class(&self.phi, q.x, q.y, class);
            if class == Inside::Outside {
                return class;
            }
        }
        // G4Sphere.cc:327 also requires rho2 != 0 or z != 0, which holds off the centre.
        if !self.theta.full {
            class = self.theta.classify(rho2.sqrt().atan2(q.z), class);
        }
        class
    }
    /// `G4Sphere::DistanceToOut(p)` (`G4Sphere.cc:2609-2694`): the smallest of the distances
    /// to the radial shells, the nearer phi plane (0 on the z axis, `2655-2662`) and the theta
    /// cones (`rds * sin(min(dTheta1, dTheta2))`, 0 at the centre, `2671-2690`), clamped at 0.
    pub(super) fn safety_local(&self, p: Vec3) -> f64 {
        let rho2 = p.x * p.x + p.y * p.y;
        let rds = (rho2 + p.z * p.z).sqrt();
        let rho = rho2.sqrt();
        let mut safe = self.rmax - rds;
        if self.rmin != 0.0 {
            safe = safe.min(rds - self.rmin);
        }
        if !self.phi.is_full() {
            let safe_phi = if rho > 0.0 {
                self.phi.safety(p.x, p.y)
            } else {
                0.0
            };
            safe = safe.min(safe_phi);
        }
        if !self.theta.full {
            let safe_theta = if rds > 0.0 {
                let p_theta = (p.z / rds).acos();
                let d1 = if self.theta.start > 0.0 {
                    p_theta - self.theta.start
                } else {
                    f64::INFINITY
                };
                let d2 = if self.theta.end < PI {
                    self.theta.end - p_theta
                } else {
                    f64::INFINITY
                };
                rds * d1.min(d2).sin()
            } else {
                0.0
            };
            safe = safe.min(safe_theta);
        }
        if safe < 0.0 { 0.0 } else { safe }
    }

    /// `G4Sphere::DistanceToOut(p, v)` without normal (`G4Sphere.cc:1714-2603`) for a point
    /// `p` relative to the centre, which is not outside, with the deviations listed in the
    /// module documentation.
    ///
    /// The candidates are the rmax far root or, before it, the rmin near root
    /// (`G4Sphere.cc:1769-1832`), the theta cones (`1836-2145`) and the phi planes; the
    /// smallest wins (`2457-2467`). A start on a tolerant surface and moving out through it
    /// returns 0. Unlike `G4Tubs`, small distances are not rounded down to 0.
    pub(super) fn distance_to_out_local(&self, p: Vec3, v: Vec3) -> f64 {
        let (rmin, rmax) = (self.rmin, self.rmax);
        let rho2 = p.x * p.x + p.y * p.y;
        let rad2 = rho2 + p.z * p.z;
        let pdotv2d = p.x * v.x + p.y * v.y;
        let pdotv3d = pdotv2d + p.z * v.z;

        // Outer sphere, G4Sphere.cc:1769-1800, over the whole tolerant shell (deviation 2).
        let c = rad2 - rmax * rmax;
        let d2 = pdotv3d * pdotv3d - c;
        if c > -self.rmax_tol * rmax && (pdotv3d >= 0.0 || d2 < 0.0) {
            // On the tolerant rmax surface, heading out or not re-entering.
            return 0.0;
        }
        // Far root, since inside rmax: d2 >= 0 here.
        let mut snxt = -pdotv3d + d2.sqrt();

        // Inner sphere, G4Sphere.cc:1806-1831: its first root comes before rmax.
        if rmin != 0.0 {
            let c = rad2 - rmin * rmin;
            let d2 = pdotv3d * pdotv3d - c;
            let tol = self.rmin_tol * rmin;
            if c > -tol {
                if c < tol && d2 >= tol && pdotv3d < 0.0 {
                    // On the tolerant rmin surface, heading into the hole.
                    return 0.0;
                }
                if d2 >= 0.0 {
                    let sd = -pdotv3d - d2.sqrt();
                    if sd >= 0.0 {
                        snxt = sd;
                    }
                }
            }
        }

        let stheta = if self.theta.full {
            f64::INFINITY
        } else if rad2 == 0.0 {
            self.theta.apex_exit(v) // deviation 6
        } else {
            let half_rmax_tol = 0.5 * self.rmax_tol;
            self.theta.exit_distance(p, v, rho2, pdotv2d, half_rmax_tol)
        };
        // Deviation 1; the clamp removes the -kCarTolerance/2 the on-axis crossing allows.
        let sphi = if self.phi.is_full() {
            f64::INFINITY
        } else {
            self.phi.exit_distance(p, v).max(0.0)
        };
        if sphi < snxt {
            snxt = sphi;
        }
        if stheta < snxt {
            snxt = stheta;
        }
        snxt
    }
}

/// Phi test of `G4Sphere::Inside` (`G4Sphere.cc:308-323`) for `(x, y)` off the z axis:
/// Outside beyond the outer angular limits, Surface for an Inside point between the outer
/// and inner limits, otherwise `class` unchanged. `atan2` gives phi in `[-pi, pi]`, moved by
/// `2 pi` towards the segment.
fn phi_class(seg: &PhiSegment, x: f64, y: f64, class: Inside) -> Inside {
    let h = HALF_ANG_TOLERANCE;
    let (start, end) = (seg.start(), seg.end());
    let mut p_phi = y.atan2(x);
    if p_phi < start - h {
        p_phi += TAU;
    } else if p_phi > end + h {
        p_phi -= TAU;
    }
    if p_phi < start - h || p_phi > end + h {
        Inside::Outside
    } else if class == Inside::Inside && (p_phi < start + h || p_phi > end - h) {
        Inside::Surface
    } else {
        class
    }
}

/// Polar (theta) range `[start, end]` measured from +z, rad, after Geant4 normalisation, with
/// the tangents of its cones.
///
/// The requested values are kept as given, so that serialisation writes back what the user
/// wrote; the normalised values drive the algorithms.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ThetaSegment {
    requested_start: f64,
    requested_delta: f64,
    /// `fSTheta`, in `[0, pi)`; 0 means no start cone.
    start: f64,
    /// `fDTheta`, in `(0, pi - start]`.
    delta: f64,
    /// `eTheta = start + delta`, in `(start, pi]`; pi means no end cone.
    end: f64,
    /// `fFullThetaSphere`.
    full: bool,
    /// `tanSTheta` and its square.
    tan_start: f64,
    tan_start2: f64,
    /// `tanETheta` and its square.
    tan_end: f64,
    tan_end2: f64,
}

impl ThetaSegment {
    /// Checks and normalises `(start_theta, delta_theta)` as `G4Sphere::CheckThetaAngles`
    /// (`G4Sphere.icc:155-190`) does, then computes the tangents as
    /// `InitializeThetaTrigonometry` (`G4Sphere.icc:138-152`) does.
    ///
    /// - `start_theta` must lie in `[0, pi)`. Geant4 also accepts `pi`, which gives an
    ///   opening of 0 (an empty solid); it is rejected here.
    /// - `delta_theta` must lie in `(0, pi]`. If `start + delta >= pi` the opening is reduced
    ///   to `pi - start`, as in Geant4; the end angle `start + (pi - start)` is then exactly
    ///   pi in double precision. Geant4 silently reduces a `delta_theta` above pi too; it is
    ///   rejected here, as `delta_phi > 2 pi` is.
    /// - The range is full only for `start = 0` and an opening of pi (`G4Sphere.icc:186`
    ///   tests `fDTheta - fSTheta >= pi`, which only that case meets).
    ///
    /// Non-finite values are rejected, which Geant4 does not check.
    fn new(start_theta: f64, delta_theta: f64) -> Result<Self, GeometryError> {
        for (field, value) in [("start_theta", start_theta), ("delta_theta", delta_theta)] {
            if !value.is_finite() {
                return Err(GeometryError::NonFinite { field, value });
            }
        }
        if !(0.0..PI).contains(&start_theta) {
            return Err(GeometryError::AngleOutOfRange {
                field: "start_theta",
                value: start_theta,
                range: "[0, pi)",
            });
        }
        if delta_theta <= 0.0 || delta_theta > PI {
            return Err(GeometryError::AngleOutOfRange {
                field: "delta_theta",
                value: delta_theta,
                range: "(0, pi]",
            });
        }
        let start = start_theta;
        let delta = if delta_theta + start >= PI {
            PI - start
        } else {
            delta_theta
        };
        let end = start + delta;
        let tan = |angle: f64| {
            let (sin, cos) = angle.sin_cos();
            sin / cos
        };
        let (tan_start, tan_end) = (tan(start), tan(end));
        Ok(Self {
            requested_start: start_theta,
            requested_delta: delta_theta,
            start,
            delta,
            end,
            full: delta - start >= PI,
            tan_start,
            tan_start2: tan_start * tan_start,
            tan_end,
            tan_end2: tan_end * tan_end,
        })
    }

    /// Theta test of `G4Sphere::Inside` (`G4Sphere.cc:332-356`) for a point at polar angle
    /// `p_theta` already classified `class` by the radial and phi tests.
    fn classify(&self, p_theta: f64, class: Inside) -> Inside {
        let h = HALF_ANG_TOLERANCE;
        let has_start = self.start > 0.0;
        let has_end = self.end < PI;
        if class == Inside::Inside {
            if (has_start && p_theta < self.start + h) || (has_end && p_theta > self.end - h) {
                let past_start = !has_start || p_theta >= self.start - h;
                let before_end = !has_end || p_theta <= self.end + h;
                if past_start && before_end {
                    Inside::Surface
                } else {
                    Inside::Outside
                }
            } else {
                Inside::Inside
            }
        } else if (has_start && p_theta < self.start - h) || (has_end && p_theta > self.end + h) {
            Inside::Outside
        } else {
            class
        }
    }

    /// Theta candidate of `G4Sphere::DistanceToOut(p, v)` (`G4Sphere.cc:1836-2145`) for a
    /// start `p` off the centre: 0 to leave at once, `f64::INFINITY` if the cones do not
    /// limit the step. `rho2 = p.x^2 + p.y^2`, `pdotv2d = p.x v.x + p.y v.y`, and `h` is
    /// `halfRmaxTolerance`.
    fn exit_distance(&self, p: Vec3, v: Vec3, rho2: f64, pdotv2d: f64, h: f64) -> f64 {
        let start = if self.start != 0.0 {
            self.start_cone_exit(p, v, rho2, pdotv2d, h)
        } else {
            f64::INFINITY
        };
        let end = if self.end < PI {
            self.end_cone_exit(p, v, rho2, pdotv2d, h)
        } else {
            f64::INFINITY
        };
        // The end cone replaces a farther start-cone candidate (G4Sphere.cc:2010, 2055, 2113,
        // 2133); a 0 from either is Geant4's early return.
        if end < start { end } else { start }
    }

    /// The cone `theta = start` (`G4Sphere.cc:1861-1992`). It is convex seen from the solid
    /// for `start > pi/2` (the solid lies inside its lower nappe), concave below.
    fn start_cone_exit(&self, p: Vec3, v: Vec3, rho2: f64, pdotv2d: f64, h: f64) -> f64 {
        if self.tan_start.abs() > PLANE_TAN {
            // The plane z = 0, solid below (G4Sphere.cc:1863-1879); deviation 5.
            return if v.z > 0.0 {
                if p.z.abs() <= h {
                    0.0
                } else {
                    (-p.z / v.z).max(0.0)
                }
            } else {
                f64::INFINITY
            };
        }
        let tan2 = self.tan_start2;
        let t1 = 1.0 - v.z * v.z * (1.0 + tan2);
        let t2 = pdotv2d - p.z * v.z * tan2; // ~ v . normal for p on the cone
        let dist2 = rho2 - p.z * p.z * tan2;
        let dist_theta = rho2.sqrt() - p.z * self.tan_start;
        let convex = self.start > FRAC_PI_2;
        let z_at = |sd: f64| p.z + sd * v.z;
        // The cone's own nappe, G4Sphere.cc:1968 and 1983.
        let on_nappe = |sd: f64| {
            if convex {
                z_at(sd) <= 0.0
            } else {
                z_at(sd) >= 0.0
            }
        };
        // Moving out of the solid: the sign test of the on-cone early returns (1928, 1945).
        let leaving = if convex { t2 >= 0.0 } else { t2 < 0.0 };
        // Deviation 7: the ray meets the cone within h of the start and moves out.
        let at_start = |sd: f64| sd.abs() <= h && leaving && on_nappe(sd);
        let accept = |sd: f64| {
            if sd > h && on_nappe(sd) {
                sd
            } else {
                f64::INFINITY
            }
        };

        if t1.abs() < HALF_ANG_TOLERANCE {
            // v parallel to the cone: linear equation (G4Sphere.cc:1888-1922).
            if v.z <= 0.0 {
                return f64::INFINITY;
            }
            if dist_theta.abs() < h && ((!convex && p.z > 0.0) || (convex && p.z <= 0.0)) {
                return 0.0;
            }
            let sd = -0.5 * dist2 / t2;
            // Deviation 5: Geant4 takes the root unchecked.
            return if at_start(sd) { 0.0 } else { accept(sd) };
        }

        // Second-order equation, G4Sphere.cc:1924-1990.
        if dist_theta.abs() < h && ((convex && t2 >= 0.0) || (!convex && t2 < 0.0 && p.z >= 0.0)) {
            return 0.0; // on the cone, leaving
        }
        let b = t2 / t1;
        let c = dist2 / t1;
        let d2 = b * b - c;
        if d2 < 0.0 {
            return f64::INFINITY;
        }
        let d = d2.sqrt();
        let (first, second) = (-b - d, -b + d);
        if at_start(first) || at_start(second) {
            return 0.0; // deviation 7
        }
        // Take the second root if the first is the start itself (moving in), behind, or on
        // the other nappe. Deviation 3: Geant4 tests |s| (CLHEP::s) in the convex case.
        let skip_first = if convex {
            (first.abs() < h && t2 < 0.0) || first < 0.0 || (first > 0.0 && z_at(first) > 0.0)
        } else {
            (first.abs() < h && t2 >= 0.0) || first < 0.0 || (first > 0.0 && z_at(first) < 0.0)
        };
        accept(if skip_first { second } else { first })
    }

    /// The cone `theta = end` (`G4Sphere.cc:1993-2143`). It is convex seen from the solid
    /// for `end < pi/2` (the solid lies inside its upper nappe), concave above.
    fn end_cone_exit(&self, p: Vec3, v: Vec3, rho2: f64, pdotv2d: f64, h: f64) -> f64 {
        if self.tan_end.abs() > PLANE_TAN {
            // The plane z = 0, solid above (G4Sphere.cc:1995-2016); deviation 5.
            return if v.z < 0.0 {
                if p.z.abs() <= h {
                    0.0
                } else {
                    (-p.z / v.z).max(0.0)
                }
            } else {
                f64::INFINITY
            };
        }
        let tan2 = self.tan_end2;
        let t1 = 1.0 - v.z * v.z * (1.0 + tan2);
        let t2 = pdotv2d - p.z * v.z * tan2; // ~ v . normal for p on the cone
        let dist2 = rho2 - p.z * p.z * tan2;
        let dist_theta = rho2.sqrt() - p.z * self.tan_end;
        let convex = self.end < FRAC_PI_2;
        let z_at = |sd: f64| p.z + sd * v.z;
        // The cone's own nappe, G4Sphere.cc:2111 and 2130-2131 (not tested for the convex
        // cone).
        let on_nappe = |sd: f64| convex || z_at(sd) <= h;
        // Moving out of the solid: the sign test of the on-cone early returns (2066, 2085).
        let leaving = if convex { t2 >= 0.0 } else { t2 < 0.0 };
        // Deviation 7: the ray meets the cone within h of the start and moves out.
        let at_start = |sd: f64| sd.abs() <= h && leaving && on_nappe(sd);
        let accept = |sd: f64| {
            if sd > h && on_nappe(sd) {
                sd
            } else {
                f64::INFINITY
            }
        };

        if t1.abs() < HALF_ANG_TOLERANCE {
            // v parallel to the cone: linear equation (G4Sphere.cc:2025-2060).
            if v.z >= 0.0 {
                return f64::INFINITY;
            }
            if dist_theta.abs() < h && ((!convex && p.z < 0.0) || (convex && p.z >= 0.0)) {
                return 0.0;
            }
            let sd = -0.5 * dist2 / t2;
            // Deviation 5: Geant4 takes the root unchecked.
            return if at_start(sd) { 0.0 } else { accept(sd) };
        }

        // Second-order equation, G4Sphere.cc:2062-2141, without the d2 clamp of 2094
        // (deviation 4).
        if dist_theta.abs() < h && ((convex && t2 >= 0.0) || (!convex && t2 < 0.0 && p.z <= 0.0)) {
            return 0.0; // on the cone, leaving
        }
        let b = t2 / t1;
        let c = dist2 / t1;
        let d2 = b * b - c;
        if d2 < 0.0 {
            return f64::INFINITY;
        }
        let d = d2.sqrt();
        let (first, second) = (-b - d, -b + d);
        if at_start(first) || at_start(second) {
            return 0.0; // deviation 7
        }
        // Take the second root if the first is the start itself (moving in), behind, or (for
        // the concave cone) on the upper nappe.
        let skip_first = if convex {
            (first.abs() < h && t2 < 0.0) || first < 0.0
        } else {
            (first.abs() < h && t2 >= 0.0) || first < 0.0 || (first > 0.0 && z_at(first) > h)
        };
        accept(if skip_first { second } else { first })
    }

    /// Theta candidate for a start at the apex of the cones, the centre (deviation 6): 0 if
    /// the polar angle of `v` is outside the tolerant theta range, else no limit.
    fn apex_exit(&self, v: Vec3) -> f64 {
        let v_theta = v.x.hypot(v.y).atan2(v.z);
        let h = HALF_ANG_TOLERANCE;
        if self.start - h <= v_theta && v_theta <= self.end + h {
            f64::INFINITY
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{centre, check_exit_invariants, size, unit_direction};
    use super::super::{
        CAR_TOLERANCE, Direction, GeometryError, HALF_ANG_TOLERANCE, HALF_CAR_TOLERANCE,
        HALF_RAD_TOLERANCE, Inside, RAD_TOLERANCE, Volume,
    };
    use super::SphereSolid;
    use crate::vec3::Vec3;
    use approx::assert_relative_eq;
    use proptest::prelude::*;
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

    const HRT: f64 = HALF_RAD_TOLERANCE;
    const HAT: f64 = HALF_ANG_TOLERANCE;

    fn v3(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z)
    }

    fn dir(x: f64, y: f64, z: f64) -> Direction {
        Direction::new(v3(x, y, z)).unwrap()
    }

    fn sphere(rmin: f64, rmax: f64, sphi: f64, dphi: f64, stheta: f64, dtheta: f64) -> Volume {
        Volume::sphere(Vec3::ZERO, rmin, rmax, sphi, dphi, stheta, dtheta).unwrap()
    }

    /// Full sphere, radius 10 nm.
    fn ball() -> Volume {
        sphere(0.0, 10.0, 0.0, TAU, 0.0, PI)
    }

    /// Spherical shell, radii 4 and 10 nm.
    fn shell() -> Volume {
        sphere(4.0, 10.0, 0.0, TAU, 0.0, PI)
    }

    /// Radius 10 nm, theta in `[start, start + delta]`, full phi.
    fn theta_section(start: f64, delta: f64) -> Volume {
        sphere(0.0, 10.0, 0.0, TAU, start, delta)
    }

    /// Radius 10 nm, phi in `[start, start + delta]`, full theta.
    fn phi_section(start: f64, delta: f64) -> Volume {
        sphere(0.0, 10.0, start, delta, 0.0, PI)
    }

    fn solid(volume: &Volume) -> &SphereSolid {
        match volume {
            Volume::Sphere(s) => s,
            other => unreachable!("not a sphere: {other:?}"),
        }
    }

    fn d(volume: &Volume, p: Vec3, v: Direction) -> f64 {
        volume.distance_to_out(p, v).unwrap()
    }

    /// Point at radius `r`, polar angle `theta` and azimuth `phi` from the volume's centre.
    fn point_at(volume: &Volume, r: f64, theta: f64, phi: f64) -> Vec3 {
        let (st, ct) = theta.sin_cos();
        let (sp, cp) = phi.sin_cos();
        volume.centre() + v3(r * st * cp, r * st * sp, r * ct)
    }

    // --- construction ---

    #[test]
    fn accessors_return_constructor_values() {
        let s = Volume::sphere(v3(1.0, 2.0, 3.0), 4.0, 5.0, 0.5, 1.0, 0.25, 2.0).unwrap();
        assert_eq!(s.centre(), v3(1.0, 2.0, 3.0));
        let t = solid(&s);
        assert_eq!(
            (
                t.rmin(),
                t.rmax(),
                t.start_phi(),
                t.delta_phi(),
                t.start_theta(),
                t.delta_theta()
            ),
            (4.0, 5.0, 0.5, 1.0, 0.25, 2.0)
        );
    }

    #[test]
    fn theta_range_is_normalised_as_in_geant4() {
        // (start, delta) -> (delta after the clamp, end, full)
        let cases = [
            (0.0, PI, PI, PI, true),
            (0.0, 1.0, 1.0, 1.0, false),
            (1.0, 1.0, 1.0, 2.0, false),
            (0.5, PI, PI - 0.5, PI, false),
            (0.5, 3.0, PI - 0.5, PI, false),
            (FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, PI, false),
            (0.0, FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, false),
        ];
        for (start, delta, delta_out, end, full) in cases {
            let t = solid(&theta_section(start, delta)).theta;
            assert_eq!(
                (t.start, t.delta, t.end, t.full),
                (start, delta_out, end, full),
                "{start} {delta}"
            );
            assert_eq!(t.requested_delta, delta);
        }
        // A range reduced to end at pi ends exactly at pi, so there is no end cone.
        for i in 1..2000 {
            let start = PI * f64::from(i) / 2000.0;
            let t = solid(&theta_section(start, PI)).theta;
            assert_eq!(t.end, PI, "start {start}");
        }
    }

    #[test]
    fn radial_tolerance_scales_beyond_five_centimetres() {
        let small = *solid(&shell());
        assert_eq!(
            (small.rmin_tol, small.rmax_tol),
            (RAD_TOLERANCE, RAD_TOLERANCE)
        );
        assert_eq!(solid(&ball()).rmin_tol, 0.0);
        // 2e-11 * 5e7 nm = kRadTolerance.
        let edge = solid(&sphere(0.0, 5.0e7, 0.0, TAU, 0.0, PI)).rmax_tol;
        assert_relative_eq!(edge, RAD_TOLERANCE, max_relative = 1e-12);
        let metre = *solid(&sphere(2.0e8, 1.0e9, 0.0, TAU, 0.0, PI));
        assert_relative_eq!(metre.rmax_tol, 0.02, max_relative = 1e-12);
        assert_relative_eq!(metre.rmin_tol, 0.004, max_relative = 1e-12);
    }

    #[test]
    fn rejects_invalid_radii() {
        for (rmin, rmax) in [(2.0, 1.0), (1.0, 1.0), (-0.5, 1.0), (0.0, 0.0), (0.0, -1.0)] {
            let err = Volume::sphere(Vec3::ZERO, rmin, rmax, 0.0, TAU, 0.0, PI).unwrap_err();
            assert_eq!(err, GeometryError::InvalidRadii { rmin, rmax });
        }
        // rmax must be at least 1.1 kRadTolerance (G4Sphere.cc:95).
        let err = Volume::sphere(Vec3::ZERO, 0.0, 1.0e-3, 0.0, TAU, 0.0, PI).unwrap_err();
        assert!(
            matches!(err, GeometryError::TooSmall { field: "rmax", .. }),
            "{err}"
        );
        assert!(Volume::sphere(Vec3::ZERO, 0.0, 1.1e-3, 0.0, TAU, 0.0, PI).is_ok());
    }

    #[test]
    fn rejects_out_of_range_angles() {
        let check = |args: [f64; 4], field: &str| {
            let [sp, dp, st, dt] = args;
            let err = Volume::sphere(Vec3::ZERO, 0.0, 1.0, sp, dp, st, dt).unwrap_err();
            assert!(
                matches!(err, GeometryError::AngleOutOfRange { field: f, .. } if f == field),
                "{args:?}: {err}"
            );
        };
        for dp in [0.0, -1.0, TAU + 1e-9] {
            check([0.0, dp, 0.0, PI], "delta_phi");
        }
        for st in [-1e-12, -1.0, PI, 4.0] {
            check([0.0, TAU, st, 1.0], "start_theta");
        }
        for dt in [0.0, -0.0, -1.0, PI * (1.0 + 1e-15), 4.0] {
            check([0.0, TAU, 0.0, dt], "delta_theta");
        }
        // The end of the range may pass pi: it is reduced, as in Geant4.
        assert!(Volume::sphere(Vec3::ZERO, 0.0, 1.0, 0.0, TAU, 3.0, 1.0).is_ok());
    }

    #[test]
    fn rejects_non_finite_values() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let args = [
                ("rmin", [bad, 1.0, 0.0, 1.0, 0.0, 1.0]),
                ("rmax", [0.0, bad, 0.0, 1.0, 0.0, 1.0]),
                ("start_phi", [0.0, 1.0, bad, 1.0, 0.0, 1.0]),
                ("delta_phi", [0.0, 1.0, 0.0, bad, 0.0, 1.0]),
                ("start_theta", [0.0, 1.0, 0.0, 1.0, bad, 1.0]),
                ("delta_theta", [0.0, 1.0, 0.0, 1.0, 0.0, bad]),
            ];
            for (name, [a, b, c, e, f, g]) in args {
                let err = Volume::sphere(Vec3::ZERO, a, b, c, e, f, g).unwrap_err();
                assert!(
                    matches!(err, GeometryError::NonFinite { field, .. } if field == name),
                    "{name}: {err}"
                );
            }
            let err = Volume::sphere(v3(0.0, bad, 0.0), 0.0, 1.0, 0.0, 1.0, 0.0, 1.0);
            assert!(matches!(
                err,
                Err(GeometryError::NonFinite {
                    field: "centre",
                    ..
                })
            ));
        }
    }

    // --- serde ---

    #[test]
    fn toml_round_trip_keeps_the_requested_angles() {
        let s = Volume::sphere(v3(1.0, -2.0, 3.5), 1.5, 4.0, -0.5, 1.0, 0.5, PI).unwrap();
        let text = toml::to_string(&s).unwrap();
        assert!(text.contains("shape = \"sphere\""), "{text}");
        assert!(text.contains("start_phi = -0.5"), "{text}");
        assert!(text.contains("delta_theta = 3.14159"), "{text}");
        let back: Volume = toml::from_str(&text).unwrap();
        assert_eq!(back, s);
        assert_relative_eq!(solid(&back).delta_theta(), PI - 0.5);
    }

    #[test]
    fn deserialisation_validates_through_the_constructor() {
        let base = "shape = \"sphere\"\ncentre = [0.0, 0.0, 0.0]\nrmin = 0.0\nrmax = 1.0\n\
                    start_phi = 0.0\ndelta_phi = 1.0\n";
        let err =
            toml::from_str::<Volume>(&format!("{base}start_theta = 4.0\ndelta_theta = 1.0\n"))
                .unwrap_err();
        assert!(err.to_string().contains("start_theta"), "{err}");
        let err = serde_json::from_str::<Volume>(
            r#"{"shape":"sphere","centre":[0,0,0],"rmin":2,"rmax":1,"start_phi":0,
                "delta_phi":1,"start_theta":0,"delta_theta":1}"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("rmin"), "{err}");
        // A missing field or an unknown one is an error.
        assert!(toml::from_str::<Volume>(&format!("{base}start_theta = 0.0\n")).is_err());
        let extra = format!("{base}start_theta = 0.0\ndelta_theta = 1.0\nhalf_z = 1.0\n");
        assert!(toml::from_str::<Volume>(&extra).is_err());
        let ok = format!("{base}start_theta = 0.0\ndelta_theta = 1.0\n");
        assert!(toml::from_str::<Volume>(&ok).is_ok());
    }

    // --- inside ---

    #[test]
    fn inside_full_sphere() {
        let s = ball();
        assert_eq!(s.inside(Vec3::ZERO), Inside::Inside);
        assert_eq!(s.inside(v3(10.0, 0.0, 0.0)), Inside::Surface);
        assert_eq!(s.inside(v3(0.0, 0.0, -10.0)), Inside::Surface); // south pole
        assert_eq!(s.inside(v3(6.0, 0.0, 8.0)), Inside::Surface);
        assert_eq!(s.inside(v3(10.0 + 0.9 * HRT, 0.0, 0.0)), Inside::Surface);
        assert_eq!(s.inside(v3(10.0 - 0.9 * HRT, 0.0, 0.0)), Inside::Surface);
        assert_eq!(s.inside(v3(10.0 + 1.1 * HRT, 0.0, 0.0)), Inside::Outside);
        assert_eq!(s.inside(v3(10.0 - 1.1 * HRT, 0.0, 0.0)), Inside::Inside);
        assert_eq!(s.inside(v3(6.0, 6.0, 6.0)), Inside::Outside); // r = 10.4
    }

    #[test]
    fn inside_spherical_shell() {
        let s = shell();
        assert_eq!(s.inside(v3(0.0, 7.0, 0.0)), Inside::Inside);
        assert_eq!(s.inside(v3(0.0, 0.0, 4.0)), Inside::Surface);
        assert_eq!(s.inside(v3(4.0 + 1.1 * HRT, 0.0, 0.0)), Inside::Inside);
        assert_eq!(s.inside(v3(4.0 - 0.9 * HRT, 0.0, 0.0)), Inside::Surface);
        assert_eq!(s.inside(v3(4.0 - 1.1 * HRT, 0.0, 0.0)), Inside::Outside);
        assert_eq!(s.inside(v3(2.0, 1.0, 0.0)), Inside::Outside); // in the hole
        assert_eq!(s.inside(Vec3::ZERO), Inside::Outside); // centre of the hole
    }

    #[test]
    fn inside_tolerance_of_a_one_metre_sphere() {
        // fRmaxTolerance = 2e-11 * 1e9 nm = 0.02 nm, so the shell is 0.01 nm on each side.
        let s = sphere(0.0, 1.0e9, 0.0, TAU, 0.0, PI);
        assert_eq!(s.inside(v3(1.0e9 + 0.009, 0.0, 0.0)), Inside::Surface);
        assert_eq!(s.inside(v3(0.0, 1.0e9 - 0.009, 0.0)), Inside::Surface);
        assert_eq!(s.inside(v3(1.0e9 + 0.011, 0.0, 0.0)), Inside::Outside);
        assert_eq!(s.inside(v3(0.0, 0.0, 1.0e9 - 0.011)), Inside::Inside);
    }

    #[test]
    fn inside_phi_segment() {
        let s = phi_section(0.0, FRAC_PI_2);
        assert_eq!(s.inside(v3(3.0, 3.0, 1.0)), Inside::Inside);
        assert_eq!(s.inside(v3(5.0, 0.0, 1.0)), Inside::Surface); // start plane
        assert_eq!(s.inside(v3(0.0, 5.0, -1.0)), Inside::Surface); // end plane
        assert_eq!(s.inside(v3(-3.0, 3.0, 0.0)), Inside::Outside);
        assert_eq!(s.inside(v3(3.0, -3.0, 0.0)), Inside::Outside);
        assert_eq!(s.inside(Vec3::ZERO), Inside::Surface); // centre of a segment
        // The phi shell is angular.
        let at = |phi: f64| point_at(&s, 5.0, 1.0, phi);
        assert_eq!(s.inside(at(-0.9 * HAT)), Inside::Surface);
        assert_eq!(s.inside(at(-1.1 * HAT)), Inside::Outside);
        assert_eq!(s.inside(at(1.1 * HAT)), Inside::Inside);
        assert_eq!(s.inside(at(FRAC_PI_2 + 0.9 * HAT)), Inside::Surface);
        assert_eq!(s.inside(at(FRAC_PI_2 + 1.1 * HAT)), Inside::Outside);
        // A Geant4 property: the phi test is skipped on the z axis, which is Inside.
        assert_eq!(s.inside(v3(0.0, 0.0, 5.0)), Inside::Inside);
        assert_eq!(s.inside(v3(0.0, 0.0, 10.0)), Inside::Surface);

        // [-pi/4, pi/4] and [pi, 2 pi]: the atan2 angle is moved towards the segment.
        let s = phi_section(-FRAC_PI_4, FRAC_PI_2);
        assert_eq!(s.inside(v3(5.0, -1.0, 0.0)), Inside::Inside);
        assert_eq!(s.inside(v3(4.0, -4.0, 0.0)), Inside::Surface);
        assert_eq!(s.inside(v3(-5.0, 0.0, 0.0)), Inside::Outside);
        let s = phi_section(PI, PI);
        assert_eq!(s.inside(v3(0.0, -5.0, 0.0)), Inside::Inside);
        assert_eq!(s.inside(v3(5.0, 0.0, 0.0)), Inside::Surface);
        assert_eq!(s.inside(v3(5.0, 1e-12, 0.0)), Inside::Surface);
        assert_eq!(s.inside(v3(5.0, 1.0, 0.0)), Inside::Outside);
    }

    #[test]
    fn inside_theta_cones() {
        // theta in [pi/4, 3 pi/4]: both cones, the poles excluded.
        let s = theta_section(FRAC_PI_4, FRAC_PI_2);
        let at = |theta: f64| point_at(&s, 5.0, theta, 0.3);
        assert_eq!(s.inside(at(FRAC_PI_2)), Inside::Inside);
        assert_eq!(s.inside(at(FRAC_PI_4)), Inside::Surface);
        assert_eq!(s.inside(at(0.75 * PI)), Inside::Surface);
        assert_eq!(s.inside(at(FRAC_PI_4 - 0.9 * HAT)), Inside::Surface);
        assert_eq!(s.inside(at(FRAC_PI_4 - 1.1 * HAT)), Inside::Outside);
        assert_eq!(s.inside(at(FRAC_PI_4 + 1.1 * HAT)), Inside::Inside);
        assert_eq!(s.inside(at(0.75 * PI + 0.9 * HAT)), Inside::Surface);
        assert_eq!(s.inside(at(0.75 * PI + 1.1 * HAT)), Inside::Outside);
        assert_eq!(s.inside(at(0.75 * PI - 1.1 * HAT)), Inside::Inside);
        assert_eq!(s.inside(v3(0.0, 0.0, 5.0)), Inside::Outside); // +z axis
        assert_eq!(s.inside(v3(0.0, 0.0, -5.0)), Inside::Outside); // -z axis
        assert_eq!(s.inside(Vec3::ZERO), Inside::Surface); // apex
        // On rmax and a cone at once, and beyond the radial shell.
        assert_eq!(
            s.inside(point_at(&s, 10.0, FRAC_PI_4, 1.0)),
            Inside::Surface
        );
        assert_eq!(
            s.inside(point_at(&s, 10.1, FRAC_PI_2, 1.0)),
            Inside::Outside
        );
    }

    #[test]
    fn inside_hemispheres_and_poles() {
        // Upper hemisphere: the end cone at pi/2 is the plane z = 0.
        let s = theta_section(0.0, FRAC_PI_2);
        assert_eq!(s.inside(v3(1.0, 2.0, 3.0)), Inside::Inside);
        assert_eq!(s.inside(v3(1.0, 2.0, 0.0)), Inside::Surface);
        assert_eq!(s.inside(v3(1.0, 2.0, -3.0)), Inside::Outside);
        assert_eq!(s.inside(v3(0.0, 0.0, 5.0)), Inside::Inside); // +z axis
        assert_eq!(s.inside(v3(0.0, 0.0, 10.0)), Inside::Surface); // north pole
        assert_eq!(s.inside(v3(0.0, 0.0, -5.0)), Inside::Outside);
        // The plane's shell is angular too: |z| <= rho * kAngTolerance/2.
        assert_eq!(s.inside(v3(5.0, 0.0, -5.0 * 0.9 * HAT)), Inside::Surface);
        assert_eq!(s.inside(v3(5.0, 0.0, -5.0 * 1.1 * HAT)), Inside::Outside);
        // Lower hemisphere: the start cone at pi/2 is the plane.
        let s = theta_section(FRAC_PI_2, FRAC_PI_2);
        assert_eq!(s.inside(v3(1.0, 2.0, -3.0)), Inside::Inside);
        assert_eq!(s.inside(v3(1.0, 2.0, 0.0)), Inside::Surface);
        assert_eq!(s.inside(v3(1.0, 2.0, 3.0)), Inside::Outside);
        assert_eq!(s.inside(v3(0.0, 0.0, -10.0)), Inside::Surface); // south pole
        // A polar cap keeps its pole; a start cone above 0 excludes it.
        let cap = theta_section(0.0, FRAC_PI_4);
        assert_eq!(cap.inside(v3(0.0, 0.0, 9.0)), Inside::Inside);
        assert_eq!(cap.inside(v3(0.0, 0.0, -9.0)), Inside::Outside);
        let ring = theta_section(0.1, PI);
        assert_eq!(ring.inside(v3(0.0, 0.0, 9.0)), Inside::Outside);
        assert_eq!(ring.inside(v3(0.0, 0.0, -9.0)), Inside::Inside);
    }

    #[test]
    fn inside_follows_the_centre_offset() {
        let c = v3(100.0, -50.0, 7.0);
        let s = Volume::sphere(c, 0.0, 10.0, 0.0, TAU, 0.0, FRAC_PI_2).unwrap();
        assert_eq!(s.inside(c + v3(1.0, 1.0, 1.0)), Inside::Inside);
        assert_eq!(s.inside(c + v3(10.0, 0.0, 0.0)), Inside::Surface);
        assert_eq!(s.inside(c), Inside::Surface);
        assert_eq!(s.inside(c + v3(1.0, 1.0, -1.0)), Inside::Outside);
        assert_eq!(s.inside(Vec3::ZERO), Inside::Outside);
        assert_eq!(s.inside(v3(f64::NAN, 0.0, 0.0)), Inside::Outside);
    }

    // --- distance_to_out: radial ---

    #[test]
    fn radial_exits_of_a_full_sphere() {
        let s = ball();
        for v in [
            dir(1.0, 0.0, 0.0),
            dir(0.0, -1.0, 0.0),
            dir(0.0, 0.0, 1.0),
            dir(1.0, -2.0, 3.0),
        ] {
            assert_relative_eq!(d(&s, Vec3::ZERO, v), 10.0, max_relative = 1e-14);
        }
        // From p along v: -p.v + sqrt((p.v)^2 - |p|^2 + R^2).
        let p = v3(3.0, 4.0, 0.0);
        assert_relative_eq!(
            d(&s, p, dir(1.0, 0.0, 0.0)),
            84f64.sqrt() - 3.0,
            max_relative = 1e-14
        );
        assert_relative_eq!(
            d(&s, p, dir(0.0, 0.0, 1.0)),
            75f64.sqrt(),
            max_relative = 1e-14
        );
        let v = dir(-1.0, 1.0, 2.0);
        let pv = p.dot(v.as_vec3());
        let expected = -pv + (pv * pv - 25.0 + 100.0).sqrt();
        assert_relative_eq!(d(&s, p, v), expected, max_relative = 1e-14);
    }

    #[test]
    fn chord_lengths_of_a_full_sphere() {
        let s = ball();
        // From the surface, inward at angle alpha to the normal: the chord is 2 R cos(alpha).
        for alpha in [0.0, 0.3, FRAC_PI_4, PI / 3.0, 1.5] {
            let v = dir(-alpha.cos(), alpha.sin(), 0.0);
            let got = d(&s, v3(10.0, 0.0, 0.0), v);
            assert_relative_eq!(got, 20.0 * alpha.cos(), max_relative = 1e-13);
        }
        // Through an inside point, both ways: 2 sqrt(R^2 - b^2) for impact parameter b.
        let p = v3(2.0, -3.0, 4.0);
        for v in [
            dir(1.0, 0.0, 0.0),
            dir(0.3, 0.4, -0.5),
            dir(-1.0, -1.0, 1.0),
        ] {
            let back = Direction::new(v.as_vec3() * -1.0).unwrap();
            let b2 = p.cross(v.as_vec3()).dot(p.cross(v.as_vec3()));
            let chord = d(&s, p, v) + d(&s, p, back);
            assert_relative_eq!(chord, 2.0 * (100.0 - b2).sqrt(), max_relative = 1e-13);
        }
    }

    #[test]
    fn exits_through_the_rmin_hole_wall() {
        let s = shell();
        assert_relative_eq!(d(&s, v3(7.0, 0.0, 0.0), dir(-1.0, 0.0, 0.0)), 3.0);
        assert_relative_eq!(d(&s, v3(7.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)), 3.0);
        assert_relative_eq!(d(&s, v3(0.0, 0.0, -6.0), dir(0.0, 0.0, 1.0)), 2.0);
        // Chord through the hole at y = 2: rmin at x = sqrt(12).
        let got = d(&s, v3(7.0, 2.0, 0.0), dir(-1.0, 0.0, 0.0));
        assert_relative_eq!(got, 7.0 - 12f64.sqrt(), max_relative = 1e-14);
        // Oblique: the near root of rmin, -p.v - sqrt((p.v)^2 - |p|^2 + rmin^2).
        let (p, v) = (v3(7.0, 0.0, 0.0), dir(-0.96, 0.0, -0.28));
        let pv = p.dot(v.as_vec3());
        let expected = -pv - (pv * pv - 49.0 + 16.0).sqrt();
        assert_relative_eq!(d(&s, p, v), expected, max_relative = 1e-13);
        // Missing the hole: from (7, 0, 0) along (-0.6, 0, -0.8) to rmax.
        let (p, v) = (v3(7.0, 0.0, 0.0), dir(-0.6, 0.0, -0.8));
        let pv = p.dot(v.as_vec3());
        let expected = -pv + (pv * pv - 49.0 + 100.0).sqrt();
        assert_relative_eq!(d(&s, p, v), expected, max_relative = 1e-13);
        // Tangent to rmin at (0, 4, 0): a boundary point, so the ray stops there
        // (d2 = 0 counts as a hit, G4Sphere.cc:1820).
        assert_eq!(d(&s, v3(7.0, 4.0, 0.0), dir(-1.0, 0.0, 0.0)), 7.0);
    }

    // --- distance_to_out: theta cones and planes ---

    #[test]
    fn start_cone_exits() {
        // Concave start cone: theta in [pi/4, pi]; the cone is rho = z.
        let s = theta_section(FRAC_PI_4, PI);
        assert_relative_eq!(d(&s, v3(5.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)), 5.0);
        assert_relative_eq!(d(&s, v3(5.0, 0.0, -3.0), dir(0.0, 0.0, 1.0)), 8.0);
        // Oblique, towards the axis and up: x = 5 - 0.6 t meets z = 0.8 t at t = 5/1.4.
        let got = d(&s, v3(5.0, 0.0, 0.0), dir(-0.6, 0.0, 0.8));
        assert_relative_eq!(got, 5.0 / 1.4, max_relative = 1e-14);
        // Down and away from the cone: rmax.
        assert_relative_eq!(
            d(&s, v3(5.0, 0.0, 0.0), dir(0.0, 0.0, -1.0)),
            75f64.sqrt(),
            max_relative = 1e-14
        );
        // Convex start cone: theta in [3 pi/4, pi], the solid is a cone about -z.
        let s = theta_section(0.75 * PI, PI);
        assert_relative_eq!(
            d(&s, v3(0.0, 0.0, -5.0), dir(1.0, 0.0, 0.0)),
            5.0,
            max_relative = 1e-14
        );
        assert_relative_eq!(
            d(&s, v3(0.0, 0.0, -5.0), dir(0.0, -1.0, 0.0)),
            5.0,
            max_relative = 1e-14
        );
        // Up the axis: leaves through the apex.
        assert_relative_eq!(
            d(&s, v3(0.0, 0.0, -5.0), dir(0.0, 0.0, 1.0)),
            5.0,
            max_relative = 1e-14
        );
        // Down the axis: rmax at the south pole.
        assert_relative_eq!(d(&s, v3(0.0, 0.0, -5.0), dir(0.0, 0.0, -1.0)), 5.0);
    }

    #[test]
    fn end_cone_exits() {
        // Convex end cone: theta in [0, pi/4], a polar cap.
        let s = theta_section(0.0, FRAC_PI_4);
        assert_relative_eq!(
            d(&s, v3(0.0, 0.0, 5.0), dir(1.0, 0.0, 0.0)),
            5.0,
            max_relative = 1e-14
        );
        assert_relative_eq!(
            d(&s, v3(0.0, 0.0, 5.0), dir(0.0, 0.0, -1.0)),
            5.0,
            max_relative = 1e-14
        );
        // Concave end cone: theta in [0, 3 pi/4], a cone about -z removed.
        let s = theta_section(0.0, 0.75 * PI);
        assert_relative_eq!(
            d(&s, v3(5.0, 0.0, 0.0), dir(0.0, 0.0, -1.0)),
            5.0,
            max_relative = 1e-14
        );
        // Down the axis: leaves through the apex into the removed cone.
        assert_relative_eq!(
            d(&s, v3(0.0, 0.0, 5.0), dir(0.0, 0.0, -1.0)),
            5.0,
            max_relative = 1e-14
        );
        // Both cones: theta in [pi/4, 3 pi/4]; from (5, 0, 0) along (-0.6, 0, -0.8) the
        // lower cone (rho = -z) is met at t = 5/1.4, before the upper one.
        let s = theta_section(FRAC_PI_4, FRAC_PI_2);
        let got = d(&s, v3(5.0, 0.0, 0.0), dir(-0.6, 0.0, -0.8));
        assert_relative_eq!(got, 5.0 / 1.4, max_relative = 1e-14);
        let got = d(&s, v3(5.0, 0.0, 0.0), dir(-0.6, 0.0, 0.8));
        assert_relative_eq!(got, 5.0 / 1.4, max_relative = 1e-14);
    }

    #[test]
    fn half_pi_planes() {
        let upper = theta_section(0.0, FRAC_PI_2);
        assert_eq!(d(&upper, v3(1.0, 2.0, 3.0), dir(0.0, 0.0, -1.0)), 3.0);
        assert_relative_eq!(
            d(&upper, v3(1.0, 2.0, 3.0), dir(0.6, 0.0, -0.8)),
            3.0 / 0.8,
            max_relative = 1e-14
        );
        // Parallel to the plane: rmax.
        assert_relative_eq!(
            d(&upper, v3(0.0, 0.0, 3.0), dir(1.0, 0.0, 0.0)),
            91f64.sqrt(),
            max_relative = 1e-14
        );
        let lower = theta_section(FRAC_PI_2, FRAC_PI_2);
        assert_eq!(d(&lower, v3(1.0, 2.0, -3.0), dir(0.0, 0.0, 1.0)), 3.0);
        assert_relative_eq!(
            d(&lower, v3(1.0, 2.0, -3.0), dir(0.0, -0.6, 0.8)),
            3.0 / 0.8,
            max_relative = 1e-14
        );
        // Up or down the z axis to the plane, which meets the axis at the apex.
        assert_eq!(d(&lower, v3(0.0, 0.0, -1.0), dir(0.0, 0.0, 1.0)), 1.0);
        assert_eq!(d(&upper, v3(0.0, 0.0, 1.0), dir(0.0, 0.0, -1.0)), 1.0);
    }

    #[test]
    fn rays_through_the_poles() {
        // Along the z axis of a full sphere, out through the poles.
        let s = ball();
        assert_eq!(d(&s, v3(0.0, 0.0, -3.0), dir(0.0, 0.0, 1.0)), 13.0);
        assert_eq!(d(&s, v3(0.0, 0.0, 3.0), dir(0.0, 0.0, -1.0)), 13.0);
        assert_eq!(s.inside(v3(0.0, 0.0, 10.0)), Inside::Surface);
        // A polar cap: out through the north pole; across the +z axis inside the cap.
        let cap = theta_section(0.0, FRAC_PI_4);
        assert_eq!(d(&cap, v3(0.0, 0.0, 1.0), dir(0.0, 0.0, 1.0)), 9.0);
        assert_relative_eq!(
            d(&cap, v3(-1.0, 0.0, 5.0), dir(1.0, 0.0, 0.0)),
            6.0,
            max_relative = 1e-14
        );
        // The south pole of a section that reaches theta = pi.
        let s = theta_section(FRAC_PI_4, PI);
        assert_eq!(d(&s, v3(0.0, 0.0, -1.0), dir(0.0, 0.0, -1.0)), 9.0);
        // A section without its poles: the ray towards the +z axis meets the start cone.
        let s = theta_section(FRAC_PI_4, FRAC_PI_2);
        assert_relative_eq!(d(&s, v3(5.0, 0.0, 2.0), dir(-1.0, 0.0, 0.0)), 3.0);
        // Along the z axis of a phi segment: the phi planes do not limit it
        // (G4Sphere.cc:2452-2455).
        let s = phi_section(0.0, FRAC_PI_2);
        assert_eq!(s.inside(v3(0.0, 0.0, -5.0)), Inside::Inside);
        assert_eq!(d(&s, v3(0.0, 0.0, -5.0), dir(0.0, 0.0, 1.0)), 15.0);
    }

    #[test]
    fn ray_through_the_apex_stops_there() {
        // The apex of a concave cone is a boundary point of the solid (G4Sphere.cc:1983).
        let s = theta_section(FRAC_PI_4, PI);
        assert_eq!(s.inside(Vec3::ZERO), Inside::Surface);
        assert_relative_eq!(
            d(&s, v3(5.0, 0.0, 0.0), dir(-1.0, 0.0, 0.0)),
            5.0,
            max_relative = 1e-14
        );
    }

    // --- distance_to_out: phi planes ---

    #[test]
    fn phi_plane_exits() {
        let s = phi_section(0.0, FRAC_PI_2);
        let p = v3(3.0, 3.0, 1.0);
        assert_relative_eq!(d(&s, p, dir(0.0, -1.0, 0.0)), 3.0, max_relative = 1e-14);
        assert_relative_eq!(d(&s, p, dir(-1.0, 0.0, 0.0)), 3.0, max_relative = 1e-14);
        let v = dir(1.0, 1.0, 0.0);
        let pv = p.dot(v.as_vec3());
        let expected = -pv + (pv * pv - 19.0 + 100.0).sqrt();
        assert_relative_eq!(d(&s, p, v), expected, max_relative = 1e-14);
        // Oblique onto the start plane.
        let got = d(&s, v3(2.0, 3.0, 0.0), dir(0.0, -0.6, 0.8));
        assert_relative_eq!(got, 3.0 / 0.6, max_relative = 1e-14);
        // Opening above pi: from phi = 3 pi/4 along -y, across phi = pi to rmax.
        let s = phi_section(0.0, 1.5 * PI);
        let got = d(&s, v3(-5.0, 5.0, 0.0), dir(0.0, -1.0, 0.0));
        assert_relative_eq!(got, 5.0 + 75f64.sqrt(), max_relative = 1e-14);
        assert_relative_eq!(
            d(&s, v3(-5.0, -5.0, 0.0), dir(1.0, 0.0, 0.0)),
            5.0,
            max_relative = 1e-14
        );
        // Across phi = 0.
        let s = phi_section(-FRAC_PI_4, FRAC_PI_2);
        assert_relative_eq!(
            d(&s, v3(5.0, 0.0, 0.0), dir(0.0, 1.0, 0.0)),
            5.0,
            max_relative = 1e-14
        );
    }

    // --- distance_to_out: surface starts ---

    #[test]
    fn surface_start_moving_outward_gives_zero() {
        let cases = [
            (ball(), v3(10.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)), // rmax
            (ball(), v3(10.0, 0.0, 0.0), dir(1.0, 1.0, 0.3)),
            (ball(), v3(10.0, 0.0, 0.0), dir(0.0, 1.0, 0.0)), // tangent to rmax
            (ball(), v3(0.0, 0.0, 10.0 + 0.9 * HRT), dir(0.0, 0.0, 1.0)),
            (ball(), v3(0.0, 0.0, 10.0 - 0.9 * HRT), dir(0.0, 0.0, 1.0)),
            (shell(), v3(4.0, 0.0, 0.0), dir(-1.0, 0.0, 0.0)), // rmin, into the hole
            (shell(), v3(0.0, -4.0, 0.0), dir(0.3, 1.0, 0.2)),
            (shell(), v3(4.0 - 0.9 * HRT, 0.0, 0.0), dir(-1.0, 0.0, 0.0)),
            // Concave start cone, into the removed cone about +z.
            (
                theta_section(FRAC_PI_4, PI),
                v3(5.0, 0.0, 5.0),
                dir(-1.0, 0.0, 0.0),
            ),
            (
                theta_section(FRAC_PI_4, PI),
                v3(0.0, 5.0, 5.0),
                dir(0.0, 0.0, 1.0),
            ),
            // Convex start cone, out of the cone about -z.
            (
                theta_section(0.75 * PI, PI),
                v3(5.0, 0.0, -5.0),
                dir(1.0, 0.0, 0.0),
            ),
            // Convex end cone, out of the polar cap.
            (
                theta_section(0.0, FRAC_PI_4),
                v3(5.0, 0.0, 5.0),
                dir(1.0, 0.0, 0.0),
            ),
            // Along the normal, which is parallel to the lower nappe (|t1| < kAngTol/2).
            (
                theta_section(0.0, FRAC_PI_4),
                v3(0.0, 5.0, 5.0),
                dir(0.0, 1.0, -1.0),
            ),
            // Concave end cone, into the removed cone about -z.
            (
                theta_section(0.0, 0.75 * PI),
                v3(5.0, 0.0, -5.0),
                dir(-1.0, 0.0, 0.0),
            ),
            (
                theta_section(0.0, 0.75 * PI),
                v3(5.0, 0.0, -5.0),
                dir(0.0, 0.0, -1.0),
            ),
            // The planes z = 0, including a grazing direction.
            (
                theta_section(0.0, FRAC_PI_2),
                v3(1.0, 2.0, 0.0),
                dir(0.0, 0.0, -1.0),
            ),
            (
                theta_section(0.0, FRAC_PI_2),
                v3(1.0, 2.0, 0.0),
                dir(1.0, 0.0, -1e-6),
            ),
            (
                theta_section(FRAC_PI_2, FRAC_PI_2),
                v3(1.0, 2.0, 0.0),
                dir(0.0, 0.3, 1.0),
            ),
            // Phi planes.
            (
                phi_section(0.0, FRAC_PI_2),
                v3(5.0, 0.0, 1.0),
                dir(0.0, -1.0, 0.0),
            ),
            (
                phi_section(0.0, FRAC_PI_2),
                v3(0.0, 5.0, 1.0),
                dir(-1.0, 0.2, 0.5),
            ),
            (
                phi_section(0.0, 1.5 * PI),
                v3(0.0, -5.0, 0.0),
                dir(1.0, 0.0, 0.0),
            ),
        ];
        for (s, p, v) in cases {
            assert_eq!(s.inside(p), Inside::Surface, "{p:?}");
            assert_eq!(s.distance_to_out(p, v), Some(0.0), "{p:?} {v:?}");
        }
    }

    #[test]
    fn surface_start_moving_inward_crosses_the_solid() {
        assert_relative_eq!(d(&ball(), v3(10.0, 0.0, 0.0), dir(-1.0, 0.0, 0.0)), 20.0);
        assert_relative_eq!(d(&shell(), v3(4.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)), 6.0);
        let outer = v3(10.0 + 0.9 * HRT, 0.0, 0.0);
        assert_relative_eq!(
            d(&ball(), outer, dir(-1.0, 0.0, 0.0)),
            20.0 + 0.9 * HRT,
            max_relative = 1e-14
        );
        // Concave start cone: down from the cone to rmax.
        let s = theta_section(FRAC_PI_4, PI);
        assert_relative_eq!(
            d(&s, v3(5.0, 0.0, 5.0), dir(0.0, 0.0, -1.0)),
            5.0 + 75f64.sqrt(),
            max_relative = 1e-14
        );
        // Convex start cone: across the cone about -z to its far side.
        let s = theta_section(0.75 * PI, PI);
        assert_relative_eq!(
            d(&s, v3(5.0, 0.0, -5.0), dir(-1.0, 0.0, 0.0)),
            10.0,
            max_relative = 1e-14
        );
        // Convex end cone: across the cap.
        let s = theta_section(0.0, FRAC_PI_4);
        assert_relative_eq!(
            d(&s, v3(5.0, 0.0, 5.0), dir(-1.0, 0.0, 0.0)),
            10.0,
            max_relative = 1e-14
        );
        // Concave end cone: up from the cone to rmax.
        let s = theta_section(0.0, 0.75 * PI);
        assert_relative_eq!(
            d(&s, v3(5.0, 0.0, -5.0), dir(0.0, 0.0, 1.0)),
            5.0 + 75f64.sqrt(),
            max_relative = 1e-14
        );
        // A plane, then a phi plane.
        let s = theta_section(0.0, FRAC_PI_2);
        assert_relative_eq!(
            d(&s, v3(1.0, 2.0, 0.0), dir(0.0, 0.0, 1.0)),
            95f64.sqrt(),
            max_relative = 1e-14
        );
        let s = phi_section(0.0, FRAC_PI_2);
        assert_relative_eq!(
            d(&s, v3(5.0, 0.0, 0.0), dir(0.0, 1.0, 0.0)),
            75f64.sqrt(),
            max_relative = 1e-14
        );
    }

    #[test]
    fn outside_start_gives_none() {
        let v = dir(-1.0, 0.0, 0.0);
        assert_eq!(ball().distance_to_out(v3(11.0, 0.0, 0.0), v), None);
        assert_eq!(shell().distance_to_out(v3(2.0, 0.0, 0.0), v), None);
        let cone = theta_section(FRAC_PI_4, PI);
        assert_eq!(cone.distance_to_out(v3(0.0, 1.0, 5.0), v), None);
        let segment = phi_section(0.0, FRAC_PI_2);
        assert_eq!(segment.distance_to_out(v3(-3.0, 3.0, 0.0), v), None);
    }

    #[test]
    fn exits_with_a_centre_offset() {
        let c = v3(1.0e3, -2.0e3, 5.0e2);
        let s = Volume::sphere(c, 4.0, 10.0, 0.0, FRAC_PI_2, FRAC_PI_4, FRAC_PI_2).unwrap();
        assert_relative_eq!(
            d(&s, c + v3(5.0, 2.0, 0.0), dir(0.0, -1.0, 0.0)),
            2.0,
            max_relative = 1e-12
        );
        assert_relative_eq!(
            d(&s, c + v3(5.0, 1.0, 0.0), dir(0.0, 0.0, 1.0)),
            26f64.sqrt(),
            max_relative = 1e-12
        );
        assert_eq!(
            s.distance_to_out(v3(5.0, 2.0, 0.0), dir(0.0, -1.0, 0.0)),
            None
        );
    }

    // --- Geant4 properties and deviations ---

    /// Deviation 1 (a) and a Geant4 property. On the z axis of phi in [pi/2, 2 pi] the
    /// point is Inside (the phi test is skipped there). A direction at -pi/4, inside the
    /// segment as 7 pi/4, leaves through rmax; G4Sphere compares the raw atan2 angle with
    /// [pi/2, 2 pi] (G4Sphere.cc:2441-2442) and returns 0. A direction outside the segment
    /// leaves at once, although the start is Inside.
    #[test]
    fn start_on_the_z_axis_of_a_phi_segment() {
        let s = phi_section(FRAC_PI_2, 1.5 * PI);
        let p = v3(0.0, 0.0, 3.0);
        assert_eq!(s.inside(p), Inside::Inside);
        assert_relative_eq!(
            d(&s, p, dir(1.0, -1.0, 0.0)),
            91f64.sqrt(),
            max_relative = 1e-14
        );
        assert_relative_eq!(
            d(&s, p, dir(-1.0, 0.0, 0.0)),
            91f64.sqrt(),
            max_relative = 1e-14
        );
        assert_eq!(d(&s, p, dir(0.0, 0.0, 1.0)), 7.0);
        assert_eq!(d(&s, p, dir(1.0, 1.0, 0.0)), 0.0);
        assert_eq!(d(&s, p, dir(1.0, 0.2, -1.0)), 0.0);
    }

    /// Deviation 1 (a). Phi in [pi/2, 2 pi]; from phi = 3 pi/4 through the z axis with the
    /// direction at -pi/4 (= 7 pi/4, inside). The crossing on the axis is not an exit
    /// (G4Tubs.cc:1364-1372), so the ray goes on to rmax: 5 + 10. G4Sphere's raw atan2 test
    /// (G4Sphere.cc:2363-2371) stops it on the axis, at 5.
    #[test]
    fn ray_through_the_z_axis_with_a_direction_below_the_start_angle() {
        let s = phi_section(FRAC_PI_2, 1.5 * PI);
        let p = point_at(&s, 5.0, FRAC_PI_2, 0.75 * PI);
        let v = dir(1.0, -1.0, 0.0);
        assert_eq!(s.inside(p), Inside::Inside);
        assert_relative_eq!(d(&s, p, v), 15.0, max_relative = 1e-14);
    }

    /// Deviation 1 (b). Phi in [0.3, 4.8]: the start lies 1e-9 nm outside the start plane,
    /// within the angular shell (2e-10 rad at r = 5), and leaves through that plane with
    /// `compE > 0`. G4Sphere falls into its "outside both full planes" case for an opening
    /// above pi (G4Sphere.cc:2238-2252), which needs `compS < 0` and `compE < 0` to stop,
    /// and lets the ray go on. The tolerant G4Tubs test (G4Tubs.cc:1345-1348) stops it.
    #[test]
    fn start_just_outside_a_phi_plane_still_leaves() {
        let (start, delta) = (0.3, 4.5);
        let s = phi_section(start, delta);
        let radial = v3(start.cos(), start.sin(), 0.0);
        let normal = v3(start.sin(), -start.cos(), 0.0); // outward at the start plane
        let p = radial * 5.0 + normal * 1e-9;
        let v = Direction::new(normal * 0.5 - radial * 0.866).unwrap();
        let seg = solid(&s).phi;
        let (sin_s, cos_s) = seg.start_trig();
        let (sin_e, cos_e) = seg.end_trig();
        let w = v.as_vec3();
        assert!(p.x * sin_s - p.y * cos_s > 0.0, "pDistS");
        assert!(-p.x * sin_e + p.y * cos_e > 0.0, "pDistE");
        assert!(-sin_s * w.x + cos_s * w.y < 0.0, "compS");
        assert!(sin_e * w.x - cos_e * w.y > 0.0, "compE");
        assert_eq!(s.inside(p), Inside::Surface);
        assert_eq!(d(&s, p, v), 0.0);
    }

    /// Deviation 2. In the outermost tol^2/(8 rmax) of the rmax shell, Inside gives Surface
    /// but G4Sphere's linearised test `c < fRmaxTolerance*fRmax` (G4Sphere.cc:1773) fails,
    /// so it has no rmax limit (kInfinity for an outward ray). With rmin below its half
    /// tolerance, G4Sphere's negative `Rmin_minus` (1727) skips the radial tests near the
    /// centre in the same way.
    #[test]
    fn every_start_has_an_rmax_limit() {
        let s = ball();
        let tol = solid(&s).rmax_tol;
        let rad2 = 100.0 + tol * 10.0 + 0.4 * (0.5 * tol).powi(2);
        let p = v3(rad2.sqrt(), 0.0, 0.0);
        assert!(p.x * p.x - 100.0 >= tol * 10.0, "past the linearised shell");
        assert_eq!(s.inside(p), Inside::Surface);
        assert_eq!(d(&s, p, dir(1.0, 0.0, 0.0)), 0.0);
        assert_relative_eq!(
            d(&s, p, dir(-1.0, 0.0, 0.0)),
            p.x + 10.0,
            max_relative = 1e-14
        );

        let s = sphere(1.0e-4, 10.0, 0.0, TAU, 0.0, PI);
        let p = v3(2.0e-4, 0.0, 0.0);
        assert_eq!(s.inside(p), Inside::Surface);
        assert_relative_eq!(
            d(&s, p, dir(1.0, 0.0, 0.0)),
            10.0 - 2.0e-4,
            max_relative = 1e-14
        );
    }

    /// Deviation 3. Theta in [3 pi/4, pi]: the start lies exactly on the cone (its
    /// quadratic has c = 0, so the first root is exactly 0) and moves in, across the cone to
    /// its far side at 2 rho. G4Sphere.cc:1963 tests |CLHEP::s| instead of |sd|, keeps the
    /// zero root, then rejects it as not beyond the tolerance (1968) and ignores the cone:
    /// it would return the rmax exit, 1.41 + sqrt(98) = 11.3 nm.
    #[test]
    fn start_on_a_convex_start_cone_with_a_zero_first_root() {
        let s = theta_section(0.75 * PI, PI);
        let p = point_at(&s, 2.0, 0.75 * PI, 0.0);
        let tan2 = solid(&s).theta.tan_start2;
        assert_eq!(p.x * p.x - p.z * p.z * tan2, 0.0, "c of the cone quadratic");
        assert_eq!(s.inside(p), Inside::Surface);
        let got = d(&s, p, dir(-1.0, 0.0, 0.0));
        assert_relative_eq!(got, 2.0 * p.x, max_relative = 1e-14);
    }

    /// Deviation 4. A 50 nm polar cap, theta in [0, pi/4]: from 20 nm up the axis
    /// sideways, the cone is met at 20 nm (d2 = 400 nm^2). With G4Sphere.cc:2094 evaluated
    /// in Geant4's mm, d2 = 4e-10 mm^2 < 5e-10 is set to 0, the cone is missed and the step
    /// runs to rmax at sqrt(2100) = 45.8 nm.
    #[test]
    fn end_cone_of_a_nanometre_sphere() {
        let s = sphere(0.0, 50.0, 0.0, TAU, 0.0, FRAC_PI_4);
        let got = d(&s, v3(0.0, 0.0, 20.0), dir(1.0, 0.0, 0.0));
        assert_relative_eq!(got, 20.0, max_relative = 1e-14);
        // And a sub-nm one, where the half-tolerance clamp in nm would bite.
        let s = sphere(0.0, 0.05, 0.0, TAU, 0.0, FRAC_PI_4);
        let got = d(&s, v3(0.0, 0.0, 0.01), dir(1.0, 0.0, 0.0));
        assert_relative_eq!(got, 0.01, max_relative = 1e-12);
    }

    /// Deviation 5. Theta in [1e-5, pi]: a vertical ray is parallel to the narrow cone
    /// (|t1| = tan^2 = 1e-10 < kAngTolerance/2), and from below the equator the root of the
    /// linear equation is behind the start: G4Sphere.cc:1920 returns about -2.5e9 nm. The
    /// cone is not reached before rmax.
    #[test]
    fn parallel_ray_root_behind_the_start() {
        let s = theta_section(1e-5, PI);
        let tan2 = solid(&s).theta.tan_start2;
        assert!((1.0 - (1.0 + tan2)).abs() < HAT, "t1 for v = +z");
        let p = v3(1.0, 0.0, -2.0);
        assert_eq!(s.inside(p), Inside::Inside);
        let got = d(&s, p, dir(0.0, 0.0, 1.0));
        assert_relative_eq!(got, 2.0 + 99f64.sqrt(), max_relative = 1e-14);
    }

    /// Deviation 5, the other nappe. Theta in [pi/4, pi]: from (0, 0, -5) along
    /// (1, 0, 1)/sqrt 2, parallel to the cone (|t1| ~ 4e-16), the ray crosses the lower
    /// nappe, which is not a boundary, at 5/sqrt 2 = 3.54 nm, where G4Sphere.cc:1920 stops it.
    /// It never reaches the removed cone about +z (rho = z + 5 > z), so it leaves through
    /// rmax at 12.89 nm.
    #[test]
    fn parallel_ray_crossing_the_other_nappe_goes_on() {
        let s = theta_section(FRAC_PI_4, PI);
        let tan2 = solid(&s).theta.tan_start2;
        let (p, v) = (v3(0.0, 0.0, -5.0), dir(1.0, 0.0, 1.0));
        let w = v.as_vec3();
        assert!(
            (1.0 - w.z * w.z * (1.0 + tan2)).abs() < HAT,
            "parallel to the cone"
        );
        assert_eq!(s.inside(p), Inside::Inside);
        let pv = p.dot(w);
        let expected = -pv + (pv * pv - 25.0 + 100.0).sqrt();
        assert_relative_eq!(expected, 12.89, max_relative = 1e-3);
        assert_relative_eq!(d(&s, p, v), expected, max_relative = 1e-14);
    }

    /// Deviation 7. Starts 0.9 h = 4.5e-4 nm inside each kind of cone (h = halfRmaxTolerance
    /// for rmax = 10 nm), at r = 5 nm, moving out along the cone's normal. The horizontal
    /// distance 0.9 h / cos(pi/4) = 1.27 h misses Geant4's on-cone test (G4Sphere.cc:1926,
    /// 2064), and the exit root 0.9 h fails its `sd > h` acceptance (1968, 1983, 2111,
    /// 2130). Geant4 therefore ignores the cone and runs on through the removed region to
    /// rmax, 8.66 nm away. The first case is the reviewer's example. The port leaves at once.
    #[test]
    fn start_within_h_of_a_cone_moving_out_leaves_at_once() {
        let h = 0.5 * RAD_TOLERANCE;
        let off = 0.9 * h / 5.0;
        // (section, polar angle of the start, sign of the outward normal along theta_hat)
        let cases = [
            (theta_section(FRAC_PI_4, PI), FRAC_PI_4 + off, -1.0), // concave start cone
            (theta_section(0.75 * PI, PI), 0.75 * PI + off, -1.0), // convex start cone
            (theta_section(0.0, FRAC_PI_4), FRAC_PI_4 - off, 1.0), // convex end cone
            (theta_section(0.0, 0.75 * PI), 0.75 * PI - off, 1.0), // concave end cone
        ];
        for (s, theta, sign) in cases {
            assert_eq!(0.5 * solid(&s).rmax_tol, h);
            let t = solid(&s).theta;
            let cone = if sign < 0.0 { t.start } else { t.end };
            let p = point_at(&s, 5.0, theta, 0.0);
            let (sin, cos) = cone.sin_cos();
            let horizontal = (p.x - p.z * sin / cos).abs();
            assert!(horizontal > h, "{cone}: horizontal distance {horizontal}");
            let (_, theta_hat, _) = spherical_basis(theta, 0.0);
            let v = Direction::new(theta_hat * sign).unwrap();
            assert_eq!(s.inside(p), Inside::Inside, "{cone}");
            assert_eq!(d(&s, p, v), 0.0, "{cone}");
        }
    }

    /// Deviation 7, root behind the start. A Surface start 3e-7 nm outside a start cone
    /// at pi/2 - 1e-4 (r = 1e3 nm, 3e-10 rad, inside the angular shell) moving out along
    /// `-theta_hat` crossed the cone 3e-7 nm behind it. The horizontal distance
    /// 3e-7 / cos(theta) = 3e-3 nm exceeds h, so Geant4's early return misses it, and the
    /// root behind the start is skipped: Geant4 returns infinity. The port leaves at once.
    #[test]
    fn surface_start_with_cone_root_behind_moving_out_leaves_at_once() {
        let cone = FRAC_PI_2 - 1e-4;
        let s = sphere(0.0, 2.0e3, 0.0, TAU, cone, PI - cone);
        let theta = cone - 3.0e-10;
        let p = point_at(&s, 1.0e3, theta, 0.0);
        assert_eq!(s.inside(p), Inside::Surface);
        let (sin, cos) = cone.sin_cos();
        assert!((p.x - p.z * sin / cos).abs() > 0.5 * RAD_TOLERANCE);
        let (_, theta_hat, _) = spherical_basis(theta, 0.0);
        let v = Direction::new(-theta_hat).unwrap();
        assert_eq!(d(&s, p, v), 0.0);
    }

    /// The sign test of deviation 7. A Surface start 1e-9 nm outside the start cone (within
    /// its angular shell, 2e-10 rad at r = 5 nm) moving in obliquely meets the cone 1e-9 nm
    /// ahead, entering the solid. That root is within h but the start is not leaving, so the
    /// step runs on to rmax, sqrt(75) nm away (v is orthogonal to r_hat).
    #[test]
    fn surface_start_just_outside_a_cone_moving_in_crosses_the_solid() {
        let s = theta_section(FRAC_PI_4, PI);
        let theta = FRAC_PI_4 - 0.4 * HAT;
        let p = point_at(&s, 5.0, theta, 0.0);
        let (_, theta_hat, phi_hat) = spherical_basis(theta, 0.0);
        let v = Direction::new(theta_hat * 0.8 + phi_hat * 0.6).unwrap();
        assert_eq!(s.inside(p), Inside::Surface);
        assert_relative_eq!(d(&s, p, v), 75f64.sqrt(), max_relative = 1e-12);
    }

    /// Deviation 5. A lower hemisphere of radius 1e7 nm: 5e6 nm from the axis the angular
    /// shell of the plane is 2.5e-3 nm, so a start 2e-3 nm above it is Surface. Moving up it
    /// leaves at once; G4Sphere.cc:1876 returns -p.z/v.z = -2e-3 nm.
    #[test]
    fn plane_root_behind_the_start_of_a_large_hemisphere() {
        let s = sphere(0.0, 1.0e7, 0.0, TAU, FRAC_PI_2, FRAC_PI_2);
        let p = v3(5.0e6, 0.0, 2.0e-3);
        assert_eq!(s.inside(p), Inside::Surface);
        assert_eq!(d(&s, p, dir(0.0, 0.0, 1.0)), 0.0);
        assert_eq!(d(&s, p, dir(0.3, 0.0, 1.0)), 0.0);
    }

    /// Deviation 6. Starts at the centre, the apex of every cone: the direction decides.
    /// G4Sphere gives 0 for any direction with a convex cone and ignores a concave one.
    #[test]
    fn start_at_the_apex() {
        let at = Vec3::ZERO;
        // Convex end cone (polar cap [0, pi/4]).
        let s = theta_section(0.0, FRAC_PI_4);
        assert_eq!(s.inside(at), Inside::Surface);
        assert_eq!(d(&s, at, dir(0.0, 0.0, 1.0)), 10.0);
        assert_eq!(d(&s, at, dir(1.0, 0.0, 1.0)), 10.0); // along the cone
        assert_eq!(d(&s, at, dir(1.0, 0.0, 0.0)), 0.0);
        // Concave start cone ([pi/4, pi]).
        let s = theta_section(FRAC_PI_4, PI);
        assert_eq!(d(&s, at, dir(0.0, 0.0, 1.0)), 0.0);
        assert_eq!(d(&s, at, dir(0.0, 1.0, 0.0)), 10.0);
        // Convex start cone ([3 pi/4, pi]).
        let s = theta_section(0.75 * PI, PI);
        assert_eq!(d(&s, at, dir(0.0, 0.0, -1.0)), 10.0);
        assert_eq!(d(&s, at, dir(1.0, 0.0, 0.0)), 0.0);
        // Planes: as Geant4.
        let s = theta_section(0.0, FRAC_PI_2);
        assert_eq!(d(&s, at, dir(0.0, 0.0, -1.0)), 0.0);
        assert_eq!(d(&s, at, dir(0.0, 1.0, 0.0)), 10.0);
        // With a phi segment the phi test applies too.
        let s = sphere(0.0, 10.0, 0.0, FRAC_PI_2, 0.0, FRAC_PI_4);
        assert_eq!(d(&s, at, dir(0.1, 0.1, 1.0)), 10.0);
        assert_eq!(d(&s, at, dir(-0.1, 0.1, 1.0)), 0.0);
    }

    /// A Geant4 property: 2e-9 rad from the start cone at r = 1 nm, the point is Inside by
    /// angle but 2e-9 nm from the cone, within halfRmaxTolerance, so moving out it gets 0.
    #[test]
    fn inside_start_within_the_cartesian_tolerance_of_a_cone_leaves_at_once() {
        let s = theta_section(FRAC_PI_4, PI);
        let p = point_at(&s, 1.0, FRAC_PI_4 + 4.0 * HAT, 0.0);
        assert_eq!(s.inside(p), Inside::Inside);
        assert_eq!(d(&s, p, dir(-1.0, 0.0, 0.0)), 0.0);
    }

    /// The case proptest saved in `proptest-regressions/geometry/sphere.txt` (10M-case run):
    /// a start built on the start phi plane 0.033 nm from the z axis, 1e6 nm from the
    /// origin. It is 1.8e-11 nm from the plane, but the rounding of its coordinates (ulp
    /// 1.2e-10 nm) puts its angle 5.4e-10 rad from the plane, beyond kAngTolerance/2, so
    /// `Inside` gives Inside, not Surface (the angular-shell property of the module
    /// documentation). Moving out through the plane, it still leaves at once.
    #[test]
    fn near_axis_start_on_a_phi_plane_is_inside_by_angle() {
        let (rmax, start) = (40.69153587285085, 5.544447637890824);
        let c = v3(-630240.8172622061, -845922.9625205244, 0.0);
        let volume = Volume::sphere(c, 0.0, rmax, start, 0.2, 0.0, 0.2).unwrap();
        let (r, theta) = (0.16549048761814206 * rmax, 0.024506962022441757 * 0.2);
        let p = point_at(&volume, r, theta, start);
        let (r_hat, theta_hat, phi_hat) = spherical_basis(theta, start);
        let v = leaving(phi_hat * -1.0, r_hat, theta_hat, 0.1, 0.0);
        let q = p - c;
        assert!(q.x.hypot(q.y) < 0.034, "{q:?}");
        let (to_start, _) = half_plane_distances(solid(&volume), q);
        assert!(to_start < 1e-10, "{to_start} nm from the start half-plane");
        let offset = q.y.atan2(q.x) + TAU - start;
        assert!(offset > HAT, "angle {offset} rad from the start plane");
        assert_eq!(volume.inside(p), Inside::Inside);
        assert_eq!(volume.distance_to_out(p, v), Some(0.0));
    }

    /// The test helper `angular_excess` moves phi into [start, start + 2 pi): for the segment
    /// [-4.51, 0.27] (opening above pi), phi = 2.28 is inside as 2.28 - 2 pi. A first
    /// version only moved phi up and flagged this exit of a 1M-case run as outside.
    #[test]
    fn angular_excess_wraps_phi_into_the_segment() {
        let s = sphere(0.0, 1.0, -4.510949176316897, 4.777428006979739, 0.0, PI);
        let q = v3(
            -0.049959138153072874,
            0.058338798407838136,
            0.9970459714152262,
        );
        assert_eq!(angular_excess(solid(&s), q), 0.0);
        assert_eq!(s.inside(q), Inside::Surface);
        let lower = v3(-0.5, -0.25, 0.0); // phi = -2.68, inside without moving
        assert_eq!(angular_excess(solid(&s), lower), 0.0);
        let beyond = v3(0.5, 0.5, 0.0); // phi = pi/4, past the end 0.27
        assert!(angular_excess(solid(&s), beyond) > 0.1);
    }

    // --- invariants ---

    /// Distance (nm) from the z axis for phi planes, and from the centre for theta cones,
    /// within which a point on them need not classify as Surface: coordinates up to 2e6 nm
    /// round to about 2e-10 nm, which is `kAngTolerance/2` at 0.4 nm. Exits there are checked
    /// against the Cartesian tolerance instead, and generated boundary starts avoid the zone.
    const AXIS_ZONE: f64 = 10.0;

    /// Smallest Cartesian distance (nm) of a generated start from the phi planes and theta
    /// cones, 20 half surface thicknesses, so that the start is not on them in the
    /// Cartesian sense (see the module documentation), where 0 is a correct result. Starts
    /// within that tolerance, the band of deviation 7, are built on purpose in
    /// `start_near_a_theta_cone_moving_out_stops_at_it`: random starts would reach a band
    /// 1e-4 nm wide almost never.
    const EDGE_CLEARANCE: f64 = 0.01;

    /// Openings of the generated phi segments, rad.
    const SEGMENT: std::ops::Range<f64> = 0.2..(TAU - 0.01);

    /// Theta ranges with at least one cone or plane, built without rejection: the two
    /// hemispheres (planes at pi/2), a start of 0 with an opening in [0.2, pi), or a start in
    /// [0.05, pi - 0.25) with an opening in [0.2, pi - start) or reaching pi.
    fn theta_section_range() -> impl Strategy<Value = (f64, f64)> {
        let from_pole = (0.2..PI).prop_map(|delta| (0.0, delta));
        let from_cone = (0.05..(PI - 0.25)).prop_flat_map(|start| {
            (
                Just(start),
                prop_oneof![1 => Just(PI), 3 => 0.2..(PI - start)],
            )
        });
        prop_oneof![
            1 => Just((0.0, FRAC_PI_2)),
            1 => Just((FRAC_PI_2, FRAC_PI_2)),
            2 => from_pole,
            4 => from_cone,
        ]
    }

    /// Theta ranges: the full range (one case in eight) or a [`theta_section_range`].
    fn theta_range() -> impl Strategy<Value = (f64, f64)> {
        prop_oneof![1 => Just((0.0, PI)), 7 => theta_section_range()]
    }

    /// Sphere section with rmax in [1, 1e6) nm, `rmin = rmin_fraction * rmax`, a phi opening
    /// `delta_phi` from a start in [-4 pi, 4 pi), and a theta range.
    fn sphere_with(
        rmin_fraction: impl Strategy<Value = f64>,
        delta_phi: impl Strategy<Value = f64>,
        theta: impl Strategy<Value = (f64, f64)>,
    ) -> impl Strategy<Value = Volume> {
        (
            centre(),
            size(),
            rmin_fraction,
            -2.0 * TAU..2.0 * TAU,
            delta_phi,
            theta,
        )
            .prop_map(|(c, rmax, f, sphi, dphi, (st, dt))| {
                Volume::sphere(c, f * rmax, rmax, sphi, dphi, st, dt).unwrap()
            })
    }

    /// rmin = 0 (one case in four) or up to 0.9 rmax, a full circle (one case in four) or a
    /// [`SEGMENT`] in phi, and any [`theta_range`].
    fn sphere_strategy() -> impl Strategy<Value = Volume> {
        sphere_with(
            prop_oneof![1 => Just(0.0), 3 => 0.0..0.9],
            prop_oneof![1 => Just(TAU), 3 => SEGMENT],
            theta_range(),
        )
    }

    /// Point at radius fraction `fr` of [rmin, rmax], theta fraction `ft` of the theta range
    /// and phi fraction `fphi` of the requested phi range.
    fn point(volume: &Volume, fr: f64, ft: f64, fphi: f64) -> Vec3 {
        let s = solid(volume);
        let r = s.rmin() + fr * (s.rmax() - s.rmin());
        let theta = s.start_theta() + ft * s.delta_theta();
        let phi = s.requested_start_phi() + fphi * s.requested_delta_phi();
        point_at(volume, r, theta, phi)
    }

    /// Unit vectors `(r, theta, phi)` of the spherical basis at `(theta, phi)`.
    fn spherical_basis(theta: f64, phi: f64) -> (Vec3, Vec3, Vec3) {
        let (st, ct) = theta.sin_cos();
        let (sp, cp) = phi.sin_cos();
        (
            v3(st * cp, st * sp, ct),
            v3(ct * cp, ct * sp, -st),
            v3(-sp, cp, 0.0),
        )
    }

    /// Unit vector with component `a` along the outward normal `n`; the rest is split
    /// between `u` and `w` (orthonormal, both orthogonal to `n`) by the angle `psi`.
    fn leaving(n: Vec3, u: Vec3, w: Vec3, a: f64, psi: f64) -> Direction {
        let b = (1.0 - a * a).max(0.0).sqrt();
        Direction::new(n * a + (u * psi.cos() + w * psi.sin()) * b).unwrap()
    }

    /// Cartesian distances (nm) of the local point `q` from the start and end phi
    /// half-planes, infinite for a full circle. A point on the far side of the z axis from a
    /// half-plane (by more than kCarTolerance along it) is infinitely far from it.
    fn half_plane_distances(s: &SphereSolid, q: Vec3) -> (f64, f64) {
        if s.phi.is_full() {
            return (f64::INFINITY, f64::INFINITY);
        }
        let distance = |phi: f64| {
            let (sin, cos) = phi.sin_cos();
            if q.x * cos + q.y * sin >= -CAR_TOLERANCE {
                (q.x * sin - q.y * cos).abs()
            } else {
                f64::INFINITY
            }
        };
        (distance(s.start_phi()), distance(s.phi.end()))
    }

    /// Cartesian distances (nm) of the local point `q` from the start and end theta cones
    /// (each on its own nappe; infinite where there is no cone).
    fn cone_distances(s: &SphereSolid, q: Vec3) -> (f64, f64) {
        let r = q.norm();
        let theta = q.x.hypot(q.y).atan2(q.z);
        let distance = |cone: f64| {
            let off = (theta - cone).abs();
            if off < FRAC_PI_2 { r * off.sin() } else { r }
        };
        let t = s.theta;
        let start = if t.start > 0.0 {
            distance(t.start)
        } else {
            f64::INFINITY
        };
        let end = if t.end < PI {
            distance(t.end)
        } else {
            f64::INFINITY
        };
        (start, end)
    }

    /// Smallest Cartesian distance (nm) of `q` from the phi planes and theta cones.
    fn edge_distance(s: &SphereSolid, q: Vec3) -> f64 {
        let (a, b) = half_plane_distances(s, q);
        let (c, d) = cone_distances(s, q);
        a.min(b).min(c).min(d)
    }

    /// Cartesian distance (nm) by which the local point `q` lies outside the phi wedge and
    /// the theta range (0 inside both): to the nearer phi half-plane or the z axis, and to
    /// the nearer cone or the centre.
    fn angular_excess(s: &SphereSolid, q: Vec3) -> f64 {
        let mut excess: f64 = 0.0;
        let rho = q.x.hypot(q.y);
        if !s.phi.is_full() && rho > 0.0 {
            // phi in [start, start + 2 pi), then compared with the end.
            let mut phi = q.y.atan2(q.x);
            while phi < s.start_phi() {
                phi += TAU;
            }
            while phi >= s.start_phi() + TAU {
                phi -= TAU;
            }
            if phi > s.phi.end() {
                let (a, b) = half_plane_distances(s, q);
                excess = excess.max(a.min(b).min(rho));
            }
        }
        if !s.theta.full {
            let t = s.theta;
            let theta = rho.atan2(q.z);
            let off = (t.start - theta).max(theta - t.end).max(0.0);
            let r = q.norm();
            excess = excess.max(if off < FRAC_PI_2 { r * off.sin() } else { r });
        }
        excess
    }

    /// True if the local point `q` is within the Cartesian tolerance of the solid: inside
    /// the tolerant radial shells and at most kCarTolerance/2 outside the angular ranges.
    fn within_tolerance(s: &SphereSolid, q: Vec3) -> bool {
        let r = q.norm();
        r <= s.rmax() + 0.5 * s.rmax_tol
            && (s.rmin() == 0.0 || r >= s.rmin() - 0.5 * s.rmin_tol)
            && angular_excess(s, q) <= HALF_CAR_TOLERANCE
    }

    proptest! {
        #[test]
        fn exit_point_is_on_the_surface(
            volume in sphere_strategy(),
            fr in 0.05..0.95f64,
            ft in 0.02..0.98f64,
            fphi in 0.02..0.98f64,
            v in unit_direction(),
        ) {
            let p = point(&volume, fr, ft, fphi);
            let s = solid(&volume);
            prop_assume!(volume.inside(p) == Inside::Inside);
            prop_assume!(edge_distance(s, p - volume.centre()) > EDGE_CLEARANCE);
            let d = volume.distance_to_out(p, v).unwrap();
            let exit = p + v.as_vec3() * d;
            let q = exit - volume.centre();
            let near_axis = !s.phi.is_full() && q.x.hypot(q.y) < AXIS_ZONE;
            let near_apex = !s.theta.full && q.norm() < AXIS_ZONE;
            if near_axis || near_apex {
                // The angular shells there are thinner than the rounding of the exit point,
                // which may then classify as Outside, as in Geant4. Require it to be within
                // the Cartesian tolerance of the solid and on a boundary instead.
                prop_assert!(d > 0.0 && d.is_finite());
                prop_assert!(
                    within_tolerance(s, q),
                    "near-axis exit {:?} is outside the solid by {} nm in angle",
                    q, angular_excess(s, q)
                );
                let r = q.norm();
                let on_rmax = (r - s.rmax()).abs() <= 0.5 * s.rmax_tol;
                let on_rmin = s.rmin() > 0.0 && (r - s.rmin()).abs() <= 0.5 * s.rmin_tol;
                let edge = edge_distance(s, q);
                prop_assert!(
                    on_rmax || on_rmin || edge <= HALF_CAR_TOLERANCE
                        || volume.inside(exit) == Inside::Surface,
                    "near-axis exit {:?} is {} nm from the phi planes and theta cones",
                    q, edge
                );
            } else {
                check_exit_invariants(&volume, p, v)?;
            }
            prop_assert!(d <= 2.0 * s.rmax() * (1.0 + 1e-12), "{} > 2 rmax", d);
            // The step never passes more than kCarTolerance/2 outside the solid: a missed exit
            // followed by re-entry would leave sampled points beyond it. Rounding may make a
            // point Outside by angle, so Outside points must be within that tolerance.
            for k in 1..10 {
                let x = p + v.as_vec3() * (d * f64::from(k) / 10.0);
                prop_assert!(
                    volume.inside(x) != Inside::Outside
                        || within_tolerance(s, x - volume.centre()),
                    "point {:?} at {}/10 of the step {} is Outside",
                    x - volume.centre(), k, d
                );
            }
        }

        /// A start `f h` inside a theta cone or plane (h = halfRmaxTolerance, f in
        /// [0.05, 0.99]) moving out with a normal component `a` stops at the cone: within
        /// `delta/a` (plus 50% for the curvature of the cone over that path), whether by the
        /// on-cone test, deviation 7 or the root itself.
        #[test]
        fn start_near_a_theta_cone_moving_out_stops_at_it(
            volume in sphere_with(
                prop_oneof![1 => Just(0.0), 3 => 0.0..0.9],
                prop_oneof![Just(TAU), SEGMENT],
                theta_section_range(),
            ),
            fr in 0.05..0.95f64,
            fphi in 0.02..0.98f64,
            end_cone in any::<bool>(),
            f in 0.05..0.99f64,
            a in 0.1..=1.0f64,
            psi in 0.0..TAU,
        ) {
            let s = solid(&volume);
            let t = s.theta;
            let use_end = if t.start > 0.0 && t.end < PI { end_cone } else { t.end < PI };
            let r = s.rmin() + fr * (s.rmax() - s.rmin());
            prop_assume!(r > AXIS_ZONE);
            let delta = f * 0.5 * s.rmax_tol;
            let theta = if use_end { t.end - delta / r } else { t.start + delta / r };
            let phi = s.requested_start_phi() + fphi * s.requested_delta_phi();
            let p = point_at(&volume, r, theta, phi);
            let (r_hat, theta_hat, phi_hat) = spherical_basis(theta, phi);
            let n = if use_end { theta_hat } else { theta_hat * -1.0 };
            let v = leaving(n, r_hat, phi_hat, a, psi);
            prop_assert_ne!(volume.inside(p), Inside::Outside);
            let d = volume.distance_to_out(p, v).unwrap();
            prop_assert!(d <= 1.5 * delta / a, "{} nm for a start {} nm from the cone", d, delta);
        }

        /// d(p, v) + d(p, -v) is the chord 2 sqrt(R^2 - b^2) through p.
        #[test]
        fn chord_through_a_full_sphere(
            c in centre(),
            rmax in size(),
            f in prop::array::uniform3(-0.55..0.55f64),
            v in unit_direction(),
        ) {
            let volume = Volume::sphere(c, 0.0, rmax, 0.0, TAU, 0.0, PI).unwrap();
            let p = c + v3(f[0], f[1], f[2]) * rmax;
            let back = Direction::new(v.as_vec3() * -1.0).unwrap();
            let q = p - c;
            let b2 = q.cross(v.as_vec3()).dot(q.cross(v.as_vec3()));
            let chord = volume.distance_to_out(p, v).unwrap()
                + volume.distance_to_out(p, back).unwrap();
            let expected = 2.0 * (rmax * rmax - b2).sqrt();
            prop_assert!(
                (chord - expected).abs() <= 1e-9 * rmax,
                "chord {} vs {}", chord, expected
            );
        }

        /// The outward radial component is at least 0.1.
        #[test]
        fn leaving_through_rmax_gives_zero(
            volume in sphere_strategy(),
            ft in 0.02..0.98f64,
            fphi in 0.02..0.98f64,
            a in 0.1..=1.0f64,
            psi in 0.0..TAU,
        ) {
            let s = solid(&volume);
            let theta = s.start_theta() + ft * s.delta_theta();
            let phi = s.requested_start_phi() + fphi * s.requested_delta_phi();
            let p = point_at(&volume, s.rmax(), theta, phi);
            let (r_hat, theta_hat, phi_hat) = spherical_basis(theta, phi);
            let v = leaving(r_hat, theta_hat, phi_hat, a, psi);
            prop_assert_eq!(volume.inside(p), Inside::Surface);
            prop_assert_eq!(volume.distance_to_out(p, v), Some(0.0));
        }

        /// rmin is at least 0.1 rmax and the inward radial component at least 0.2, so that
        /// the ray is not tangent to rmin within fRminTolerance (G4Sphere.cc:1814).
        #[test]
        fn leaving_through_rmin_gives_zero(
            volume in sphere_with(
                0.1..0.9,
                prop_oneof![Just(TAU), SEGMENT],
                theta_range(),
            ),
            ft in 0.02..0.98f64,
            fphi in 0.02..0.98f64,
            a in 0.2..=1.0f64,
            psi in 0.0..TAU,
        ) {
            let s = solid(&volume);
            let theta = s.start_theta() + ft * s.delta_theta();
            let phi = s.requested_start_phi() + fphi * s.requested_delta_phi();
            let p = point_at(&volume, s.rmin(), theta, phi);
            let (r_hat, theta_hat, phi_hat) = spherical_basis(theta, phi);
            let v = leaving(r_hat * -1.0, theta_hat, phi_hat, a, psi);
            prop_assert_eq!(volume.inside(p), Inside::Surface);
            prop_assert_eq!(volume.distance_to_out(p, v), Some(0.0));
        }

        /// A start on the start or end phi plane, with an outward normal component of at
        /// least 0.1 so that the plane-crossing distance `pDist/comp` stays within tolerance.
        #[test]
        fn leaving_through_a_phi_plane_gives_zero(
            volume in sphere_with(prop_oneof![1 => Just(0.0), 3 => 0.0..0.9], SEGMENT, theta_range()),
            fr in 0.05..0.95f64,
            ft in 0.02..0.98f64,
            end_plane in any::<bool>(),
            a in 0.1..=1.0f64,
            psi in 0.0..TAU,
        ) {
            let s = solid(&volume);
            let phi = s.requested_start_phi()
                + if end_plane { s.requested_delta_phi() } else { 0.0 };
            let r = s.rmin() + fr * (s.rmax() - s.rmin());
            let theta = s.start_theta() + ft * s.delta_theta();
            // Nearer the axis the rounding of p can exceed the angular shell (see
            // near_axis_start_on_a_phi_plane_is_inside_by_angle).
            prop_assume!(r * theta.sin() > AXIS_ZONE);
            let p = point_at(&volume, r, theta, phi);
            let (r_hat, theta_hat, phi_hat) = spherical_basis(theta, phi);
            // Outward normals: -phi_hat at the start plane, +phi_hat at the end plane.
            let n = if end_plane { phi_hat } else { phi_hat * -1.0 };
            let v = leaving(n, r_hat, theta_hat, a, psi);
            prop_assert_eq!(volume.inside(p), Inside::Surface);
            prop_assert_eq!(volume.distance_to_out(p, v), Some(0.0));
        }

        /// A start on the start or end theta cone (or plane), with an outward normal
        /// component of at least 0.1: `-theta_hat` at the start cone, `+theta_hat` at the end.
        #[test]
        fn leaving_through_a_theta_cone_gives_zero(
            volume in sphere_with(
                prop_oneof![1 => Just(0.0), 3 => 0.0..0.9],
                prop_oneof![Just(TAU), SEGMENT],
                theta_section_range(),
            ),
            fr in 0.05..0.95f64,
            fphi in 0.02..0.98f64,
            end_cone in any::<bool>(),
            a in 0.1..=1.0f64,
            psi in 0.0..TAU,
        ) {
            let s = solid(&volume);
            let t = s.theta;
            let use_end = if t.start > 0.0 && t.end < PI { end_cone } else { t.end < PI };
            let theta = if use_end { t.end } else { t.start };
            let phi = s.requested_start_phi() + fphi * s.requested_delta_phi();
            let r = s.rmin() + fr * (s.rmax() - s.rmin());
            // Nearer the apex the rounding of p can exceed the angular shell.
            prop_assume!(r > AXIS_ZONE);
            let p = point_at(&volume, r, theta, phi);
            let (r_hat, theta_hat, phi_hat) = spherical_basis(theta, phi);
            let n = if use_end { theta_hat } else { theta_hat * -1.0 };
            let v = leaving(n, r_hat, phi_hat, a, psi);
            prop_assert_eq!(volume.inside(p), Inside::Surface);
            prop_assert_eq!(volume.distance_to_out(p, v), Some(0.0));
        }

        #[test]
        fn serde_round_trip_preserves_the_sphere(volume in sphere_strategy()) {
            let text = toml::to_string(&volume).unwrap();
            let back: Volume = toml::from_str(&text).unwrap();
            prop_assert_eq!(back, volume);
        }
    }

    // --- safety (`G4Sphere::DistanceToOut(p)`) ---

    #[test]
    fn safety_takes_the_nearest_shell_plane_or_cone() {
        let full = Volume::sphere(Vec3::ZERO, 0.0, 10.0, 0.0, TAU, 0.0, PI).unwrap();
        assert_relative_eq!(full.safety(v3(0.0, 0.0, 3.0)).unwrap(), 7.0);
        assert_relative_eq!(full.safety(Vec3::ZERO).unwrap(), 10.0);
        let shell = Volume::sphere(Vec3::ZERO, 4.0, 10.0, 0.0, TAU, 0.0, PI).unwrap();
        assert_relative_eq!(shell.safety(v3(5.0, 0.0, 0.0)).unwrap(), 1.0);
        // Cone of half-angle pi/4: the distance to the cone line is |x - z| / sqrt 2.
        let cap = Volume::sphere(Vec3::ZERO, 0.0, 10.0, 0.0, TAU, 0.0, FRAC_PI_4).unwrap();
        assert_relative_eq!(
            cap.safety(v3(1.0, 0.0, 5.0)).unwrap(),
            2.0 * 2f64.sqrt(),
            epsilon = 1e-12
        );
        // At the apex of a theta-cut sphere and on the axis of a phi cut, Geant4 gives 0.
        assert_eq!(cap.safety(Vec3::ZERO), Some(0.0));
        let wedge = Volume::sphere(Vec3::ZERO, 0.0, 10.0, 0.0, FRAC_PI_2, 0.0, PI).unwrap();
        assert_eq!(wedge.safety(v3(0.0, 0.0, 2.0)), Some(0.0));
        assert_relative_eq!(wedge.safety(v3(3.0, 1.0, 0.0)).unwrap(), 1.0);
        assert_eq!(full.safety(v3(11.0, 0.0, 0.0)), None);
    }

    proptest! {
        #[test]
        fn safety_never_exceeds_the_exit_distance(
            volume in sphere_strategy(),
            fr in 0.05..0.95f64,
            ft in 0.02..0.98f64,
            fphi in 0.02..0.98f64,
            v in unit_direction(),
        ) {
            let p = point(&volume, fr, ft, fphi);
            prop_assume!(volume.inside(p) == Inside::Inside);
            let safety = volume.safety(p).unwrap();
            let d = volume.distance_to_out(p, v).unwrap();
            prop_assert!(safety >= 0.0);
            prop_assert!(safety <= d + CAR_TOLERANCE, "safety {} > exit {}", safety, d);
        }
    }
}
