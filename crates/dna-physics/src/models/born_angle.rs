//! `G4DNABornAngle`: direction of the delta electron ejected by an electron.
//!
//! Geant4 reference: `dna/models/src/G4DNABornAngle.cc:67-109` (electron branch `:76-90`).
//!
//! With `W` the kinetic energy of the ejected electron and `k` that of the primary:
//!
//! | `W` | `cos(theta)` | uniform draws |
//! |---|---|---|
//! | `W < 50 eV` | `2u - 1` (isotropic) | 1 |
//! | `50 eV <= W <= 200 eV` | with probability 0.1 (`u1 <= 0.1`) isotropic `2u2 - 1`, else `u2 sqrt(2)/2` | 2 |
//! | `W > 200 eV` | `sqrt(1 - sin2)`, `sin2 = (1 - W/k) / (1 + W / (2 m_e c^2))` | 0 |
//!
//! then `phi = 2 pi u` (one more draw) and the direction `(sin theta cos phi, sin theta sin phi,
//! cos theta)` is rotated onto the primary direction with `rotateUz`. The angle is measured
//! from the **primary** direction in the laboratory frame, with no relation to the momentum
//! transferred; the proton branch (`:91-100`) is not needed in M2.

use crate::direction::direction_from_angles;
use crate::sampling::uniform;
use crate::water::ELECTRON_MASS_C2_EV;
use dna_core::units::Energy;
use dna_core::vec3::Vec3;
use rand::Rng;
use std::f64::consts::{PI, SQRT_2};

/// Polar-angle cosine of the ejected electron relative to the primary direction (electron
/// primary). Draws 1, 2 or 0 uniforms depending on `W`, as listed in the module table.
pub fn born_electron_cos_theta<R: Rng + ?Sized>(k: Energy, w: Energy, rng: &mut R) -> f64 {
    if w.ev() < 50.0 {
        2.0 * uniform(rng) - 1.0
    } else if w.ev() <= 200.0 {
        if uniform(rng) <= 0.1 {
            2.0 * uniform(rng) - 1.0
        } else {
            uniform(rng) * (SQRT_2 / 2.0)
        }
    } else {
        let sin2 = (1.0 - w.ev() / k.ev()) / (1.0 + w.ev() / (2.0 * ELECTRON_MASS_C2_EV));
        (1.0 - sin2).sqrt()
    }
}

/// Full `SampleDirectionForShell` for an electron primary: `cos(theta)`, then `phi`, then the
/// rotation onto `primary_direction` (a unit vector).
pub fn sample_born_electron_direction<R: Rng + ?Sized>(
    k: Energy,
    w: Energy,
    primary_direction: Vec3,
    rng: &mut R,
) -> Vec3 {
    let cos_theta = born_electron_cos_theta(k, w, rng);
    let phi = 2.0 * PI * uniform(rng);
    direction_from_angles(cos_theta, phi, primary_direction)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::ScriptedRng;
    use approx::assert_relative_eq;

    fn ev(x: f64) -> Energy {
        Energy::from_ev(x)
    }

    #[test]
    fn below_50_ev_is_isotropic_with_one_draw() {
        for (u, expect) in [(0.25, -0.5), (0.5, 0.0), (0.75, 0.5)] {
            let mut rng = ScriptedRng::new(&[u]);
            assert_relative_eq!(
                born_electron_cos_theta(ev(1000.0), ev(49.999), &mut rng),
                expect,
                epsilon = 1e-12
            );
            assert_eq!(rng.consumed(), 1);
        }
    }

    #[test]
    fn between_50_and_200_ev_mixes_isotropic_and_forward_cone() {
        // u1 <= 0.1 -> isotropic from the second draw
        let mut rng = ScriptedRng::new(&[0.05, 0.75]);
        assert_relative_eq!(
            born_electron_cos_theta(ev(1000.0), ev(50.0), &mut rng),
            0.5,
            epsilon = 1e-12
        );
        assert_eq!(rng.consumed(), 2);
        // u1 > 0.1 -> cos = u2 sqrt(2)/2 (never beyond 45 degrees)
        let mut rng = ScriptedRng::new(&[0.9, 0.5]);
        assert_relative_eq!(
            born_electron_cos_theta(ev(1000.0), ev(200.0), &mut rng),
            0.5 * SQRT_2 / 2.0,
            epsilon = 1e-12
        );
        assert_eq!(rng.consumed(), 2);
    }

    #[test]
    fn above_200_ev_follows_the_kinematic_formula_without_draws() {
        // k = 1000 eV, W = 300 eV: sin2 = 0.7 / (1 + 300/1021997.82)
        let (k, w) = (1000.0_f64, 300.0_f64);
        let sin2 = (1.0 - w / k) / (1.0 + w / (2.0 * 510_998.910));
        let expect = (1.0 - sin2).sqrt();
        let mut rng = ScriptedRng::new(&[]);
        assert_relative_eq!(
            born_electron_cos_theta(ev(k), ev(w), &mut rng),
            expect,
            epsilon = 1e-15
        );
        assert_eq!(rng.consumed(), 0);
        // by hand: 300/1021997.82 = 2.9353e-4; sin2 = 0.7/1.00029353 = 0.699795; cos = sqrt(0.300205)
        assert_relative_eq!(expect, 0.547_91, epsilon = 1e-5);
        // W -> k (hard collision): sin2 -> 0, cos -> 1
        let hard = born_electron_cos_theta(ev(1.0e5), ev(99_999.0), &mut ScriptedRng::new(&[]));
        assert!(hard > 0.06 && hard < 1.0, "{hard}");
        // W just above 200 eV at high k: nearly perpendicular? sin2 -> 1 => cos -> 0
        let soft = born_electron_cos_theta(ev(1.0e6), ev(200.000_1), &mut ScriptedRng::new(&[]));
        assert!(soft < 0.03, "{soft}");
    }

    #[test]
    fn the_boundaries_belong_to_the_documented_branches() {
        // W = 50 eV is in the middle branch (2 draws), W = 200 eV too; just above 200 uses none.
        let mut r = ScriptedRng::new(&[0.9, 0.5]);
        let _ = born_electron_cos_theta(ev(1000.0), ev(50.0), &mut r);
        assert_eq!(r.consumed(), 2);
        let mut r = ScriptedRng::new(&[0.9, 0.5]);
        let _ = born_electron_cos_theta(ev(1000.0), ev(200.0), &mut r);
        assert_eq!(r.consumed(), 2);
        let mut r = ScriptedRng::new(&[]);
        let _ =
            born_electron_cos_theta(ev(1000.0), ev(f64::from_bits(200f64.to_bits() + 1)), &mut r);
        assert_eq!(r.consumed(), 0);
        let mut r = ScriptedRng::new(&[0.5]);
        let _ =
            born_electron_cos_theta(ev(1000.0), ev(f64::from_bits(50f64.to_bits() - 1)), &mut r);
        assert_eq!(r.consumed(), 1);
    }

    #[test]
    fn direction_makes_the_sampled_angle_with_the_primary() {
        let axis = Vec3::new(0.2, 0.9, -0.3).normalised().unwrap();
        let mut rng = ScriptedRng::new(&[0.75, 0.3]);
        let d = sample_born_electron_direction(ev(1000.0), ev(10.0), axis, &mut rng);
        assert_relative_eq!(d.dot(axis), 0.5, epsilon = 1e-12);
        assert_relative_eq!(d.norm(), 1.0, epsilon = 1e-14);
        assert_eq!(rng.consumed(), 2);
    }
}
