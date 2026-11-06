//! Registered schedule and scientific reporting, separate from executable success.
use super::{SIZES, STEPS, TIMES, comparison, stopped};
use crate::{Anchors, Case, physical_anchors};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct FiniteAnchors {
    cases: Vec<FiniteCase>,
}
#[derive(Deserialize)]
struct FiniteCase {
    name: String,
    points: Vec<FinitePoint>,
}
#[derive(Deserialize)]
struct FinitePoint {
    ratio: f64,
    probabilities: [f64; 5],
    errors: [f64; 5],
}

fn neutral_kappa(case: &Case) -> Option<f64> {
    if matches!(case.id, 11 | 17) {
        let aa = if case.id == 17 { 0.5 } else { 1.0 };
        let kd = 4.0 * std::f64::consts::PI * case.diffusion * case.radius * aa;
        let f = case.rate_per_molar_s * 1e12 / 6.022_140_76e23 / kd;
        assert!(f > 0.0 && f < 1.0);
        Some(f / (1.0 - f))
    } else {
        None
    }
}

/// A numerical envelope must be overcome before a discrepancy is called resolved.
fn law_verdict(observed: f64, expected: f64, error: f64, se: f64, z: f64) -> &'static str {
    let difference = (observed - expected).abs();
    if z * se + error >= 0.005 {
        "INCONCLUSIVE"
    } else if difference > error + z * se {
        "FAIL"
    } else if difference + error + z * se < 0.005 {
        "PASS"
    } else {
        "INCONCLUSIVE"
    }
}

