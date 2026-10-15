//! Random-variate helpers shared by the models.

use dna_core::rng::u64_to_open01;
use rand::Rng;

/// Uniform variate in the open interval (0, 1) from any `rand::Rng`, through the same
/// 52-bit mapping as `dna_core::rng::HistoryRng::uniform_open01` (neither endpoint is returned,
/// so `ln(u)` is finite) so that results never depend on the `rand` distribution API.
pub fn uniform<R: Rng + ?Sized>(rng: &mut R) -> f64 {
    u64_to_open01(rng.next_u64())
}

/// Two independent standard-normal variates by the Box-Muller transform from two uniforms in
/// (0, 1): `sqrt(-2 ln u1) (cos 2 pi u2, sin 2 pi u2)`. Stateless (no cached second value), so
/// the number of draws per call is fixed at two and a history's stream never depends on call
/// history. Geant4's `G4RandGauss::shoot` uses a different algorithm; only the distribution
/// is reproduced (docs/reproducibility.md).
pub fn gaussian_pair<R: Rng + ?Sized>(rng: &mut R) -> (f64, f64) {
    let u1 = uniform(rng);
    let u2 = uniform(rng);
    let radius = (-2.0 * u1.ln()).sqrt();
    let (sin, cos) = (2.0 * std::f64::consts::PI * u2).sin_cos();
    (radius * cos, radius * sin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::ScriptedRng;

    #[test]
    fn gaussian_pair_consumes_two_draws_and_maps_them_as_documented() {
        let mut rng = ScriptedRng::new(&[0.25, 0.125]);
        let (a, b) = gaussian_pair(&mut rng);
        assert_eq!(rng.consumed(), 2);
        let r = (-2.0f64 * 0.25f64.ln()).sqrt();
        let phi = 2.0 * std::f64::consts::PI * 0.125;
        assert!((a - r * phi.cos()).abs() < 1e-12);
        assert!((b - r * phi.sin()).abs() < 1e-12);
    }
}
