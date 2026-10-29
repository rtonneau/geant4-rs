//! Fixed-step trajectories with an interval-censored eventual endpoint.
//!
//! Escape means the first sampled surviving radius at or beyond the outer sphere.
//! Finite observations continue unchanged after escape. Continuum return bounds
//! and Monte Carlo bands do not bound the numerical trajectory's discretization.
use super::TIMES;
use g4_core::rng::{RngStreams, Stage, u64_to_open01};
use rand::Rng;
use rayon::prelude::*;
use std::ops::Range;

/// Dimensionless parameters; the caller supplies the registered refinement step.
pub struct Params {
    pub case: usize,
    pub distance: usize,
    pub ratio: f64,
    pub charge: f64,
    pub kappa: Option<f64>,
    pub level: usize,
    pub h: f64,
    pub outer_radius: f64,
    pub horizon: f64,
}

/// Disjoint eventual categories, plus the unchanged finite reaction counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub finite: [usize; 5],
    pub reacted: usize,
    pub escaped: usize,
    pub unresolved: usize,
}

impl Counts {
    fn checked_n(&self) -> Option<usize> {
        self.reacted
            .checked_add(self.escaped)?
            .checked_add(self.unresolved)
    }

    /// Number of histories, checked against integer overflow.
    pub fn n(&self) -> usize {
        self.checked_n().expect("trajectory count overflow")
    }

    /// Combine disjoint history batches; reject overflow or invalid finite counts.
    pub fn checked_add(self, other: Self) -> Option<Self> {
        for item in [&self, &other] {
            let n = item.checked_n()?;
            if item.finite.iter().any(|&count| count > n)
                || item.finite.windows(2).any(|pair| pair[0] > pair[1])
            {
                return None;
            }
        }
        let mut finite = [0; 5];
        for (index, count) in finite.iter_mut().enumerate() {
            *count = self.finite[index].checked_add(other.finite[index])?;
        }
        let result = Self {
            finite,
            reacted: self.reacted.checked_add(other.reacted)?,
            escaped: self.escaped.checked_add(other.escaped)?,
            unresolved: self.unresolved.checked_add(other.unresolved)?,
        };
        result.checked_n()?;
        Some(result)
    }

    /// Empirical identification interval, excluding Monte Carlo uncertainty.
    /// Escaped histories remain censored; this is not a pathwise outcome interval.
    pub fn nonreaction_interval(&self, return_bound: f64) -> (f64, f64) {
        assert!((0.0..=1.0).contains(&return_bound));
        let n = self.n();
        if n == 0 {
            return (0.0, 1.0);
        }
        (
            (1.0 - return_bound) * self.escaped as f64 / n as f64,
            1.0 - self.reacted as f64 / n as f64,
        )
    }

    /// One-sided Hoeffding radius, simultaneous over 216 endpoints and 40 looks.
    /// The two category bounds share each look's alpha by a union bound.
    pub fn hoeffding_mc_half_width(&self) -> f64 {
        let n = self.n();
        if n == 0 {
            return f64::INFINITY;
        }
        let alpha_per_look = 0.001 / 216.0 / 40.0;
        ((2.0_f64 / alpha_per_look).ln() / (2.0 * n as f64)).sqrt()
    }

