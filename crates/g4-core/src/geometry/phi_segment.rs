//! Azimuthal (phi) segment shared by the solids with a phi range: its Geant4 normalisation
//! and the trigonometry of its bounding half-planes.
//!
//! `G4Tubs` and `G4Sphere` normalise the phi range the same way (`G4Tubs.icc:111-161`,
//! `G4Sphere.icc:193-246`) and precompute the same sines and cosines (`G4Tubs.icc:94-109`,
//! `G4Sphere.icc:121-136`). Their `Inside` and `DistanceToOut` phi branches differ, so those
//! stay with each shape.

use super::{GeometryError, HALF_ANG_TOLERANCE};
use std::f64::consts::TAU;

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

    /// `(sin, cos)` of the centre angle `start + delta/2` (Geant4 `sinCPhi`, `cosCPhi`).
    pub(super) fn centre_trig(&self) -> (f64, f64) {
        (self.sin_centre, self.cos_centre)
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
