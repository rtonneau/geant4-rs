//! Random-variate helpers shared by the physics and physicochemistry crates.

use crate::rng::u64_to_open01;
use rand::Rng;

/// Uniform variate in the open interval (0, 1) from any `rand::Rng`, through the same
/// 52-bit mapping as `HistoryRng::uniform_open01` (neither endpoint is returned,
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

/// Isotropic unit vector from exactly two uniforms in (0, 1): `cos theta = 2 u1 - 1`,
/// `phi = 2 pi u2`. Same distribution as Geant4 `G4RandomDirection()` (v11.4.3,
/// `source/global/HEPRandom/include/G4RandomDirection.hh`), which uses Marsaglia's rejection
/// method on the unit disc (a variable number of draws, z = 2b - 1 with b uniform in (0, 1));
/// the fixed draw count keeps a history's stream independent of the outcome. Only the
/// distribution is reproduced, not the Geant4 sequence (docs/reproducibility.md).
pub fn isotropic_direction<R: Rng + ?Sized>(rng: &mut R) -> crate::vec3::Vec3 {
    let cos_theta = 2.0 * uniform(rng) - 1.0;
    let phi = 2.0 * std::f64::consts::PI * uniform(rng);
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    let (sin_phi, cos_phi) = phi.sin_cos();
    crate::vec3::Vec3::new(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::{RngStreams, Stage};
    use crate::testing::ScriptedRng;

    /// One-sample KS statistic of sorted values against the CDF `cdf`.
    fn ks(mut x: Vec<f64>, cdf: impl Fn(f64) -> f64) -> f64 {
        x.sort_by(|a, b| a.total_cmp(b));
        let n = x.len() as f64;
        x.iter()
            .enumerate()
            .map(|(i, &v)| {
                let f = cdf(v);
                (f - i as f64 / n).max((i + 1) as f64 / n - f)
            })
            .fold(0.0, f64::max)
    }

    /// Acceptance fixed before running: N = 100 000 from the `Source` stream of seed 2026,
    /// KS D < 1.95 / sqrt(N) (asymptotic alpha ~ 0.001) for cos theta against U(-1, 1) and
    /// phi against U(0, 2 pi); every vector unit to 1e-12. Fixed seed, so deterministic.
    #[test]
    fn isotropic_direction_is_uniform_on_the_sphere() {
        const N: usize = 100_000;
        let mut r = RngStreams::new(2026).stream(0, Stage::Source).unwrap();
        let v: Vec<_> = (0..N).map(|_| isotropic_direction(&mut r)).collect();
        for d in &v {
            assert!((d.norm() - 1.0).abs() < 1e-12);
        }
        let critical = 1.95 / (N as f64).sqrt();
        let two_pi = 2.0 * std::f64::consts::PI;
        let d_cos = ks(v.iter().map(|d| d.z).collect(), |c| (c + 1.0) / 2.0);
        let d_phi = ks(
            v.iter()
                .map(|d| d.y.atan2(d.x).rem_euclid(two_pi))
                .collect(),
            |p| p / two_pi,
        );
        assert!(d_cos < critical, "cos theta KS D = {d_cos} >= {critical}");
        assert!(d_phi < critical, "phi KS D = {d_phi} >= {critical}");
    }

    #[test]
    fn isotropic_direction_draws_exactly_two_uniforms() {
        let mut r = ScriptedRng::new(&[0.75, 0.25, 0.9]);
        let d = isotropic_direction(&mut r);
        assert_eq!(r.consumed(), 2);
        assert!((d.z - 0.5).abs() < 1e-12);
        assert!(d.x.abs() < 1e-12 && d.y > 0.0);
    }
}