    /// Simultaneous population band, including censoring and sampling uncertainty.
    /// This deliberately leaves discretization error outside the statistical band.
    pub fn nonreaction_band(&self, return_bound: f64) -> (f64, f64) {
        assert!((0.0..=1.0).contains(&return_bound));
        let n = self.n();
        if n == 0 {
            return (0.0, 1.0);
        }
        let width = self.hoeffding_mc_half_width();
        let escaped_lower = (self.escaped as f64 / n as f64 - width).max(0.0);
        let reacted_lower = (self.reacted as f64 / n as f64 - width).max(0.0);
        ((1.0 - return_bound) * escaped_lower, 1.0 - reacted_lower)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Endpoint {
    Reacted,
    Escaped,
    Unresolved,
}

struct Trajectory {
    r: f64,
    t: f64,
    local_time: f64,
    threshold: Option<f64>,
    reacted: bool,
    spare: Option<f64>,
}

impl Trajectory {
    fn step(&mut self, dt: f64, charge: f64, uniform: &mut impl FnMut() -> f64) {
        let normal = self.spare.take().unwrap_or_else(|| {
            let radius = (-2.0 * uniform().ln()).sqrt();
            let angle = std::f64::consts::TAU * uniform();
            let (s, c) = angle.sin_cos();
            self.spare = Some(radius * s);
            radius * c
        });
        let x = self.r - 1.0;
        let drift = 2.0 / self.r + charge / (self.r * self.r);
        let y = x + drift * dt + (2.0 * dt).sqrt() * normal;
        let minimum = 0.5 * (x + y - ((x - y).powi(2) - 4.0 * dt * uniform().ln()).sqrt());
        let push = (-minimum).max(0.0);
        self.local_time += push;
        self.r = 1.0 + y + push;
        self.reacted = self
            .threshold
            .map_or(push > 0.0, |limit| self.local_time >= limit);
        let next_time = self.t + dt;
        assert!(
            next_time > self.t,
            "step cannot advance floating-point time"
        );
        self.t = next_time;
    }

    fn classify(&self, outer_radius: f64) -> Endpoint {
        if self.reacted {
            Endpoint::Reacted
        } else if self.r >= outer_radius {
            Endpoint::Escaped
        } else {
            Endpoint::Unresolved
        }
    }
}

/// Evaluate a disjoint range of registered pair ordinals without changing streams.
pub fn counts(params: &Params, range: Range<usize>) -> Counts {
    assert!(range.start <= range.end && range.end <= 1_000_000);
    assert!(params.distance < 3 && params.level < 5);
    assert!(params.ratio.is_finite() && params.ratio >= 1.0);
    assert!(params.charge.is_finite());
    assert!(params.kappa.is_none_or(|k| k.is_finite() && k > 0.0));
    assert!(params.h.is_finite() && params.h > 0.0);
    assert!(params.outer_radius.is_finite() && params.outer_radius > 1.0);
    assert!(params.horizon.is_finite() && params.horizon >= TIMES[4]);
    let history_base = params
        .case
        .checked_mul(3)
        .and_then(|value| value.checked_add(params.distance))
        .and_then(|value| value.checked_mul(5))
        .and_then(|value| value.checked_add(params.level))
        .and_then(|value| value.checked_mul(1_000_000))
        .expect("history ordinal overflow");
    let streams = RngStreams::new(20_261_027_402);
    range
        .into_par_iter()
        .map(|pair| {
            let history = u64::try_from(history_base.checked_add(pair).unwrap()).unwrap();
            let mut rng = streams.stream(history, Stage::Chemistry).unwrap();
            let mut uniform = || u64_to_open01(rng.next_u64());
            let mut trajectory = Trajectory {
                r: params.ratio,
                t: 0.0,
                local_time: 0.0,
                threshold: params.kappa.map(|k| -uniform().ln() / k),
                reacted: params.kappa.is_none() && params.ratio == 1.0,
                spare: None,
            };
            let mut endpoint = trajectory.classify(params.outer_radius);
            let mut finite = [0; 5];
            for (index, &time) in TIMES.iter().enumerate() {
                while !trajectory.reacted && trajectory.t < time {
                    let dt = params.h.min(time - trajectory.t);
                    trajectory.step(dt, params.charge, &mut uniform);
                    if endpoint == Endpoint::Unresolved {
                        endpoint = trajectory.classify(params.outer_radius);
                    }
                }
                finite[index] = usize::from(trajectory.reacted);
            }
            while endpoint == Endpoint::Unresolved && trajectory.t < params.horizon {
                let dt = params.h.min(params.horizon - trajectory.t);
                trajectory.step(dt, params.charge, &mut uniform);
                endpoint = trajectory.classify(params.outer_radius);
            }
            Counts {
                finite,
                reacted: usize::from(endpoint == Endpoint::Reacted),
                escaped: usize::from(endpoint == Endpoint::Escaped),
                unresolved: usize::from(endpoint == Endpoint::Unresolved),
            }
        })
        .reduce(Counts::default, |a, b| {
            a.checked_add(b).expect("invalid trajectory counts")
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pair_oracle::{STEPS, neutral};

    fn params() -> Params {
        Params {
            case: 2,
            distance: 1,
            ratio: 1.5,
            charge: 0.0,
            kappa: Some(1.0),
            level: 0,
            h: STEPS[0],
            outer_radius: 16.0,
            horizon: 100.0,
        }
    }

    #[test]
    fn batches_conserve_mass_and_preserve_finite_streams() {
        let mut p = params();
        for charge in [0.0, -1.0, 1.0] {
            p.charge = charge;
            let whole = counts(&p, 0..12);
            let batched = counts(&p, 0..5).checked_add(counts(&p, 5..12)).unwrap();
            assert_eq!(whole, batched);
            assert_eq!(whole.n(), 12);
            assert_eq!(
                whole.finite,
                crate::charged::counts(
                    p.case, p.distance, p.ratio, charge, p.kappa, p.level, p.h, 12
                )
            );
            if charge == 0.0 {
                assert_eq!(
                    whole.finite,
                    neutral::counts(p.case, p.distance, p.ratio, p.kappa, p.level, p.h, 12)
                );
            }
        }
    }

    #[test]
    fn contact_and_empty_ranges_are_exact() {
        let mut p = params();
        p.ratio = 1.0;
        p.kappa = None;
        assert_eq!(counts(&p, 4..4), Counts::default());
        assert_eq!(
            counts(&p, 4..11),
            Counts {
                finite: [7; 5],
                reacted: 7,
                escaped: 0,
                unresolved: 0
            }
        );
        assert_eq!(Counts::default().nonreaction_band(0.1), (0.0, 1.0));
    }

    #[test]
    fn escape_remains_censored_while_finite_observations_continue() {
        let mut p = params();
        p.outer_radius = 1.25;
        let result = counts(&p, 0..12);
        assert_eq!(result.escaped, 12);
        assert_eq!(result.reacted, 0);
        assert_eq!(result.unresolved, 0);
        assert!(result.finite[4] > 0);
        assert_eq!(
            result.finite,
            neutral::counts(p.case, p.distance, p.ratio, p.kappa, p.level, p.h, 12)
        );
    }

    #[test]
    fn longer_tail_preserves_finite_observations_and_resolves_only_survivors() {
        let mut p = params();
        p.outer_radius = 1_000_000.0;
        let initial = counts(&p, 0..12);
        p.horizon = 101.0;
        let continued = counts(&p, 0..12);
        assert_eq!(initial.finite, continued.finite);
        assert_eq!(continued.n(), initial.n());
        assert!(continued.reacted >= initial.reacted);
        assert!(continued.escaped >= initial.escaped);
        assert!(continued.unresolved <= initial.unresolved);
    }

    #[test]
    fn interval_contains_censoring_and_band_adds_sampling_error() {
        let result = Counts {
            finite: [300; 5],
            reacted: 300,
            escaped: 500,
            unresolved: 200,
        };
        let q = crate::escape::return_probability(0.0, 100.0);
        let interval = result.nonreaction_interval(q);
        assert!((interval.0 - 0.495).abs() < 1e-15);
        assert!((interval.1 - 0.7).abs() < 1e-15);
        let band = result.nonreaction_band(q);
        assert!(band.0 < interval.0 && band.1 > interval.1);
        let alpha = 0.001 / 216.0 / 40.0;
        let width = result.hoeffding_mc_half_width();
        assert!((2.0 * (-2.0 * result.n() as f64 * width * width).exp() - alpha).abs() < 1e-15);
    }

    #[test]
    fn combine_rejects_overflow_and_invalid_counts() {
        let huge = Counts {
            finite: [0; 5],
            reacted: usize::MAX,
            escaped: 0,
            unresolved: 0,
        };
        let one = Counts {
            finite: [0; 5],
            reacted: 0,
            escaped: 1,
            unresolved: 0,
        };
        assert!(huge.checked_add(one).is_none());
        let invalid = Counts {
            finite: [2; 5],
            ..one
        };
        assert!(invalid.checked_add(Counts::default()).is_none());
    }
}
