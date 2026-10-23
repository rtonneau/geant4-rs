//! Complementary error functions for the IRT kernels; no new numerical dependency.
//!
//! Forward evaluation uses W. J. Cody, "Rational Chebyshev approximations for
//! the error function", Math. Comp. 23 (1969), 631-638, DOI: 10.1090/S0025-5718-1969-0247736-4.
//! Coefficients and interval boundaries are from Cody's CALERF (19 March 1990):
//! <https://netlib.org/specfun/erf>. Unlike CALERF's historical machine cutoff,
//! this implementation retains binary64 subnormal results.
//!
//! Geant4 provenance: 11.4.1, `source/processes/electromagnetic/dna/utils/`
//! `src/G4ErrorFunction.cc` and `include/G4ErrorFunction.hh`. Its `erfc` calls
//! `std::erfc`; `erfcInv` uses `-NormQuantile(p / 2) / sqrt(2)`. The Rust kernel
//! deliberately uses independently validated algorithms and checked domains.
//!
//! Inversion uses bounded bisection of `ln(erfc(x)) = ln(p)` on the positive
//! half-line, followed by symmetry. The bound `erfc(x) <= exp(-x*x)` brackets
//! the root between zero and `sqrt(-ln(p)) + 1`. Logarithms of the scaled Cody
//! approximation retain information even for the smallest positive subnormal p;
//! evaluating `erfc(x)` directly would quantize such roots into wide plateaus.

use crate::error::ChemistryError;

const INV_SQRT_PI: f64 = 0.564_189_583_547_756_3;

/// Complementary error function on all finite real inputs.
///
/// Returns a finite value in `[0, 2]`, with `erfc(0) = 1`. Positive tails
/// round to zero when unrepresentable; negative tails round to two. NaN and
/// infinities are rejected rather than accepted as extended-real arguments.
pub(crate) fn erfc(x: f64) -> Result<f64, ChemistryError> {
    if !x.is_finite() {
        return Err(ChemistryError::SpecialFunctionDomain {
            function: "erfc",
            value: x,
            domain: "a finite real argument",
        });
    }
    let y = x.abs();
    if y <= 0.46875 {
        return Ok(1.0 - erf_small(x));
    }
    // erfc(28) < half the smallest subnormal. Check before squaring huge x.
    let tail = if y >= 28.0 {
        0.0
    } else {
        // The rounded square alone loses relative accuracy in exp(-y*y).
        // Cody's split makes the first square exact and keeps its correction small.
        let high = (y * 16.0).trunc() / 16.0;
        let correction = (y - high) * (y + high);
        (-high * high).exp() * ((-correction).exp() * erfc_scaled_positive(y))
    };
    Ok(if x < 0.0 { 2.0 - tail } else { tail })
}

/// Inverse complementary error function for finite `0 < p < 2`.
///
/// All accepted inputs have finite outputs; `erfc_inv(1) = 0`. The exact
/// boundaries zero and two have infinite mathematical limits, so both return
/// `ChemistryError::SpecialFunctionDomain`, as do out-of-range and non-finite p.
/// Callers that interpret a zero probability as an infinite time must handle
/// that physical outcome before invoking this finite numerical kernel.
pub(crate) fn erfc_inv(p: f64) -> Result<f64, ChemistryError> {
    if !p.is_finite() || p <= 0.0 || p >= 2.0 {
        return Err(ChemistryError::SpecialFunctionDomain {
            function: "erfc_inv",
            value: p,
            domain: "a finite probability strictly between zero and two",
        });
    }
    if p == 1.0 {
        return Ok(0.0);
    }
    // For p > 1 the subtraction is exact (Sterbenz's lemma). Never form 1-p
    // or p/2 in the positive tail: either would destroy tiny probabilities.
    let tail = if p > 1.0 { 2.0 - p } else { p };
    let log_tail = tail.ln();
    let mut lower = 0.0;
    let mut upper = (-log_tail).sqrt() + 1.0;
    // The initial interval is shorter than 29. The extra iterations resolve
    // even the smallest nonzero root (p immediately adjacent to one) down to
    // adjacent f64 endpoints; ordinary roots finish in about 55 iterations.
    for _ in 0..128 {
        let midpoint = lower + (upper - lower) * 0.5;
        if midpoint == lower || midpoint == upper {
            break;
        }
        let residual = if midpoint <= 0.46875 {
            // log1p preserves the information in 1-erf(x) when x is tiny.
            (-erf_small(midpoint)).ln_1p() - log_tail
        } else {
            // FMA avoids rounding x*x before subtracting the nearby -ln(p).
            erfc_scaled_positive(midpoint).ln() - midpoint.mul_add(midpoint, log_tail)
        };
        if residual > 0.0 {
            lower = midpoint;
        } else {
            upper = midpoint;
        }
    }
    let root = lower + (upper - lower) * 0.5;
    Ok(if p > 1.0 { -root } else { root })
}

