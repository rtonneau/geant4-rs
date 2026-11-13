//! Primitive solids with Geant4's tolerant surface: an inside test and the distance to exit
//! along a ray.
//!
//! A [`Volume`] is one solid placed by a translation of its centre (no rotation). Lengths are
//! nm and directions are dimensionless unit vectors ([`Direction`]). The algorithms follow the
//! Geant4 11.4.1 CSG solids (`source/geometry/solids/CSG/src/`), with the line citations given
//! on each shape. Only `Inside(p)` and `DistanceToOut(p, v)` are ported: there is no
//! `DistanceToIn`, no safety distance, no surface normal and no navigation.
//!
//! # Tolerant surface
//!
//! Geant4 treats a solid's surface as a shell of thickness [`CAR_TOLERANCE`] centred on the
//! geometric boundary. A point within [`HALF_CAR_TOLERANCE`] of the boundary is
//! [`Inside::Surface`]; deeper points are [`Inside::Inside`], farther ones
//! [`Inside::Outside`]. Geant4 can rescale the tolerance from the world extent
//! (`G4GeometryTolerance::SetSurfaceTolerance`, `G4GeometryTolerance.cc:92-107`); this module
//! uses the default value only.
//!
//! # Deviations from Geant4
//!
//! - `distance_to_out` returns `None` for a start point that classifies as
//!   [`Inside::Outside`]. Geant4 does not check this case and returns an unspecified distance.
//! - A point with a NaN coordinate classifies as [`Inside::Outside`] (Geant4 behaviour is
//!   undefined for NaN).

mod box_solid;

pub use box_solid::BoxSolid;

use crate::vec3::Vec3;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Geant4's Cartesian surface tolerance `kCarTolerance`, in nm.
///
/// Geant4 sets `fCarTolerance = 1E-9 * mm` (`source/global/management/src/
/// G4GeometryTolerance.cc:48`), copied into each solid's `kCarTolerance`
/// (`source/geometry/management/src/G4VSolid.cc:62`). 1e-9 mm = 1e-3 nm.
pub const CAR_TOLERANCE: f64 = 1.0e-3;

/// Half the surface thickness, `delta = 0.5*kCarTolerance` (`G4Box.cc:64`), in nm.
pub const HALF_CAR_TOLERANCE: f64 = 0.5 * CAR_TOLERANCE;

/// Smallest accepted half-length, `2*kCarTolerance` (`G4Box.cc:65-73`), in nm: a thinner
/// solid would be no thicker than its own surface.
pub const MIN_HALF_LENGTH: f64 = 2.0 * CAR_TOLERANCE;

/// Classification of a point with respect to a solid (Geant4 `EInside`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inside {
    /// Farther than [`HALF_CAR_TOLERANCE`] inside the boundary.
    Inside,
    /// Within [`HALF_CAR_TOLERANCE`] of the boundary.
    Surface,
    /// Farther than [`HALF_CAR_TOLERANCE`] outside the boundary.
    Outside,
}

/// Invalid geometry parameters.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum GeometryError {
    /// A parameter is NaN or infinite.
    #[error("geometry parameter {field} must be finite, got {value}")]
    NonFinite { field: &'static str, value: f64 },
    /// A size is non-positive or below the Geant4 minimum (e.g. [`MIN_HALF_LENGTH`]).
    #[error("geometry parameter {field} = {value} nm is below the minimum {min} nm")]
    TooSmall {
        field: &'static str,
        value: f64,
        min: f64,
    },
    /// A direction is zero or not finite, so it has no unit vector.
    #[error("direction ({x}, {y}, {z}) is zero or not finite")]
    InvalidDirection { x: f64, y: f64, z: f64 },
}

/// A unit direction vector.
///
/// [`Direction::new`] normalises its input, so callers may pass any non-zero finite vector;
/// the zero vector and non-finite vectors are rejected. Taking a `Direction` rather than a
/// bare [`Vec3`] lets the ray methods rely on `|v| = 1` without checking it on every call.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction(Vec3);

impl Direction {
    /// Normalises `v`; fails for a zero or non-finite vector.
    pub fn new(v: Vec3) -> Result<Self, GeometryError> {
        v.normalised()
            .map(Self)
            .ok_or(GeometryError::InvalidDirection {
                x: v.x,
                y: v.y,
                z: v.z,
            })
    }

    /// The unit vector.
    pub fn as_vec3(self) -> Vec3 {
        self.0
    }
}