pub fn run(charged: bool) {
    let analytic: Anchors =
        serde_json::from_str(include_str!("../fixtures/pair_analytic_anchors.json")).unwrap();
    let physical = physical_anchors();
    let finite: FiniteAnchors =
        serde_json::from_str(include_str!("../fixtures/pair_coulomb_finite_anchors.json")).unwrap();
    let cap = std::env::var("PAIR_ORACLE_MAX_PAIRS").map_or(10_000, |s| s.parse().unwrap());
    let levels = std::env::var("PAIR_ORACLE_LEVELS").map_or(3, |s| s.parse::<usize>().unwrap());
    let radius = std::env::var("PAIR_ORACLE_RADIUS").map_or(16.0, |s| s.parse().unwrap());
    let horizon = std::env::var("PAIR_ORACLE_HORIZON").map_or(100.0, |s| s.parse().unwrap());
    let filter = std::env::var("PAIR_ORACLE_CASE").ok();
    assert!(SIZES.contains(&cap) && (3..=5).contains(&levels));
    assert!(radius > 3.0 && horizon >= 100.0);
    eprintln!(
        "R04_JSON {}",
        json!({"record":"run", "charged":charged,
        "seed":20261027402_u64,"cap":cap,"levels":levels,"radius":radius,"horizon":horizon,
        "criterion":"frozen; censored eventual endpoint cannot certify convergence",
        "case_filter":filter})
    );
    for (case_index, case) in analytic.cases.iter().enumerate() {
        let physical_case = physical.cases.iter().find(|p| p.name == case.name);
        if physical_case.is_some() != charged || filter.as_ref().is_some_and(|f| f != &case.name) {
            continue;
        }
        let charge = physical_case.map_or(0.0, |p| p.charge);
        let kappa = physical_case.map_or_else(|| neutral_kappa(case), |p| p.kappa);
        let q = crate::escape::return_probability(charge, radius);
        for (distance, point) in case.points.iter().enumerate() {
            let pde = finite
                .cases
                .iter()
                .find(|p| p.name == case.name)
                .map(|p| &p.points[distance]);
            assert_eq!(
                pde.is_some(),
                charged,
                "missing or unexpected physical PDE case"
            );
            if let Some(pde) = pde {
                assert_eq!(pde.ratio, point.ratio);
            }
            let mut cached = [stopped::Counts::default(); 5];
            let mut final_n = 0;
            let mut final_level = 0;
            let mut finite_status = ["INCONCLUSIVE"; 5];
            for &n in SIZES.iter().take_while(|&&n| n <= cap) {
                let mut precise = false;
                for (level, &h) in STEPS.iter().take(levels).enumerate() {
                    let params = stopped::Params {
                        case: case_index,
                        distance,
                        ratio: point.ratio,
                        charge,
                        kappa,
                        level,
                        h,
                        outer_radius: radius,
                        horizon,
                    };
                    let added = stopped::counts(&params, cached[level].n()..n);
                    cached[level] = cached[level].checked_add(added).unwrap();
                    let current = cached[level];
                    assert_eq!(current.n(), n);
                    assert!(current.finite.windows(2).all(|p| p[0] <= p[1]));
                    precise = level > 0;
                    let mut finite_converged = level >= 2;
                    for endpoint in 0..5 {
                        let (delta, width, ok) = if level == 0 {
                            (f64::NAN, f64::INFINITY, false)
                        } else {
                            comparison(
                                cached[level - 1].finite[endpoint],
                                current.finite[endpoint],
                                n,
                            )
                        };
                        precise &= width < 0.005;
                        finite_converged &= ok;
                        let observed = current.finite[endpoint] as f64 / n as f64;
                        let stabilized = (current.finite[endpoint] as f64 + 0.5) / (n as f64 + 1.0);
                        let exact_contact = kappa.is_none() && point.ratio == 1.0;
                        let se = if exact_contact {
                            0.0
                        } else {
                            (stabilized * (1.0 - stabilized) / n as f64).sqrt()
                        };
                        let expected = pde
                            .map_or(point.probabilities[endpoint], |p| p.probabilities[endpoint]);
                        let error = pde.map_or(0.0, |p| p.errors[endpoint]);
                        let verdict = if ok && level >= 2 {
                            law_verdict(observed, expected, error, se, physical.z_threshold)
                        } else {
                            "INCONCLUSIVE"
                        };
                        finite_status[endpoint] = verdict;
                        precise &= physical.z_threshold * se + error < 0.005;
                        eprintln!(
                            "R04_JSON {}",
                            json!({"record":"finite", "case":case.name,
                            "distance":point.ratio,"n":n,"level":level,"h":h,"tau":TIMES[endpoint],
                            "count":current.finite[endpoint],"cdf":observed,"physical_anchor":expected,
                            "anchor_error_estimate":error,"source_law_anchor":point.probabilities[endpoint],
                            "difference":observed-expected,"z":if se==0.0 && observed==expected {
                                0.0 } else { (observed-expected)/se },
                            "z_threshold":physical.z_threshold,"delta":delta,"three_se":width,
                            "refined":ok && level>=2,"physical_numerical_diagnostic":verdict})
                        );
                    }
                    let mut category_precision = level > 0;
                    if level > 0 {
                        for (a, b) in [
                            (cached[level - 1].reacted, current.reacted),
                            (cached[level - 1].escaped, current.escaped),
                        ] {
                            category_precision &= comparison(a, b, n).1 < 0.005;
                        }
                    }
                    precise &= category_precision;
                    let identification = current.nonreaction_interval(q);
                    let interval = if kappa.is_none() && point.ratio == 1.0 {
                        assert_eq!(current.reacted, n);
                        (0.0, 0.0)
                    } else {
                        current.nonreaction_band(q)
                    };
                    eprintln!(
                        "R04_JSON {}",
                        json!({"record":"eventual","case":case.name,
                        "distance":point.ratio,"n":n,"level":level,"h":h,
                        "reacted":current.reacted,"escaped":current.escaped,"unresolved":current.unresolved,
                        "return_bound":q,"nonreaction_identification":identification,
                        "nonreaction_mc_interval":interval,"physical_anchor":physical_case.map_or(
                            point.probabilities[5], |p| 1.0-p.points[distance].eventual),
                        "source_law_anchor":point.probabilities[5],
                        "discretization_bound":null,"verdict":if kappa.is_none() && point.ratio==1.0 {
                            "PASS(exact-contact)" } else {"INCONCLUSIVE"}})
                    );
                    final_n = n;
                    final_level = level;
                    // First resolve Monte Carlo width on the first three levels.
                    // Additional levels cannot fix an inadequate MC width. Neither
                    // this expansion nor stopping consults law-test significance.
                    let exact_eventual = kappa.is_none() && point.ratio == 1.0;
                    if level >= 2 && (!precise || (finite_converged && exact_eventual)) {
                        break;
                    }
                }
                if precise {
                    break;
                }
            }
            let exact_contact_pass =
                kappa.is_none() && point.ratio == 1.0 && finite_status.iter().all(|&s| s == "PASS");
            eprintln!(
                "R04_JSON {}",
                json!({"record":"summary","case":case.name,
                "distance":point.ratio,"n":final_n,"level":final_level,
                "finite_physical_diagnostics":finite_status,"brownian_verdict":if exact_contact_pass {
                    "PASS(exact-physical-contact-only)" } else { "INCONCLUSIVE" },
                "reason":if exact_contact_pass {"deterministic physical absorbing contact"} else {
                    "eventual interval censoring and continuum discretization not resolved"},
                "g1":"OPEN"})
            );
        }
    }
    eprintln!(
        "R04_JSON {}",
        json!({"record":"complete","charged":charged,"g1":"OPEN"})
    );
}

#[test]
fn law_gate_distinguishes_precision_from_discrepancy() {
    assert_eq!(law_verdict(0.5, 0.5, 0.0, 0.01, 4.58), "INCONCLUSIVE");
    assert_eq!(law_verdict(0.6, 0.5, 0.0, 0.0001, 4.58), "FAIL");
    assert_eq!(law_verdict(0.5, 0.5, 0.0001, 0.0001, 4.58), "PASS");
    assert_eq!(law_verdict(0.5, 0.5, 0.01, 0.0001, 4.58), "INCONCLUSIVE");
}
