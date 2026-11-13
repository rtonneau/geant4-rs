#![allow(dead_code)]
use g4_chemistry::reaction_network::{CompiledReaction, ReactionNetwork};
use g4_chemistry::reactions::ReactionTable;
use g4_chemistry::species::SpeciesTable;
use g4_core::vec3::Vec3;

pub const BATCHES: usize = 30;
pub const DRAWS_PER_HISTORY: usize = 300;
pub const PLACEMENT_IDS: [u16; 3] = [1, 2, 11];
pub const DURATIONS: [f64; 3] = [0.0, 1.0, 1000.0];
pub const RATIOS: [f64; 2] = [1.0, 3.0];
pub const ORIENTATIONS: [&str; 3] = ["x", "z", "translated_x"];
// 4860 stochastic placement endpoints + 108 distances + 108 product counts.
pub const PLACEMENT_ENDPOINTS: usize = 5076;
pub const DIFFUSION_ENDPOINTS: usize = 1152;
pub fn network() -> ReactionNetwork {
    ReactionNetwork::compile(
        &ReactionTable::option3().unwrap(),
        &SpeciesTable::option3().unwrap(),
    )
    .unwrap()
}
pub fn positions(radius: f64, ratio: f64, orientation: usize) -> [Vec3; 2] {
    let origin = if orientation == 2 {
        Vec3::new(10.0 * radius, -7.0 * radius, 3.0 * radius)
    } else {
        Vec3::ZERO
    };
    let displacement = if orientation == 1 {
        Vec3::new(0.0, 0.0, ratio * radius)
    } else {
        Vec3::new(ratio * radius, 0.0, 0.0)
    };
    [origin, origin + displacement]
}
pub fn flat(v: Vec3) -> [f64; 3] {
    [v.x, v.y, v.z]
}

#[derive(Clone, Debug)]
pub struct Moments {
    pub n: usize,
    pub mean: Vec<f64>,
    pub cov: Vec<Vec<f64>>,
}
impl Moments {
    pub fn new(dim: usize) -> Self {
        Self {
            n: 0,
            mean: vec![0.0; dim],
            cov: vec![vec![0.0; dim]; dim],
        }
    }
    pub fn push(&mut self, values: &[f64]) {
        self.n += 1;
        let delta: Vec<_> = values.iter().zip(&self.mean).map(|(a, b)| a - b).collect();
        for (mean, d) in self.mean.iter_mut().zip(&delta) {
            *mean += d / self.n as f64;
        }
        for (i, row) in self.cov.iter_mut().enumerate() {
            for (j, value) in row.iter_mut().enumerate() {
                *value += delta[i] * (values[j] - self.mean[j]);
            }
        }
    }
    pub fn endpoints(&self) -> Vec<f64> {
        assert!(self.n > 1);
        let mut values = self.mean.clone();
        for i in 0..self.mean.len() {
            for j in i..self.mean.len() {
                values.push(self.cov[i][j] / (self.n - 1) as f64);
            }
        }
        values
    }
}

// Independent angular integration in theta. Composite Simpson integrates smooth
// sin(theta)*exp(alpha*(cos(theta)-1)), including axial sin(theta) moments.
// 4096 and 8192 panels are compared by the deterministic tests.
fn angular(alpha: f64, axial: bool, panels: usize) -> ([f64; 3], [[f64; 3]; 3]) {
    let mut integrals = [0.0; 6];
    let h = std::f64::consts::PI / panels as f64;
    let normal = alpha / -(-2.0 * alpha).exp_m1();
    for i in 0..=panels {
        let theta = i as f64 * h;
        let c = theta.cos();
        let s = theta.sin();
        let weight = if i == 0 || i == panels {
            1.0
        } else if i % 2 == 0 {
            2.0
        } else {
            4.0
        };
        let density = normal * (alpha * (c - 1.0)).exp() * s;
        for (sum, v) in integrals.iter_mut().zip([1.0, c, c * c, s, s * s, s * c]) {
            *sum += weight * density * v * h / 3.0;
        }
    }
    assert!((integrals[0] - 1.0).abs() < 1e-10);
    let mut mean = [0.0, 0.0, integrals[1]];
    let mut cov = [[0.0; 3]; 3];
    cov[2][2] = integrals[2] - mean[2] * mean[2];
    if axial {
        mean[0] = integrals[3];
        cov[0][0] = integrals[4] - mean[0] * mean[0];
        cov[0][2] = integrals[5] - mean[0] * mean[2];
        cov[2][0] = cov[0][2];
    } else {
        cov[0][0] = integrals[4] / 2.0;
        cov[1][1] = cov[0][0];
    }
    (mean, cov)
}

