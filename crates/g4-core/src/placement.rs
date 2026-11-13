//! Physicochemical placement in a finite volume: the Geant4 `0.8 x safety` clamps
//! (docs/04 D.7, Geant4 11.4.1).
//!
//! Geant4 places a solvated electron and each dissociation product by asking the navigator for
//! the isotropic safety at the origin and shortening the displacement when it could reach the
//! boundary. The two call sites use different rules, both reproduced here:
//!
//! * [`ClampRule::Thermalisation`], `G4DNAOneStepThermalizationModel::SampleSecondaries`
//!   (`processes/electromagnetic/dna/models/include/G4DNAOneStepThermalizationModel.hpp:224-239`):
//!   if `safety <= |d|` the electron is placed at `origin + unit(d) * 0.8 * safety`, otherwise at
//!   `origin + d`.
//! * [`ClampRule::Dissociation`], `G4DNAMolecularDissociation::DecayIt`
//!   (`processes/electromagnetic/dna/processes/src/G4DNAMolecularDissociation.cc:203-222`): the
//!   length is always `min(0.8 * safety, |d|)` along `d / (|d| + 1e-30)`, so a displacement is also
//!   shortened when `0.8 * safety < |d| <= safety`.
//!
//! In a single water world the navigator safety is the world solid's `DistanceToOut(p)`, i.e.
//! [`Volume::safety`]. An origin that is Outside gets safety 0, as Geant4's `DistanceToOut(p)`
//! clamps a negative value to 0. The species is clamped, never killed. Because the safety is
//! isotropic, the clamp also shortens displacements that would stay inside (docs/09 R-B1); this is
//! Geant4 behaviour and is kept.
//!
//! Geant4 flags a dissociation product with `OUTSIDE_OF_MOTHER_VOLUME` when it is not strictly
//! Inside the solid, and only if the mother logical volume is not `G4_WATER` (`:236-257`); in a
//! world-only water geometry the warning never fires, but the test `!= kInside` is reported here
//! as [`Placement::inside`]. No random number is drawn.

use crate::geometry::{Inside, Volume};
use crate::vec3::Vec3;

/// Which Geant4 call site's clamp to apply (see the module documentation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClampRule {
    /// Solvated electron, `G4DNAOneStepThermalizationModel`.
    Thermalisation,
    /// Dissociation product, `G4DNAMolecularDissociation`.
    Dissociation,
}

/// Result of placing one species.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// Final position, nm.
    pub position: Vec3,
    /// True if the clamp shortened the displacement.
    pub shortened: bool,
    /// Classification of the final position.
    pub inside: Inside,
}

/// Places a species displaced by `displacement` (nm) from `origin` (nm) inside `volume`, with
/// the Geant4 clamp of `rule`. A zero displacement leaves the species at `origin` (Geant4's
/// thermalisation divides by `|d|` there and is undefined).
pub fn clamp_displacement(
    volume: &Volume,
    origin: Vec3,
    displacement: Vec3,
    rule: ClampRule,
) -> Placement {
    let safety = volume.safety(origin).unwrap_or(0.0);
    let mag = displacement.norm();
    let (position, shortened) = match rule {
        ClampRule::Thermalisation => {
            if mag == 0.0 {
                (origin, false)
            } else if safety <= mag {
                (origin + div(displacement, mag) * (safety * 0.80), true)
            } else {
                (origin + displacement, false)
            }
        }
        ClampRule::Dissociation => {
            let direction = div(displacement, mag + 1e-30);
            let clamped = (safety * 0.8).min(mag);
            (origin + direction * clamped, clamped < mag)
        }
    };
    Placement {
        position,
        shortened,
        inside: volume.inside(position),
    }
}

/// Component-wise `v / c`, as CLHEP `operator/(Hep3Vector, double)`.
fn div(v: Vec3, c: f64) -> Vec3 {
    Vec3::new(v.x / c, v.y / c, v.z / c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn sphere(r: f64) -> Volume {
        Volume::sphere(
            Vec3::ZERO,
            0.0,
            r,
            0.0,
            std::f64::consts::TAU,
            0.0,
            std::f64::consts::PI,
        )
        .unwrap()
    }

    #[test]
    fn thermalisation_shortens_to_eight_tenths_of_safety_when_safety_is_not_larger() {
        let v = sphere(10.0);
        let origin = Vec3::new(8.0, 0.0, 0.0); // safety 2
        let p = clamp_displacement(
            &v,
            origin,
            Vec3::new(3.0, 0.0, 0.0),
            ClampRule::Thermalisation,
        );
        assert!(p.shortened);
        assert_relative_eq!(p.position.x, 9.6);
        assert_eq!(p.inside, Inside::Inside);
        // safety == |d| also clamps (`safety <= displacementMag`).
        let p = clamp_displacement(
            &v,
            origin,
            Vec3::new(0.0, 2.0, 0.0),
            ClampRule::Thermalisation,
        );
        assert!(p.shortened);
        assert_relative_eq!(p.position.y, 1.6);
        // |d| < safety: unchanged, bit for bit.
        let d = Vec3::new(0.0, 0.0, -1.9);
        let p = clamp_displacement(&v, origin, d, ClampRule::Thermalisation);
        assert!(!p.shortened);
        assert_eq!(p.position, origin + d);
    }

    #[test]
    fn dissociation_uses_min_of_eight_tenths_safety_and_length() {
        let v = sphere(10.0);
        let origin = Vec3::new(8.0, 0.0, 0.0); // safety 2, 0.8 safety = 1.6
        // 1.6 < |d| = 1.9 <= safety: shortened although origin + d is inside.
        let d = Vec3::new(0.0, 1.9, 0.0);
        assert_eq!(v.inside(origin + d), Inside::Inside);
        let p = clamp_displacement(&v, origin, d, ClampRule::Dissociation);
        assert!(p.shortened);
        assert_relative_eq!(p.position.y, 1.6);
        // The same displacement is not shortened by the thermalisation rule.
        assert!(!clamp_displacement(&v, origin, d, ClampRule::Thermalisation).shortened);
        let p = clamp_displacement(
            &v,
            origin,
            Vec3::new(1.0, 0.0, 0.0),
            ClampRule::Dissociation,
        );
        assert!(!p.shortened);
        assert_relative_eq!(p.position.x, 9.0);
    }

    #[test]
    fn origin_on_or_outside_the_surface_stays_put() {
        let v = sphere(10.0);
        for origin in [Vec3::new(10.0, 0.0, 0.0), Vec3::new(12.0, 0.0, 0.0)] {
            for rule in [ClampRule::Thermalisation, ClampRule::Dissociation] {
                let p = clamp_displacement(&v, origin, Vec3::new(1.0, 0.0, 0.0), rule);
                assert!(p.shortened);
                assert_eq!(p.position, origin);
                assert_ne!(p.inside, Inside::Inside);
            }
        }
    }

    #[test]
    fn zero_displacement_is_not_shortened_inside() {
        let v = sphere(10.0);
        for rule in [ClampRule::Thermalisation, ClampRule::Dissociation] {
            let p = clamp_displacement(&v, Vec3::new(1.0, 2.0, 3.0), Vec3::ZERO, rule);
            assert!(!p.shortened);
            assert_eq!(p.position, Vec3::new(1.0, 2.0, 3.0));
        }
    }
}