/// A primitive solid placed by a translation of its centre (no rotation).
///
/// Shapes are built with checked constructors (such as [`Volume::box_`]), and serde
/// deserialisation goes through the same validation. The serialised form is a table tagged
/// by `shape`, for example in TOML:
///
/// ```toml
/// shape = "box"
/// centre = [0.0, 0.0, 0.0]        # nm
/// half_lengths = [10.0, 10.0, 5.0] # nm
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawVolume", into = "RawVolume")]
pub enum Volume {
    /// Rectangular box, Geant4 `G4Box`.
    Box(BoxSolid),
}

impl Volume {
    /// Box of half-lengths `half_lengths` (nm) centred at `centre` (nm).
    ///
    /// Fails if a coordinate is not finite or a half-length is below [`MIN_HALF_LENGTH`]
    /// (which rejects zero and negative values).
    pub fn box_(centre: Vec3, half_lengths: Vec3) -> Result<Self, GeometryError> {
        BoxSolid::new(centre, half_lengths).map(Self::Box)
    }

    /// Centre of the solid, nm.
    pub fn centre(&self) -> Vec3 {
        match self {
            Self::Box(b) => b.centre(),
        }
    }

    /// Classifies `p` (nm) with Geant4's tolerant surface. A point with a NaN coordinate is
    /// [`Inside::Outside`].
    pub fn inside(&self, p: Vec3) -> Inside {
        if p.x.is_nan() || p.y.is_nan() || p.z.is_nan() {
            return Inside::Outside;
        }
        let q = p - self.centre();
        match self {
            Self::Box(b) => b.inside_local(q),
        }
    }

    /// Distance (nm) from `p` along `v` to the boundary, for a start that is not outside.
    ///
    /// Returns `None` if `p` classifies as [`Inside::Outside`], and `Some(0.0)` for a
    /// [`Inside::Surface`] start moving outward through the surface. A start on the surface
    /// moving inward or along it gets the distance to the far boundary. Otherwise follows
    /// Geant4 `DistanceToOut(p, v)` of the shape.
    pub fn distance_to_out(&self, p: Vec3, v: Direction) -> Option<f64> {
        if self.inside(p) == Inside::Outside {
            return None;
        }
        let q = p - self.centre();
        let v = v.as_vec3();
        Some(match self {
            Self::Box(b) => b.distance_to_out_local(q, v),
        })
    }
}

/// Serialised form of [`Volume`]; converting it back runs the checked constructors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case", deny_unknown_fields)]
enum RawVolume {
    Box {
        centre: [f64; 3],
        half_lengths: [f64; 3],
    },
}

fn to_array(v: Vec3) -> [f64; 3] {
    [v.x, v.y, v.z]
}

fn from_array(a: [f64; 3]) -> Vec3 {
    Vec3::new(a[0], a[1], a[2])
}

impl TryFrom<RawVolume> for Volume {
    type Error = GeometryError;
    fn try_from(raw: RawVolume) -> Result<Self, GeometryError> {
        match raw {
            RawVolume::Box {
                centre,
                half_lengths,
            } => Self::box_(from_array(centre), from_array(half_lengths)),
        }
    }
}

impl From<Volume> for RawVolume {
    fn from(volume: Volume) -> Self {
        match volume {
            Volume::Box(b) => Self::Box {
                centre: to_array(b.centre()),
                half_lengths: to_array(b.half_lengths()),
            },
        }
    }
}

/// Fails unless every component of `v` is finite; `field` names `v` in the error.
fn check_finite(field: &'static str, v: Vec3) -> Result<(), GeometryError> {
    for value in [v.x, v.y, v.z] {
        if !value.is_finite() {
            return Err(GeometryError::NonFinite { field, value });
        }
    }
    Ok(())
}

/// Fails unless the finite `value` is at least `min`.
fn check_at_least(field: &'static str, value: f64, min: f64) -> Result<(), GeometryError> {
    if !value.is_finite() {
        return Err(GeometryError::NonFinite { field, value });
    }
    if value < min {
        return Err(GeometryError::TooSmall { field, value, min });
    }
    Ok(())
}

/// Proptest generators and invariant checks shared by the shape tests.
#[cfg(test)]
mod test_support {
    use super::{Direction, Inside, Volume};
    use crate::vec3::Vec3;
    use proptest::prelude::*;
    use std::f64::consts::TAU;