/// Cody's central rational approximation to erf, used only for |x| <= 0.46875.
fn erf_small(x: f64) -> f64 {
    let a = [
        3.161_123_743_870_565_5,
        113.864_154_151_050_16,
        377.485_237_685_302,
        3_209.377_589_138_469_4,
        0.185_777_706_184_603_15,
    ];
    let b = [
        23.601_290_952_344_12,
        244.024_637_934_444_17,
        1_282.616_526_077_372_3,
        2_844.236_833_439_171,
    ];
    let square = x * x;
    let mut numerator = a[4] * square;
    let mut denominator = square;
    for i in 0..3 {
        numerator = (numerator + a[i]) * square;
        denominator = (denominator + b[i]) * square;
    }
    x * (numerator + a[3]) / (denominator + b[3])
}

/// exp(x*x) * erfc(x), evaluated without exponentials for 0.46875 < x < 28.
fn erfc_scaled_positive(x: f64) -> f64 {
    if x <= 4.0 {
        let c = [
            0.564_188_496_988_670_1,
            8.883_149_794_388_376,
            66.119_190_637_141_63,
            298.635_138_197_400_1,
            881.952_221_241_769,
            1_712.047_612_634_070_6,
            2_051.078_377_826_071_5,
            1_230.339_354_797_997_2,
            2.153_115_354_744_038_5e-8,
        ];
        let d = [
            15.744_926_110_709_835,
            117.693_950_891_312_5,
            537.181_101_862_009_9,
            1_621.389_574_566_690_2,
            3_290.799_235_733_459_7,
            4_362.619_090_143_247,
            3_439.367_674_143_721_6,
            1_230.339_354_803_749_4,
        ];
        let mut numerator = c[8] * x;
        let mut denominator = x;
        for i in 0..7 {
            numerator = (numerator + c[i]) * x;
            denominator = (denominator + d[i]) * x;
        }
        (numerator + c[7]) / (denominator + d[7])
    } else {
        let p = [
            0.305_326_634_961_232_34,
            0.360_344_899_949_804_44,
            0.125_781_726_111_229_25,
            0.016_083_785_148_742_28,
            0.000_658_749_161_529_837_8,
            0.016_315_387_137_302_098,
        ];
        let q = [
            2.568_520_192_289_822,
            1.872_952_849_923_460_4,
            0.527_905_102_951_428_4,
            0.060_518_341_312_441_32,
            0.002_335_204_976_268_692,
        ];
        let reciprocal_square = 1.0 / (x * x);
        let mut numerator = p[5] * reciprocal_square;
        let mut denominator = reciprocal_square;
        for i in 0..4 {
            numerator = (numerator + p[i]) * reciprocal_square;
            denominator = (denominator + q[i]) * reciprocal_square;
        }
        let correction = reciprocal_square * (numerator + p[4]) / (denominator + q[4]);
        (INV_SQRT_PI - correction) / x
    }
}
