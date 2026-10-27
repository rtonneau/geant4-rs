//! Radial Euler/Skorokhod dynamics in units a=D=1.
//! dr = 2/r dt + sqrt(2) dW + dL; radiation survival exp(-kappa L).
//! The minimum of each frozen-drift Brownian bridge detects hidden contacts.
use super::TIMES;
use g4_core::rng::{RngStreams, Stage, u64_to_open01};
use rand::Rng;
use rayon::prelude::*;

/// None is an absorbing boundary; Some(kappa) is a radiation boundary.
pub fn counts(
    case: usize,
    distance: usize,
    ratio: f64,
    kappa: Option<f64>,
    level: usize,
    h: f64,
    n: usize,
) -> [usize; 5] {
    let streams = RngStreams::new(20_261_027_402);
    (0..n)
        .into_par_iter()
        .map(|pair| {
            // Fixed strides preserve histories when the precision budget grows.
            let history = (((case * 3 + distance) * 5 + level) * 1_000_000 + pair) as u64;
            let mut rng = streams.stream(history, Stage::Chemistry).unwrap();
            let mut uniform = || u64_to_open01(rng.next_u64());
            let threshold = kappa.map(|k| -uniform().ln() / k);
            let mut local_time = 0.0;
            let mut r = ratio;
            let mut t = 0.0;
            let mut reacted = kappa.is_none() && ratio == 1.0;
            let mut result = [0; 5];
            let mut spare = None;
            for (endpoint, &time) in TIMES.iter().enumerate() {
                while !reacted && t < time {
                    let dt = h.min(time - t);
                    let normal = spare.take().unwrap_or_else(|| {
                        let radius = (-2.0 * uniform().ln()).sqrt();
                        let angle = std::f64::consts::TAU * uniform();
                        let (s, c) = angle.sin_cos();
                        spare = Some(radius * s);
                        radius * c
                    });
                    let x = r - 1.0;
                    let y = x + 2.0 / r * dt + (2.0 * dt).sqrt() * normal;
                    // Conditional bridge minimum: P(min < m)=exp(-(x-m)(y-m)/dt).
                    let minimum =
                        0.5 * (x + y - ((x - y).powi(2) - 4.0 * dt * uniform().ln()).sqrt());
                    let push = (-minimum).max(0.0);
                    local_time += push;
                    r = 1.0 + y + push;
                    reacted = threshold.map_or(push > 0.0, |limit| local_time >= limit);
                    t += dt;
                }
                result[endpoint] = usize::from(reacted);
            }
            result
        })
        .reduce(|| [0; 5], |a, b| std::array::from_fn(|i| a[i] + b[i]))
}
