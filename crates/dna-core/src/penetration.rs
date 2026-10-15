//! Penetration (thermalisation-length) law of sub-excitation electrons: the default
//! `Meesungnoen2002` of `D/models/src/G4DNAOneStepThermalizationModel.cc` (`D` =
//! `source/processes/electromagnetic/dna`), shared by the solvation model of `dna-physics`
//! and by the auto-ionisation displacer of `dna-chemistry` (single source of truth, docs/03).

use crate::sampling::gaussian_pair;
use crate::vec3::Vec3;
use rand::Rng;

/// `sqrt(pi) / 2^{3/2}` (`convertRmean3DToSigma1D`).
pub const SIGMA_1D_PER_RMEAN: f64 = 0.626_657_068_657_750_1;

/// `Meesungnoen2002::gCoeff` (`G4DNAOneStepThermalizationModel.cc:48-55`), highest power first.
pub const MEESUNGNOEN_COEFFICIENTS: [f64; 13] = [
    -4.06217193e-08,
    3.06848412e-06,
    -9.93217814e-05,
    1.80172797e-03,
    -2.01135480e-02,
    1.42939448e-01,
    -6.48348714e-01,
    1.85227848e+00,
    -3.36450378e+00,
    4.37785068e+00,
    -4.20557339e+00,
    3.81679083e+00,
    -2.34069784e-01,
];

/// `Meesungnoen2002::GetRmean` (`G4DNAOneStepThermalizationModel.cc:95-107`), in nm: the polynomial for `E > 0.1 eV`, 0 otherwise.
pub fn meesungnoen_r_mean_nm(energy_ev: f64) -> f64 {
    if energy_ev > 0.1 {
        let mut r = 0.0;
        for i in (0..=12).rev() {
            r += MEESUNGNOEN_COEFFICIENTS[12 - i] * energy_ev.powf(i as f64);
        }
        r
    } else {
        0.0
    }
}


/// Gaussian displacement of a solvated electron whose mean radius is `r_mean_nm`
/// (`GetGaussianPenetrationFromRmean3D`, `G4DNAOneStepThermalizationModel.cc:123-142`): three
/// independent Gaussians of `sigma_1D = r_mean * sqrt(pi) / 2^{3/2}`. `r_mean_nm` must be
/// positive (the `r_mean == 0` branch needs a random direction and stays in `dna-physics`).
/// Consumes four uniforms (two Box-Muller pairs, the fourth variate discarded).
pub fn gaussian_penetration<R: Rng + ?Sized>(r_mean_nm: f64, rng: &mut R) -> Vec3 {
    let sigma = r_mean_nm * SIGMA_1D_PER_RMEAN;
    let (x, y) = gaussian_pair(rng);
    let (z, _unused) = gaussian_pair(rng);
    Vec3::new(sigma * x, sigma * y, sigma * z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r_mean_is_zero_up_to_a_tenth_of_an_ev_and_positive_above() {
        assert_eq!(meesungnoen_r_mean_nm(0.1), 0.0);
        assert!(meesungnoen_r_mean_nm(1.7) > 0.0);
    }
}
