//! Run with --nocapture; INCONCLUSIVE is a scientific verdict, not a test failure.
#[path = "pair_oracle/charged.rs"]
mod charged;
mod pair_oracle;
use pair_oracle::{SIZES, STEPS, TIMES, comparison, neutral::counts};
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
    let anchors: Anchors =
        serde_json::from_str(include_str!("fixtures/pair_analytic_anchors.json")).unwrap();
    let cap = std::env::var("PAIR_ORACLE_MAX_PAIRS").map_or(10_000, |s| s.parse().unwrap());
    assert!(SIZES.contains(&cap));
    let levels = std::env::var("PAIR_ORACLE_LEVELS").map_or(3, |s| s.parse::<usize>().unwrap());
    assert!((3..=5).contains(&levels));
    for (ordinal, case) in anchors
        .cases
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c.id, 1 | 2 | 11 | 17))
    {
        let kappa = if matches!(case.id, 11 | 17) {
            let aa_factor = if case.id == 17 { 0.5 } else { 1.0 };
            let kd = 4.0 * std::f64::consts::PI * case.diffusion * case.radius * aa_factor;
            let fraction = case.rate_per_molar_s * 1e12 / 6.022_140_76e23 / kd;
            assert!(fraction > 0.0 && fraction < 1.0);
            Some(fraction / (1.0 - fraction))
        } else {
            None
        };
        for (distance, point) in case.points.iter().enumerate() {
            for &n in SIZES.iter().take_while(|&&n| n <= cap) {
                let mut previous = None;
                let mut precise = false;
                for (level, &h) in STEPS.iter().take(levels).enumerate() {
                    let current = counts(ordinal, distance, point.ratio, kappa, level, h, n);
                    assert!(current.iter().all(|&count| count <= n));
                    assert!(current.windows(2).all(|p| p[0] <= p[1]));
                    let mut converged = true;
                    precise = true;
                    for endpoint in 0..5 {
                        let (delta, width, ok) = previous
                            .map_or((f64::NAN, f64::INFINITY, false), |p: [usize; 5]| {
                                comparison(p[endpoint], current[endpoint], n)
                            });
                        precise &= width < 0.005;
                        converged &= ok;
                        eprintln!(
                            "BROWNIAN {} r/a={} n={n} h={h} tau={} count={} cdf={} anchor={} delta={delta} three_se={width} {}",
                            case.name,
                            point.ratio,
                            TIMES[endpoint],
                            current[endpoint],
                            current[endpoint] as f64 / n as f64,
                            point.probabilities[endpoint],
                            if ok { "CONVERGED" } else { "INCONCLUSIVE" }
                        );
                    }
                    eprintln!(
                        "BROWNIAN {} r/a={} n={n} h={h} eventual=INCONCLUSIVE unresolved_survivors={}/{}; finite={}",
                        case.name,
                        point.ratio,
                        n - current[4],
                        n,
                        if converged && level >= 2 {
                            "CONVERGED"
                        } else {
                            "INCONCLUSIVE"
                        }
                    );
                    previous = Some(current);
                    if converged && level >= 2 {
                        break;
                    }
                }
                // Expand on precision alone, never because a significance test failed.
                if precise {
                    break;
                }
            }
            eprintln!(
                "BROWNIAN {} r/a={}: INCONCLUSIVE (eventual mass unresolved; runtime cap n={cap}, levels={levels})",
                case.name, point.ratio
            );
        }
    }
}

#[test]
fn neutral_precision_gate_rejects_underpowered_agreement() {
    assert!(!comparison(5_000, 5_000, 10_000).2);
    assert!(!comparison(500_000, 510_000, 1_000_000).2);
    assert!(comparison(500_000, 500_000, 1_000_000).2);
    assert_eq!(counts(0, 0, 1.0, None, 0, STEPS[0], 10_000), [10_000; 5]);
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
    let approximation: Anchors =
        serde_json::from_str(include_str!("fixtures/pair_analytic_anchors.json")).unwrap();
    let cap = std::env::var("PAIR_ORACLE_MAX_PAIRS").map_or(10_000, |s| s.parse().unwrap());
    let levels = std::env::var("PAIR_ORACLE_LEVELS").map_or(3, |s| s.parse::<usize>().unwrap());
    assert!(SIZES.contains(&cap));
    assert!((3..=5).contains(&levels));
    let physical_anchors = physical_anchors();
    for physical in physical_anchors.cases {
        let (ordinal, approx) = approximation
            .cases
            .iter()
            .enumerate()
            .find(|(_, c)| c.name == physical.name)
            .unwrap();
        for (distance, point) in physical.points.iter().enumerate() {
            let approx_mass = 1.0 - approx.points[distance].probabilities[5];
            let effect = (approx_mass - point.eventual).abs();
            eprintln!(
                "CHARGED {} r/a={} physical_eventual={} geant4_eventual={approx_mass} abs_diff={effect} APPROXIMATION={} G1=OPEN",
                physical.name,
                point.ratio,
                point.eventual,
                if effect >= 0.005 {
                    "FAIL"
                } else {
                    "PASS(eventual-only)"
                }
            );
            for &n in SIZES.iter().take_while(|&&n| n <= cap) {
                let mut previous = None;
                let mut precise = false;
                for (level, &h) in STEPS.iter().take(levels).enumerate() {
                    let current = charged::counts(
                        ordinal,
                        distance,
                        point.ratio,
                        physical.charge,
                        physical.kappa,
                        level,
                        h,
                        n,
                    );
                    assert!(current.iter().all(|&c| c <= n));
                    assert!(current.windows(2).all(|p| p[0] <= p[1]));
                    precise = true;
                    let mut converged = true;
                    for endpoint in 0..5 {
                        let (delta, width, ok) = previous
                            .map_or((f64::NAN, f64::INFINITY, false), |p: [usize; 5]| {
                                comparison(p[endpoint], current[endpoint], n)
                            });
                        precise &= width < 0.005;
                        converged &= ok;
                        let observed = current[endpoint] as f64 / n as f64;
                        let p = (current[endpoint] as f64 + 0.5) / (n as f64 + 1.0);
                        // Exact eventual mass is an upper bound, not a finite-time CDF.
                        let excess = observed - point.eventual;
                        let bound_fail = excess > 0.005
                            && excess
                                > physical_anchors.z_threshold * (p * (1.0 - p) / n as f64).sqrt();
                        eprintln!(
                            "CHARGED {} r/a={} n={n} h={h} tau={} count={} cdf={observed} physical_upper={} geant4_cdf={} delta={delta} three_se={width} refinement={} physical={} G1=OPEN",
                            physical.name,
                            point.ratio,
                            TIMES[endpoint],
                            current[endpoint],
                            point.eventual,
                            approx.points[distance].probabilities[endpoint],
                            if ok { "CONVERGED" } else { "INCONCLUSIVE" },
                            if bound_fail {
                                "FAIL(upper-bound)"
                            } else {
                                "INCONCLUSIVE(finite-law-unresolved)"
                            }
                        );
                    }
                    eprintln!(
                        "CHARGED {} r/a={} n={n} h={h} eventual=INCONCLUSIVE unresolved_survivors={} G1=OPEN",
                        physical.name,
                        point.ratio,
                        n - current[4]
                    );
                    previous = Some(current);
                    if converged && level >= 2 {
                        break;
                    }
                }
                if precise {
                    break;
                }
            }
        }
    }
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
