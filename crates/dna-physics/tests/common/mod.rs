//! Shared helpers for the integration tests: data access, an independent raw-file reader (used as
//! oracle, deliberately not `dna-data`) and the statistical tests with thresholds fixed in the
//! test files before any run.
#![allow(dead_code)]

use dna_core::units::Energy;
use dna_data::datadir::{DataDir, repo_data_dir};
use std::path::PathBuf;

pub fn ev(x: f64) -> Energy {
    Energy::from_ev(x)
}

/// The committed data subset, or `None` (with a visible message) when it is absent, so that a
/// source-only checkout skips real-table tests instead of failing. Loads after this still unwrap,
/// so a present but corrupt data set fails loudly.
pub fn data() -> Option<DataDir> {
    let root = repo_data_dir();
    if !root.join("dna").is_dir() || !root.join("MANIFEST.toml").is_file() {
        eprintln!(
            "SKIPPED: Geant4 data not found at {} (expected dna/ and MANIFEST.toml)",
            root.display()
        );
        return None;
    }
    Some(DataDir::new(root).unwrap())
}

pub fn raw_path(name: &str) -> PathBuf {
    repo_data_dir().join("dna").join(name)
}

/// Rows of a numeric file: whitespace separated columns, `#` comments, no other processing.
pub fn raw_rows(name: &str) -> Vec<Vec<f64>> {
    std::fs::read_to_string(raw_path(name))
        .unwrap()
        .lines()
        .filter_map(|l| {
            let t: Vec<f64> = l
                .split('#')
                .next()
                .unwrap()
                .split_whitespace()
                .map(|x| x.parse().unwrap())
                .collect();
            (!t.is_empty()).then_some(t)
        })
        .collect()
}

/// Rows grouped by equal first column: `(E, rows_without_the_E_column)`.
pub fn raw_blocks(name: &str) -> Vec<(f64, Vec<Vec<f64>>)> {
    let mut out: Vec<(f64, Vec<Vec<f64>>)> = Vec::new();
    for r in raw_rows(name) {
        match out.last_mut() {
            Some((e, rows)) if *e == r[0] => rows.push(r[1..].to_vec()),
            _ => out.push((r[0], vec![r[1..].to_vec()])),
        }
    }
    out
}

/// Kolmogorov-Smirnov statistic of `sorted` samples against a *continuous* CDF.
pub fn ks_statistic(sorted: &[f64], cdf: impl Fn(f64) -> f64) -> f64 {
    let n = sorted.len() as f64;
    sorted
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let f = cdf(x);
            (f - i as f64 / n).abs().max(((i + 1) as f64 / n - f).abs())
        })
        .fold(0.0, f64::max)
}

/// Asymptotic p-value of the KS statistic (Stephens' small-sample correction).
pub fn ks_pvalue(d: f64, n: usize) -> f64 {
    let sn = (n as f64).sqrt();
    let lambda = (sn + 0.12 + 0.11 / sn) * d;
    let mut sum = 0.0;
    for j in 1..=100 {
        let j = j as f64;
        let term = (-2.0 * j * j * lambda * lambda).exp();
        sum += if (j as i64) % 2 == 1 { term } else { -term };
        if term < 1e-18 {
            break;
        }
    }
    (2.0 * sum).clamp(0.0, 1.0)
}

/// Regularised upper incomplete gamma `Q(a, x)` (Numerical Recipes `gammq`).
fn gamma_q(a: f64, x: f64) -> f64 {
    fn ln_gamma(x: f64) -> f64 {
        const C: [f64; 6] = [
            76.18009172947146,
            -86.50532032941677,
            24.01409824083091,
            -1.231739572450155,
            0.1208650973866179e-2,
            -0.5395239384953e-5,
        ];
        let mut y = x;
        let tmp = x + 5.5 - (x + 0.5) * (x + 5.5).ln();
        let mut ser = 1.000000000190015;
        for c in C {
            y += 1.0;
            ser += c / y;
        }
        -tmp + (2.5066282746310005 * ser / x).ln()
    }
    if x <= 0.0 {
        return 1.0;
    }
    if x < a + 1.0 {
        let (mut ap, mut sum, mut del) = (a, 1.0 / a, 1.0 / a);
        for _ in 0..1000 {
            ap += 1.0;
            del *= x / ap;
            sum += del;
            if del.abs() < sum.abs() * 1e-16 {
                break;
            }
        }
        1.0 - sum * (-x + a * x.ln() - ln_gamma(a)).exp()
    } else {
        let tiny = 1e-300;
        let mut b = x + 1.0 - a;
        let mut c = 1.0 / tiny;
        let mut d = 1.0 / b;
        let mut h = d;
        for i in 1..1000 {
            let an = -(i as f64) * (i as f64 - a);
            b += 2.0;
            d = an * d + b;
            if d.abs() < tiny {
                d = tiny;
            }
            c = b + an / c;
            if c.abs() < tiny {
                c = tiny;
            }
            d = 1.0 / d;
            let del = d * c;
            h *= del;
            if (del - 1.0).abs() < 1e-16 {
                break;
            }
        }
        (-x + a * x.ln() - ln_gamma(a)).exp() * h
    }
}

/// p-value of a chi-square statistic with `dof` degrees of freedom.
pub fn chi2_pvalue(chi2: f64, dof: usize) -> f64 {
    gamma_q(dof as f64 / 2.0, chi2 / 2.0)
}

/// Pearson chi-square of `observed` counts against `expected` counts; bins with an expected count
/// below 5 are merged into their neighbour, returns `(chi2, dof)`.
pub fn chi2_merged(observed: &[f64], expected: &[f64]) -> (f64, usize) {
    let (mut o_acc, mut e_acc) = (0.0, 0.0);
    let mut bins: Vec<(f64, f64)> = Vec::new();
    for (&o, &e) in observed.iter().zip(expected) {
        o_acc += o;
        e_acc += e;
        if e_acc >= 5.0 {
            bins.push((o_acc, e_acc));
            o_acc = 0.0;
            e_acc = 0.0;
        }
    }
    if e_acc > 0.0 {
        match bins.last_mut() {
            Some(last) => {
                last.0 += o_acc;
                last.1 += e_acc;
            }
            None => bins.push((o_acc, e_acc)),
        }
    }
    let chi2 = bins.iter().map(|(o, e)| (o - e) * (o - e) / e).sum();
    (chi2, bins.len().saturating_sub(1))
}

#[cfg(test)]
mod self_tests {
    use super::*;

    #[test]
    fn chi2_pvalue_matches_known_values() {
        // chi2 = 3.841 with 1 dof -> p = 0.05; chi2 = 18.307 with 10 dof -> p = 0.05
        assert!((chi2_pvalue(3.841, 1) - 0.05).abs() < 1e-3);
        assert!((chi2_pvalue(18.307, 10) - 0.05).abs() < 1e-3);
        assert!((chi2_pvalue(0.0, 5) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn ks_pvalue_matches_known_values() {
        // D sqrt(n) = 1.358 -> p ~ 0.05 asymptotically
        let n = 10_000;
        let d = 1.358 / (n as f64).sqrt();
        assert!((ks_pvalue(d, n) - 0.05).abs() < 0.01);
    }
}