/// Independent source distribution moments, never sampled from the Rust kernel.
pub fn expected(
    reaction: &CompiledReaction,
    ratio: f64,
    dt: f64,
    orientation: usize,
    panels: usize,
) -> Moments {
    let p = reaction.pair().unwrap();
    let a = p.effective_radius().nm();
    let points = positions(a, ratio, orientation);
    let [d1, d2] = reaction.reactant_diffusivities().map(|d| d.nm2_per_ps());
    let has_products = !reaction.source().products.is_empty();
    let dim = if has_products { 9 } else { 6 };
    let mut output = Moments::new(dim);
    let w1 = d1 / (d1 + d2);
    let w2 = d2 / (d1 + d2);
    let q = d2.sqrt() / (d1.sqrt() + d2.sqrt());
    if dt == 0.0 {
        output.mean[..3].copy_from_slice(&flat(points[0]));
        output.mean[3..6].copy_from_slice(&flat(points[1]));
        if has_products {
            let c = (q + 0.5) / 2.0;
            output.mean[6..].copy_from_slice(&flat(points[0] * c + points[1] * (1.0 - c)));
            let delta = flat(points[0] - points[1]);
            for i in 0..3 {
                for j in 0..3 {
                    output.cov[6 + i][6 + j] = delta[i] * delta[j] / 48.0;
                }
            }
        }
        return output;
    }
    let alpha = a * (ratio * a) / (2.0 * (d1 + d2) * dt);
    let (mut mean_s1, mut cov_s1) = angular(alpha, orientation == 1, panels);
    for x in &mut mean_s1 {
        *x *= a;
    }
    for row in &mut cov_s1 {
        for x in row {
            *x *= a * a;
        }
    }
    let mean_s2 = flat(points[0] + points[1] * (d1 / d2));
    // Native-mm numerical width, converted to nm. Independent expression.
    let sigma_mm = 2.0 * (d1 * 1e-9) * (dt * 1e-3) * (1.0 + d1 / d2);
    let sigma_nm = sigma_mm * 1e6;
    let coefficients = [w1, -w2, (q + 0.5) / 2.0 - w2];
    for group in 0..dim / 3 {
        for i in 0..3 {
            output.mean[3 * group + i] = coefficients[group] * mean_s1[i] + w2 * mean_s2[i];
            for other in 0..dim / 3 {
                for j in 0..3 {
                    let mut cov = coefficients[group] * coefficients[other] * cov_s1[i][j];
                    if i == j {
                        cov += w2 * w2 * sigma_nm * sigma_nm;
                    }
                    if group == 2 && other == 2 {
                        cov += (cov_s1[i][j] + mean_s1[i] * mean_s1[j]) / 48.0;
                    }
                    output.cov[3 * group + i][3 * other + j] = cov;
                }
            }
        }
    }
    output
}
pub fn oracle_endpoints(m: &Moments) -> (Vec<f64>, Vec<f64>) {
    let dim = m.mean.len();
    let mut values = m.mean.clone();
    let mut scales = vec![];
    for i in 0..dim {
        scales.push(m.cov[i][i].max(0.0).sqrt());
    }
    let std = scales.clone();
    for i in 0..dim {
        for j in i..dim {
            values.push(m.cov[i][j]);
            scales.push(std[i] * std[j]);
        }
    }
    (values, scales)
}

fn ln_gamma(z: f64) -> f64 {
    let c = [
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    let z = z - 1.0;
    let mut x = 0.999_999_999_999_809_9;
    for (i, c) in c.iter().enumerate() {
        x += c / (z + i as f64 + 1.0);
    }
    let t = z + 7.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + x.ln()
}
fn beta_fraction(a: f64, b: f64, x: f64) -> f64 {
    let tiny = 1e-300;
    let mut c = 1.0;
    let mut d = 1.0 - (a + b) * x / (a + 1.0);
    if d.abs() < tiny {
        d = tiny;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=1000 {
        let m = m as f64;
        for aa in [
            m * (b - m) * x / ((a + 2.0 * m - 1.0) * (a + 2.0 * m)),
            -(a + m) * (a + b + m) * x / ((a + 2.0 * m) * (a + 2.0 * m + 1.0)),
        ] {
            d = 1.0 + aa * d;
            if d.abs() < tiny {
                d = tiny;
            }
            c = 1.0 + aa / c;
            if c.abs() < tiny {
                c = tiny;
            }
            d = 1.0 / d;
            let delta = d * c;
            h *= delta;
            if aa < 0.0 && (delta - 1.0).abs() < 2e-14 {
                return h;
            }
        }
    }
    panic!("incomplete beta failed to converge")
}
fn beta_regularized(x: f64, a: f64, b: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    if x == 1.0 {
        return 1.0;
    }
    let factor =
        (ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (-x).ln_1p()).exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        factor * beta_fraction(a, b, x) / a
    } else {
        1.0 - factor * beta_fraction(b, a, 1.0 - x) / b
    }
}
pub fn t_critical(df: f64, alpha: f64) -> f64 {
    let (mut lo, mut hi) = (0.0, 1000.0);
    for _ in 0..65 {
        let t = (lo + hi) / 2.0;
        let tail = 0.5 * beta_regularized(df / (df + t * t), df / 2.0, 0.5);
        if tail > alpha / 2.0 {
            lo = t;
        } else {
            hi = t;
        }
    }
    (lo + hi) / 2.0
}
fn mean_variance(values: &[f64]) -> (f64, f64) {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance =
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    (mean, variance)
}
pub fn interval(
    first: &[f64],
    second: Option<&[f64]>,
    expected: f64,
    alpha: f64,
) -> (f64, f64, f64, f64) {
    let (mean, var) = mean_variance(first);
    let v = var / first.len() as f64;
    let (other, w, other_n) = second.map_or((expected, 0.0, first.len()), |b| {
        let (m, s) = mean_variance(b);
        (m, s / b.len() as f64, b.len())
    });
    let difference = mean - other;
    if v + w == 0.0 {
        return (difference, difference, mean, var);
    }
    let df = (v + w).powi(2) / (v * v / (first.len() - 1) as f64 + w * w / (other_n - 1) as f64);
    let margin = t_critical(df, alpha) * (v + w).sqrt();
    (difference - margin, difference + margin, mean, var)
}
pub fn verdict(lo: f64, hi: f64, band: f64) -> &'static str {
    if lo > -band && hi < band {
        "PASS"
    } else if lo > band || hi < -band {
        "FAIL"
    } else {
        "INCONCLUSIVE"
    }
}
