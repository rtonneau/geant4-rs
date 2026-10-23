//! Free Brownian diffusion in an unbounded 3-D liquid-water volume.
//!
//! For each axis, the independent displacement is `N(0, 2 D dt)` in nm. The
//! caller owns the random stream, so chemistry history scheduling cannot change
//! stream selection. This kernel has no boundaries or reaction handling.

use crate::error::ChemistryError;
use dna_core::sampling::gaussian_pair;
use dna_core::units::{Diffusivity, Time};
use dna_core::vec3::Vec3;
use rand::Rng;

/// Diffuse a position by `dt` using four uniform draws (two Box-Muller pairs).
///
/// Inputs and the resulting position must be finite. Zero time or diffusivity
/// returns the input bits unchanged without advancing `rng`.
pub fn diffuse(
    position: Vec3,
    diffusivity: Diffusivity,
    dt: Time,
    rng: &mut impl Rng,
) -> Result<Vec3, ChemistryError> {
    for (field, value) in [
        ("position.x", position.x),
        ("position.y", position.y),
        ("position.z", position.z),
    ] {
        if !value.is_finite() {
            return Err(ChemistryError::InvalidDiffusionInput { field, value });
        }
    }
    let d = diffusivity.nm2_per_ps();
    let time = dt.ps();
    if !d.is_finite() || d < 0.0 {
        return Err(ChemistryError::InvalidDiffusionInput {
            field: "diffusivity",
            value: d,
        });
    }
    if !time.is_finite() || time < 0.0 {
        return Err(ChemistryError::InvalidDiffusionInput {
            field: "time",
            value: time,
        });
    }
    if d == 0.0 || time == 0.0 {
        return Ok(position);
    }

    // Taking separate square roots avoids overflowing 2 D dt when its square
    // root is still representable (a useful property at extreme inputs).
    let sigma = std::f64::consts::SQRT_2 * d.sqrt() * time.sqrt();
    if !sigma.is_finite() {
        return Err(ChemistryError::NonFiniteDiffusionScale { value: sigma });
    }
    let (x, y) = gaussian_pair(rng);
    let (z, _) = gaussian_pair(rng);
    let output = Vec3::new(
        position.x + sigma * x,
        position.y + sigma * y,
        position.z + sigma * z,
    );
    for (axis, value) in [("x", output.x), ("y", output.y), ("z", output.z)] {
        if !value.is_finite() {
            return Err(ChemistryError::NonFiniteDiffusionOutput { axis, value });
        }
    }
    Ok(output)
}
