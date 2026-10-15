//! One-dimensional interpolation formulas and the 2-D `QuadInterpolator` combination, written
//! to follow the Geant4-DNA formulas operation by operation.
//!
//! Sources (paths relative to `source/processes/electromagnetic/dna/models/src`):
//! * [`lin_lin`]: `G4DNAChampionElasticModel::LinLinInterpolate` (`...Champion...cc:381-390`),
//!   identical in `G4DNAIonElasticModel.cc:425`.
//! * [`log_lin_or_lin`]: `G4DNABornIonisationModel1::Interpolate` with `fasterCode == true`
//!   (`G4DNABornIonisationModel1.cc:819-875`), the branch used by option 2 (`SetDNAFast(true)`).
//! * [`log_log`]: the same function with `fasterCode == false` (slow path, kept for the Q4
//!   equivalence check in M2).
//! * [`quad`]: `QuadInterpolator` (`Champion.cc:411-445`, `Born1.cc:883-905`).
//!
//! Inputs are never validated here: callers guarantee `x1 != x2`, as Geant4's bracketing does.

/// Linear interpolation of `y` between `(x1, y1)` and `(x2, y2)` at `x`.
pub fn lin_lin(x1: f64, x2: f64, x: f64, y1: f64, y2: f64) -> f64 {
    y1 + (y2 - y1) * (x - x1) / (x2 - x1)
}

/// Born1 `Interpolate`, `fasterCode == true`: `log10(y)` linear in `x` when both `y` are
/// non-zero, plain linear interpolation when one of them is zero, `0` when `x1 == x2`.
pub fn log_lin_or_lin(x1: f64, x2: f64, x: f64, y1: f64, y2: f64) -> f64 {
    if x2 - x1 == 0.0 {
        return 0.0;
    }
    if y1 != 0.0 && y2 != 0.0 {
        let d1 = y1.log10();
        let d2 = y2.log10();
        10f64.powf(d1 + (d2 - d1) * (x - x1) / (x2 - x1))
    } else {
        y1 + (y2 - y1) * (x - x1) / (x2 - x1)
    }
}

/// Born1 `Interpolate`, `fasterCode == false`: log-log through
/// `a = (log y2 - log y1)/(log x2 - log x1)`, `b = log y2 - a log x2`; `0` if `x1` or `x2` is
/// zero or the log-spacing vanishes.
pub fn log_log(x1: f64, x2: f64, x: f64, y1: f64, y2: f64) -> f64 {
    if x1 != 0.0 && x2 != 0.0 && (x2.log10() - x1.log10()) != 0.0 {
        let a = (y2.log10() - y1.log10()) / (x2.log10() - x1.log10());
        let b = y2.log10() - a * x2.log10();
        10f64.powf(a * x.log10() + b)
    } else {
        0.0
    }
}

/// The 1-D scheme a [`quad`] interpolation applies along both axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    /// Champion elastic and ion elastic (the log variants are commented out in the C++).
    LinLin,
    /// Born ionisation, cumulated ("fast") tables.
    LogLinOrLin,
    /// Born ionisation, differential ("slow") tables.
    LogLog,
}

impl Scheme {
    fn apply(self, x1: f64, x2: f64, x: f64, y1: f64, y2: f64) -> f64 {
        match self {
            Scheme::LinLin => lin_lin(x1, x2, x, y1, y2),
            Scheme::LogLinOrLin => log_lin_or_lin(x1, x2, x, y1, y2),
            Scheme::LogLog => log_log(x1, x2, x, y1, y2),
        }
    }
}

/// Geant4 `QuadInterpolator`: interpolate along the second coordinate (`e`) inside energy
/// block `t1` between `(e11, xs11)` and `(e12, xs12)`, likewise inside block `t2`, then along
/// the first coordinate (`t`) between the two results. The two blocks have independent grids.
pub fn quad(
    scheme: Scheme,
    (e11, e12, xs11, xs12): (f64, f64, f64, f64),
    (e21, e22, xs21, xs22): (f64, f64, f64, f64),
    (t1, t2): (f64, f64),
    t: f64,
    e: f64,
) -> f64 {
    let v1 = scheme.apply(e11, e12, e, xs11, xs12);
    let v2 = scheme.apply(e21, e22, e, xs21, xs22);
    scheme.apply(t1, t2, t, v1, v2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn lin_lin_hand_values() {
        assert_eq!(lin_lin(1.0, 3.0, 2.0, 10.0, 20.0), 15.0);
        assert_eq!(lin_lin(1.0, 3.0, 1.0, 10.0, 20.0), 10.0);
        assert_eq!(lin_lin(1.0, 3.0, 3.0, 10.0, 20.0), 20.0);
    }

    #[test]
    fn log_lin_is_geometric_mean_at_midpoint() {
        // log10 y linear in x: midpoint gives sqrt(y1 y2).
        assert_relative_eq!(log_lin_or_lin(0.0, 2.0, 1.0, 4.0, 9.0), 6.0, max_relative = 1e-14);
    }

    #[test]
    fn log_lin_falls_back_to_linear_when_a_value_is_zero() {
        assert_eq!(log_lin_or_lin(0.0, 2.0, 1.0, 0.0, 8.0), 4.0);
        assert_eq!(log_lin_or_lin(0.0, 2.0, 0.5, 6.0, 0.0), 4.5);
        assert_eq!(log_lin_or_lin(1.0, 1.0, 1.0, 3.0, 4.0), 0.0);
    }

    #[test]
    fn log_log_is_geometric_mean_at_log_midpoint() {
        // x = sqrt(x1 x2) -> y = sqrt(y1 y2) for a power law.
        assert_relative_eq!(log_log(1.0, 100.0, 10.0, 2.0, 200.0), 20.0, max_relative = 1e-14);
        assert_eq!(log_log(0.0, 100.0, 10.0, 2.0, 200.0), 0.0);
    }

    #[test]
    fn quad_lin_lin_is_bilinear() {
        // value = 1 + 2 t + 3 e is reproduced exactly by bilinear interpolation.
        let f = |t: f64, e: f64| 1.0 + 2.0 * t + 3.0 * e;
        let v = quad(
            Scheme::LinLin,
            (0.0, 1.0, f(1.0, 0.0), f(1.0, 1.0)),
            (0.25, 2.0, f(2.0, 0.25), f(2.0, 2.0)),
            (1.0, 2.0),
            1.5,
            0.6,
        );
        assert_relative_eq!(v, f(1.5, 0.6), max_relative = 1e-14);
    }
}
