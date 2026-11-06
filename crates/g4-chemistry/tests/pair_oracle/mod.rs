//! Independent test-only pair dynamics. No production chemistry samplers.
pub mod experiment;
pub mod neutral;
pub mod stopped;

pub const TIMES: [f64; 5] = [0.01, 0.1, 1.0, 10.0, 100.0];
pub const STEPS: [f64; 5] = [1.0 / 16.0, 1.0 / 32.0, 1.0 / 64.0, 1.0 / 128.0, 1.0 / 256.0];
pub const SIZES: [usize; 8] = [
    10_000, 20_000, 40_000, 80_000, 160_000, 320_000, 640_000, 1_000_000,
];

/// Three-standard-error half-width must resolve the absolute effect limit.
pub fn comparison(a: usize, b: usize, n: usize) -> (f64, f64, bool) {
    let p = a as f64 / n as f64;
    let q = b as f64 / n as f64;
    // Jeffreys half-count stabilisation avoids claiming zero uncertainty for
    // an unobserved rare event. Counts themselves remain unconditional.
    let pa = (a as f64 + 0.5) / (n as f64 + 1.0);
    let pb = (b as f64 + 0.5) / (n as f64 + 1.0);
    let width = 3.0 * ((pa * (1.0 - pa) + pb * (1.0 - pb)) / n as f64).sqrt();
    let difference = (p - q).abs();
    (
        difference,
        width,
        width < 0.005 && difference < 0.005 && difference <= width,
    )
}