    /// Largest generated size or centre offset, nm. Rounding of `p + d v` at this scale is
    /// about 1e-9 nm, far below the half surface thickness of 5e-4 nm.
    pub(super) const MAX_EXTENT: f64 = 1.0e6;

    /// Smallest generated size, nm: with [`FRACTION`] it keeps generated points at least
    /// 0.01 nm (20 half surface thicknesses) inside the boundary.
    pub(super) const MIN_SIZE: f64 = 1.0;

    /// Range of the relative coordinate of a generated inside point.
    pub(super) const FRACTION: std::ops::Range<f64> = -0.99..0.99;

    /// A size in `[MIN_SIZE, MAX_EXTENT)`, nm.
    pub(super) fn size() -> impl Strategy<Value = f64> {
        MIN_SIZE..MAX_EXTENT
    }

    /// A centre with each coordinate in `(-MAX_EXTENT, MAX_EXTENT)`, nm.
    pub(super) fn centre() -> impl Strategy<Value = Vec3> {
        let c = -MAX_EXTENT..MAX_EXTENT;
        (c.clone(), c.clone(), c).prop_map(|(x, y, z)| Vec3::new(x, y, z))
    }

    /// Three relative coordinates in [`FRACTION`].
    pub(super) fn fractions() -> impl Strategy<Value = Vec3> {
        (FRACTION, FRACTION, FRACTION).prop_map(|(x, y, z)| Vec3::new(x, y, z))
    }

    /// An isotropic unit direction, or (one case in four) a direction along a coordinate
    /// axis, so that rays parallel to faces are exercised.
    pub(super) fn unit_direction() -> impl Strategy<Value = Direction> {
        let isotropic = (-1.0f64..=1.0, 0.0..TAU).prop_map(|(cos_theta, phi)| {
            let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
            Vec3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta)
        });
        let axis = prop_oneof![
            Just(Vec3::new(1.0, 0.0, 0.0)),
            Just(Vec3::new(-1.0, 0.0, 0.0)),
            Just(Vec3::new(0.0, 1.0, 0.0)),
            Just(Vec3::new(0.0, -1.0, 0.0)),
            Just(Vec3::new(0.0, 0.0, 1.0)),
            Just(Vec3::new(0.0, 0.0, -1.0)),
        ];
        prop_oneof![3 => isotropic, 1 => axis]
            .prop_map(|v| Direction::new(v).expect("generated directions are non-zero"))
    }

    /// For an [`Inside::Inside`] start: the distance to exit is positive and finite, and the
    /// exit point classifies as [`Inside::Surface`].
    pub(super) fn check_exit_invariants(
        volume: &Volume,
        p: Vec3,
        v: Direction,
    ) -> Result<(), TestCaseError> {
        prop_assert_eq!(volume.inside(p), Inside::Inside, "start must be inside");
        let d = volume.distance_to_out(p, v);
        prop_assert!(d.is_some(), "inside start gave None");
        let d = d.unwrap();
        prop_assert!(d > 0.0 && d.is_finite(), "distance {} not positive", d);
        let exit = p + v.as_vec3() * d;
        prop_assert_eq!(
            volume.inside(exit),
            Inside::Surface,
            "exit point {:?}",
            exit
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::Length;
    use approx::assert_relative_eq;

    #[test]
    fn car_tolerance_is_geant4_default_in_nm() {
        assert_relative_eq!(
            CAR_TOLERANCE,
            Length::from_mm(1.0e-9).nm(),
            max_relative = 1e-15
        );
        assert_eq!(HALF_CAR_TOLERANCE, 5.0e-4);
        assert_eq!(MIN_HALF_LENGTH, 2.0e-3);
    }

    #[test]
    fn direction_is_normalised() {
        let v = Direction::new(Vec3::new(3.0, 0.0, 4.0)).unwrap().as_vec3();
        assert_relative_eq!(v.x, 0.6, epsilon = 1e-15);
        assert_relative_eq!(v.z, 0.8, epsilon = 1e-15);
        assert_relative_eq!(v.norm(), 1.0, epsilon = 1e-15);
    }

    #[test]
    fn direction_rejects_zero_and_non_finite_vectors() {
        assert!(matches!(
            Direction::new(Vec3::ZERO),
            Err(GeometryError::InvalidDirection { .. })
        ));
        assert!(Direction::new(Vec3::new(f64::NAN, 0.0, 1.0)).is_err());
        assert!(Direction::new(Vec3::new(f64::INFINITY, 0.0, 0.0)).is_err());
    }
}
