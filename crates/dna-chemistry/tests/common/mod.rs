//! Shared helpers of the integration tests: fixtures and the chi-square p-value.
#![allow(dead_code)]

use serde_json::Value;
use std::path::PathBuf;

/// Geant4 11.4.1 `G4MoleculeTable` keys represented by `Species`, in C++ creation order.
pub const CPP_SPECIES_KEYS: [&str; 15] = [
    "H3Op", "°OH", "OHm", "e_aq", "H", "H2", "H2O2", "HO2°", "HO2m", "Oxy", "Om", "O2", "O2m",
    "O3", "O3m",
];

/// These C++ configurations serve chemistry bookkeeping and are outside `Species`.
pub const CPP_BOOKKEEPING_KEYS: [&str; 4] = ["H2O(B)", "H3Op(B)", "OHm(B)", "NoneM"];

pub fn fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
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

#[cfg(test)]
mod self_tests {
    use super::*;

    #[test]
    fn chi2_pvalue_matches_known_values() {
        assert!((chi2_pvalue(3.841, 1) - 0.05).abs() < 1e-3);
        assert!((chi2_pvalue(18.307, 10) - 0.05).abs() < 1e-3);
    }
}
