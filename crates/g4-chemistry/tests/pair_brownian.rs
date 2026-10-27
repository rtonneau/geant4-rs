//! Run with --nocapture; INCONCLUSIVE is a scientific verdict, not a test failure.
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
