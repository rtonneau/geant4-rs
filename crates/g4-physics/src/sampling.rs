//! Random-variate helpers shared by the models.

pub use g4_core::sampling::{gaussian_pair, uniform};

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
