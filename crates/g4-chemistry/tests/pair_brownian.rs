//! Run with --nocapture; INCONCLUSIVE is a scientific verdict, not a test failure.
#[path = "pair_oracle/charged.rs"]
mod charged;
#[path = "pair_oracle/escape.rs"]
mod escape;
mod pair_oracle;
use pair_oracle::{STEPS, TIMES, comparison, neutral::counts};
use serde::Deserialize;

#[derive(Deserialize)]
struct Anchors {
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    name: String,
    id: u16,
    radius: f64,
    diffusion: f64,
    rate_per_molar_s: f64,
    points: Vec<Point>,
}
#[derive(Deserialize)]
struct Point {
    ratio: f64,
    probabilities: [f64; 6],
}

#[test]
fn neutral_brownian() {
    pair_oracle::experiment::run(false);
}

#[test]
fn neutral_precision_gate_rejects_underpowered_agreement() {
    assert!(!comparison(5_000, 5_000, 10_000).2);
    assert!(!comparison(500_000, 510_000, 1_000_000).2);
    assert!(comparison(500_000, 500_000, 1_000_000).2);
    assert_eq!(counts(0, 0, 1.0, None, 0, STEPS[0], 10_000), [10_000; 5]);
}

#[test]
fn escape_bounds_keep_unresolved_trajectories_interval_censored() {
    assert!((escape::return_probability(0.0, 400.0) - 0.0025).abs() < 1e-15);
    assert!(
        (escape::return_probability(-1.0, 400.0) - (-0.0025_f64).exp_m1() / (-1.0_f64).exp_m1())
            .abs()
            < 1e-15
    );
    assert!(escape::return_probability(1.0, 400.0) < 0.0025);
    assert!((escape::return_probability(1000.0, 2.0) / (-500.0_f64).exp() - 1.0).abs() < 1e-12);
    let (lo, hi) = escape::reaction_interval(300, 500, 200, 1000, 0.01);
    assert!((lo - 0.3).abs() < 1e-15);
    assert!((hi - 0.505).abs() < 1e-15);
}

#[derive(Deserialize)]
struct PhysicalAnchors {
    z_threshold: f64,
    cases: Vec<PhysicalCase>,
}
#[derive(Deserialize)]
struct PhysicalCase {
    name: String,
    charge: f64,
    kappa: Option<f64>,
    points: Vec<PhysicalPoint>,
}
#[derive(Deserialize)]
struct PhysicalPoint {
    ratio: f64,
    drift: f64,
    eventual: f64,
}

fn physical_anchors() -> PhysicalAnchors {
    serde_json::from_str(include_str!("fixtures/pair_coulomb_anchors.json")).unwrap()
}

#[test]
fn charged_brownian() {
    pair_oracle::experiment::run(true);
}

#[test]
fn charged_physical_drift_and_boundary_anchors() {
    for case in physical_anchors().cases {
        for p in case.points {
            assert!((charged::drift(p.ratio, case.charge) - p.drift).abs() < 1e-13);
            let contact_fraction = case
                .kappa
                .map_or(1.0, |k| k / (k + case.charge / -(-case.charge).exp_m1()));
            let eventual =
                contact_fraction * (case.charge / p.ratio).exp_m1() / case.charge.exp_m1();
            assert!((eventual - p.eventual).abs() < 1e-14);
        }
    }
    assert_eq!(
        charged::counts(2, 0, 1.0, 1.0, None, 0, STEPS[0], 10_000),
        [10_000; 5]
    );
    // Identical streams at zero charge must recover the independent neutral integrator.
    assert_eq!(
        charged::counts(2, 1, 1.5, 0.0, Some(1.0), 0, STEPS[0], 100),
        counts(2, 1, 1.5, Some(1.0), 0, STEPS[0], 100)
    );
    assert!(charged::drift(1.5, 1.0) > charged::drift(1.5, 0.0));
    assert!(charged::drift(1.5, -1.0) < charged::drift(1.5, 0.0));
}
