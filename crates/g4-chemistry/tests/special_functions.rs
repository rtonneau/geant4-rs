//! Independent high-precision checks of the private IRT numerical kernel.
//!
//! Compile the production module here so the integration-test entry point does not
//! require making a numerical implementation detail part of the public API.

mod error {
    pub use g4_chemistry::ChemistryError;
}
#[path = "../src/special.rs"]
mod special;

use g4_chemistry::ChemistryError;
use serde::Deserialize;
use special::{erfc, erfc_inv};

#[derive(Deserialize)]
struct Value {
    input: String,
    expected: String,
}

#[derive(Deserialize)]
struct Fixtures {
    erfc: Vec<Value>,
    erfc_inv: Vec<Value>,
}

fn fixtures() -> Fixtures {
    serde_json::from_str(include_str!("fixtures/special_functions.json")).unwrap()
}

#[test]
fn erfc_matches_independent_high_precision_values() {
    for row in fixtures().erfc {
        let x = row.input.parse().unwrap();
        let expected: f64 = row.expected.parse().unwrap();
        let actual = erfc(x).unwrap();
        // Relative accuracy throughout the normal tails; one representable step
        // for subnormals, whose binary64 spacing can exceed relative 1e-14.
        let tolerance = (expected.abs() * 1e-14).max(f64::from_bits(1));
        assert!(actual.is_finite());
        assert!(
            (actual - expected).abs() <= tolerance,
            "erfc({x}) = {actual:e}, reference {expected:e}"
        );
    }
}

#[test]
fn inverse_matches_independent_high_precision_values() {
    for row in fixtures().erfc_inv {
        let p = row.input.parse().unwrap();
        let expected: f64 = row.expected.parse().unwrap();
        let actual = erfc_inv(p).unwrap();
        assert!(actual.is_finite());
        assert!(
            (actual - expected).abs() <= 1e-14,
            "erfc_inv({p:e}) = {actual:.17}, reference {expected:.17}"
        );
    }
}

#[test]
fn boundaries_are_explicit_and_invalid_inputs_are_typed_errors() {
    assert_eq!(erfc(-0.0).unwrap(), 1.0);
    assert_eq!(erfc(f64::MAX).unwrap(), 0.0);
    assert_eq!(erfc(-f64::MAX).unwrap(), 2.0);
    assert_eq!(erfc_inv(1.0).unwrap(), 0.0);
    assert!(erfc_inv(f64::from_bits(1)).unwrap().is_finite());
    assert!(
        erfc_inv(f64::from_bits(2.0_f64.to_bits() - 1))
            .unwrap()
            .is_finite()
    );

    for x in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            erfc(x),
            Err(ChemistryError::SpecialFunctionDomain {
                function: "erfc",
                ..
            })
        ));
    }
    // Infinite inverse limits at zero and two are outside the finite API domain.
    for p in [
        -1.0,
        -f64::from_bits(1),
        -0.0,
        0.0,
        2.0,
        f64::from_bits(2.0_f64.to_bits() + 1),
        f64::MAX,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        assert!(matches!(
            erfc_inv(p),
            Err(ChemistryError::SpecialFunctionDomain {
                function: "erfc_inv",
                ..
            })
        ));
    }
}

#[test]
fn functions_are_monotone_across_central_values_tails_and_interval_boundaries() {
    let mut previous = 2.0;
    for i in -28_000..=28_000 {
        let x = f64::from(i) / 1_000.0;
        let value = erfc(x).unwrap();
        assert!(value.is_finite() && (0.0..=2.0).contains(&value));
        assert!(value <= previous, "erfc increases at {x}");
        previous = value;
    }
    for boundary in [0.46875_f64, 4.0, 28.0] {
        let lower = erfc(f64::from_bits(boundary.to_bits() - 1)).unwrap();
        let middle = erfc(boundary).unwrap();
        let upper = erfc(f64::from_bits(boundary.to_bits() + 1)).unwrap();
        assert!(lower >= middle && middle >= upper, "boundary {boundary}");
    }

    let mut probabilities = (1..20_000)
        .map(|i| f64::from(i) / 10_000.0)
        .collect::<Vec<_>>();
    let mut power_of_two = 1.0;
    for _ in 1..=1074 {
        power_of_two *= 0.5;
        probabilities.push(power_of_two);
    }
    probabilities.extend((1..=52).map(|i| 2.0 - 2.0_f64.powi(-i)));
    probabilities.sort_by(f64::total_cmp);
    let mut previous = f64::INFINITY;
    for p in probabilities {
        let value = erfc_inv(p).unwrap();
        assert!(value.is_finite());
        assert!(value <= previous, "inverse increases at {p:e}");
        previous = value;
    }
}

#[test]
fn symmetry_and_round_trips() {
    for i in 0..=512 {
        let x = f64::from(i) / 64.0;
        // The two subtractions may round differently in the central interval.
        assert!((erfc(-x).unwrap() - (2.0 - erfc(x).unwrap())).abs() <= 1e-14);
    }
    // Dyadic p makes 2-p exact, so this tests mathematical symmetry without
    // injecting asymmetric cancellation from the test's own probability input.
    for i in 1..=1024 {
        let p = f64::from(i) / 1024.0;
        assert_eq!(erfc_inv(2.0 - p).unwrap(), -erfc_inv(p).unwrap());
    }
    // Forward rounding erases information in the far negative tail and in
    // subnormal probabilities. Restrict x round trips to the well-resolved range.
    for i in -64..=1664 {
        let x = f64::from(i) / 64.0;
        let round_trip = erfc_inv(erfc(x).unwrap()).unwrap();
        assert!(
            (round_trip - x).abs() <= 1e-14,
            "x round trip at {x}: {round_trip}"
        );
    }
    for row in fixtures().erfc_inv {
        let p: f64 = row.input.parse().unwrap();
        let round_trip = erfc(erfc_inv(p).unwrap()).unwrap();
        assert!((round_trip - p).abs() <= 1e-14, "p round trip at {p:e}");
    }
}

#[test]
fn erfc_central_values() {
    assert_eq!(erfc(0.0).unwrap(), 1.0);
    let expected = 0.157_299_207_050_285_13;
    assert!((erfc(1.0).unwrap() - expected).abs() < 1e-14 * expected);
}
