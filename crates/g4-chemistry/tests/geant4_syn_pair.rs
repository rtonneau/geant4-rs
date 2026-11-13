//! RR-006: strict source diagnostic consumer; not complete SYN-1 acceptance.
mod common;
use common::{isolated_pair, option3_parameters};
use g4_chemistry::irt::ReactionTime;
use g4_chemistry::molecule::MoleculePair;
use g4_chemistry::syn::{PairContext, sample_pair};
use g4_core::rng::{RngStreams, Stage};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const IDS: [u16; 8] = [1, 2, 8, 9, 11, 17, 42, 51];
const RATIOS: [f64; 4] = [0.5, 1.0, 1.5, 3.0];
const TAUS: [f64; 5] = [0.01, 0.1, 1.0, 10.0, 100.0];
const G4_SEEDS: [u64; 2] = [20261108061, 20261108062];
const RUST_SEEDS: [u64; 2] = [20261108063, 20261108064];
const ROWS: usize = 232;

// Exact binomial-tail inversion for Clopper-Pearson. The summation starts
// at k and proceeds toward the tail, with the PMF initialized in log space.
struct Binomial {
    n: usize,
    log_factorial: Vec<f64>,
}
impl Binomial {
    fn new(n: usize) -> Self {
        let mut f = vec![0.0; n + 1];
        for i in 1..=n {
            f[i] = f[i - 1] + (i as f64).ln();
        }
        Self {
            n,
            log_factorial: f,
        }
    }
    fn tail(&self, k: usize, p: f64, upper: bool) -> f64 {
        if upper && p >= k as f64 / self.n as f64 || !upper && p <= k as f64 / self.n as f64 {
            return 1.0;
        }
        let mut term =
            (self.log_factorial[self.n] - self.log_factorial[k] - self.log_factorial[self.n - k]
                + k as f64 * p.ln()
                + (self.n - k) as f64 * (-p).ln_1p())
            .exp();
        let mut sum = term;
        if upper {
            for i in k..self.n {
                term *= (self.n - i) as f64 / (i + 1) as f64 * p / (1.0 - p);
                sum += term;
                if term <= sum * 1e-15 {
                    break;
                }
            }
        } else {
            for i in (1..=k).rev() {
                term *= i as f64 / (self.n - i + 1) as f64 * (1.0 - p) / p;
                sum += term;
                if term <= sum * 1e-15 {
                    break;
                }
            }
        }
        sum.min(1.0)
    }
    fn interval(&self, k: usize, alpha: f64) -> (f64, f64) {
        let quantile = |upper: bool| {
            let (mut lo, mut hi) = (0.0, 1.0);
            for _ in 0..60 {
                let mid = (lo + hi) / 2.0;
                let tail = self.tail(k, mid, upper);
                if (tail < alpha / 2.0) == upper {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            (lo + hi) / 2.0
        };
        (
            if k == 0 { 0.0 } else { quantile(true) },
            if k == self.n { 1.0 } else { quantile(false) },
        )
    }
}

fn read_counts(csv: &str, n: usize, seed: u64) -> Vec<[usize; 7]> {
    let lines: Vec<_> = csv.lines().collect();
    assert_eq!(
        lines[0],
        "# schema: syn_pair_v1; Geant4: 1141; model: IRT_syn; constructor: G4EmDNAChemistry_option3"
    );
    assert_eq!(
        lines[1],
        format!(
            "# seed: {seed}; samples: {n}; overlap: source branch diagnostic, inherited candidate deliberately unresolved"
        )
    );
    assert_eq!(
        lines[2],
        "reaction_id,context,r_over_a,a_nm,D_nm2_per_ps,contact_probability,endpoint,tau,samples,count"
    );
    assert_eq!(
        lines.last().unwrap(),
        &format!("# complete: true; seed: {seed}; samples: {n}; rows: {ROWS}")
    );
    assert_eq!(lines.len(), ROWS + 4);
    let mut output = vec![];
    let mut row = 3;
    for id in IDS {
        let p = option3_parameters(id);
        for ratio in RATIOS {
            let mut counts = [0; 7];
            for endpoint in 0..7 {
                let columns: Vec<_> = lines[row].split(',').collect();
                row += 1;
                assert_eq!(columns.len(), 10);
                assert_eq!(columns[0].parse::<u16>().unwrap(), id);
                assert_eq!(columns[1], "initial");
                assert_eq!(columns[2].parse::<f64>().unwrap(), ratio);
                for (column, expected) in [
                    (3, p.effective_radius().nm()),
                    (4, p.relative_diffusivity().nm2_per_ps()),
                    (5, p.contact_probability()),
                ] {
                    let actual = columns[column].parse::<f64>().unwrap();
                    assert!(
                        actual.is_finite() && (actual - expected).abs() <= 1e-12 * expected.abs()
                    );
                }
                assert_eq!(
                    columns[6],
                    if endpoint < 5 {
                        "cdf"
                    } else if endpoint == 5 {
                        "no_reaction"
                    } else {
                        "unresolved_overlap"
                    }
                );
                assert_eq!(
                    columns[7].parse::<f64>().unwrap(),
                    if endpoint < 5 { TAUS[endpoint] } else { -1.0 }
                );
                assert_eq!(columns[8].parse::<usize>().unwrap(), n);
                counts[endpoint] = columns[9].parse().unwrap();
                assert!(counts[endpoint] <= n);
            }
            assert!(counts[..5].windows(2).all(|w| w[0] <= w[1]));
            assert!(counts[4] + counts[5] + counts[6] <= n);
            if ratio < 1.0 {
                assert_eq!(counts[5], 0);
                assert_eq!(counts[0] + counts[6], n);
            } else {
                assert_eq!(counts[6], 0);
            }
            output.push(counts);
        }
        let columns: Vec<_> = lines[row].split(',').collect();
        row += 1;
        assert_eq!(columns.len(), 10);
        assert_eq!(columns[0].parse::<u16>().unwrap(), id);
        assert_eq!(&columns[1..3], &["post_start", "0.5"]);
        for (column, expected) in [
            (3, p.effective_radius().nm()),
            (4, p.relative_diffusivity().nm2_per_ps()),
            (5, p.contact_probability()),
        ] {
            let actual = columns[column].parse::<f64>().unwrap();
            assert!((actual - expected).abs() <= 1e-12 * expected.abs());
        }
        assert_eq!(&columns[6..8], &["unresolved_overlap", "-1"]);
        assert_eq!(columns[8].parse::<usize>().unwrap(), n);
        assert_eq!(columns[9].parse::<usize>().unwrap(), n);
    }
    output
}

fn provenance(text: &str, csv: &str, n: usize, seed: u64, replicate: usize) {
    let v: toml::Value = toml::from_str(text).unwrap();
    for (key, expected) in [
        ("geant4_version", "11.4.1"),
        ("timestep_model", "IRT_syn"),
        ("physics_constructor", "G4EmDNAPhysics_option2"),
        ("chemistry_constructor", "G4EmDNAChemistry_option3"),
        ("mode", "syn_pair_matrix"),
        ("schema", "syn_pair_v1"),
        ("g4emlow_version", "8.8"),
    ] {
        assert_eq!(v[key].as_str(), Some(expected));
    }
    assert_eq!(v["seed"].as_integer(), Some(seed as i64));
    assert_eq!(v["samples_per_setting"].as_integer(), Some(n as i64));
    assert_eq!(v["completion_rows"].as_integer(), Some(ROWS as i64));
    assert_eq!(
        v["csv_sha256"].as_str(),
        Some(format!("{:x}", Sha256::digest(csv.as_bytes())).as_str())
    );
    let command = format!(
        "g4rs_ref syn_pair_matrix ../../reference/syn_pair_1141_r{replicate}_n{n}.csv {seed} {n}"
    );
    assert_eq!(v["command"].as_str(), Some(command.as_str()));
    for key in [
        "generator_commit",
        "compiler",
        "platform",
        "date",
        "g4emlow_md5",
        "geant4_source_revision",
        "source_sha256",
        "generator_source_sha256",
        "actual_command",
    ] {
        assert!(!v[key].as_str().unwrap().is_empty(), "missing {key}");
    }
    assert!(v["generator_dirty"].as_bool().is_some());
}

#[test]
fn clopper_pearson_matches_closed_form_and_tabulated_interval() {
    let b = Binomial::new(10);
    let (lo, hi) = b.interval(0, 0.05);
    assert_eq!(lo, 0.0);
    assert!((hi - (1.0 - 0.025_f64.powf(0.1))).abs() < 1e-12);
    let (lo, hi) = b.interval(5, 0.05);
    assert!((lo - 0.1870860284473985).abs() < 1e-12);
    assert!((hi - 0.8129139715526015).abs() < 1e-12);
    let (lo, hi) = b.interval(10, 0.05);
    assert_eq!(hi, 1.0);
    assert!((lo - 0.025_f64.powf(0.1)).abs() < 1e-12);
}

#[test]
#[ignore = "RR-006 requires two complete normative source diagnostic files and provenance"]
fn syn_pair_source_diagnostics_match_geant4() {
    let n: usize = std::env::var("SYN_PAIR_SAMPLES").map_or(100000, |s| s.parse().unwrap());
    assert!([100000, 200000, 1000000].contains(&n));
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../validation/reference");
    let binomial = Binomial::new(n);
    // Includes 232 endpoints per replicate, including deterministic diagnostic rows.
    let alpha = 0.001 / (3.0 * (2 * ROWS) as f64);
    let mut unresolved = 0;
    for replicate in 0..2 {
        let stem = format!("syn_pair_1141_r{}_n{n}", replicate + 1);
        let csv = std::fs::read_to_string(root.join(format!("{stem}.csv")))
            .expect("RR-006 missing CSV")
            .replace("\r\n", "\n");
        provenance(
            &std::fs::read_to_string(root.join(format!("provenance_{stem}.toml")))
                .expect("RR-006 missing provenance"),
            &csv,
            n,
            G4_SEEDS[replicate],
            replicate + 1,
        );
        let reference = read_counts(&csv, n, G4_SEEDS[replicate]);
        let mut rng = RngStreams::new(RUST_SEEDS[replicate])
            .stream(0, Stage::Chemistry)
            .unwrap();
        for (case, id) in IDS.into_iter().enumerate() {
            let p = option3_parameters(id);
            for (distance, ratio) in RATIOS.into_iter().enumerate() {
                let m = isolated_pair(p, ratio * p.effective_radius().nm(), 0);
                let pair = MoleculePair::new(&m[0], &m[1]).unwrap();
                let scale =
                    p.effective_radius().nm().powi(2) / p.relative_diffusivity().nm2_per_ps();
                let mut counts = [0; 7];
                for _ in 0..n {
                    match sample_pair(pair, &p, PairContext::Initial, &mut rng) {
                        Err(g4_chemistry::ChemistryError::IrtDomain {
                            operation: "unresolved source overlap time inheritance",
                            ..
                        }) if ratio < 1.0 => counts[6] += 1,
                        Ok(ReactionTime::NoReaction) => counts[5] += 1,
                        result => {
                            let time = match result.unwrap() {
                                ReactionTime::Immediate => 0.0,
                                ReactionTime::After(t) => t.ps(),
                                _ => unreachable!(),
                            };
                            for (i, tau) in TAUS.iter().enumerate() {
                                if time <= tau * scale {
                                    counts[i] += 1;
                                }
                            }
                        }
                    }
                }
                for endpoint in 0..7 {
                    let ref_count = reference[case * 4 + distance][endpoint];
                    let a = binomial.interval(counts[endpoint], alpha / 2.0);
                    let b = binomial.interval(ref_count, alpha / 2.0);
                    let (lo, hi) = (a.0 - b.1, a.1 - b.0);
                    let verdict = if lo > -0.01 && hi < 0.01 {
                        "PASS"
                    } else if lo > 0.01 || hi < -0.01 {
                        "FAIL"
                    } else {
                        "INCONCLUSIVE"
                    };
                    println!(
                        "replicate={} channel={id} ratio={ratio} endpoint={endpoint} rust={} reference={ref_count} n={n} difference={} interval=[{lo},{hi}] {verdict}",
                        replicate + 1,
                        counts[endpoint],
                        (counts[endpoint] as f64 - ref_count as f64) / n as f64
                    );
                    unresolved += usize::from(verdict != "PASS");
                }
            }
        }
    }
    assert_eq!(
        unresolved, 0,
        "RR-006 source diagnostics not equivalent; this is not full SYN-1 acceptance"
    );
}

// Independently integrate erfcx(x)=2/sqrt(pi)*integral_0^infinity
// exp(-v*v-2*x*v)dv, using v=u/(1-u). This does not call the sampler's
// Cody implementation or inverse-erfc. Refinement differences are checked.
fn integral(f: &impl Fn(f64) -> f64, a: f64, b: f64, tolerance: f64) -> f64 {
    fn refine(
        f: &impl Fn(f64) -> f64,
        a: f64,
        b: f64,
        values: [f64; 3],
        whole: f64,
        tolerance: f64,
        depth: u32,
    ) -> f64 {
        let mid = (a + b) / 2.0;
        let l = f((a + mid) / 2.0);
        let r = f((mid + b) / 2.0);
        let left = (mid - a) * (values[0] + 4.0 * l + values[1]) / 6.0;
        let right = (b - mid) * (values[1] + 4.0 * r + values[2]) / 6.0;
        let difference = left + right - whole;
        if difference.abs() <= 15.0 * tolerance {
            return left + right + difference / 15.0;
        }
        assert!(depth > 0, "independent quadrature failed to converge");
        refine(
            f,
            a,
            mid,
            [values[0], l, values[1]],
            left,
            tolerance / 2.0,
            depth - 1,
        ) + refine(
            f,
            mid,
            b,
            [values[1], r, values[2]],
            right,
            tolerance / 2.0,
            depth - 1,
        )
    }
    let values = [f(a), f((a + b) / 2.0), f(b)];
    let whole = (b - a) * (values[0] + 4.0 * values[1] + values[2]) / 6.0;
    refine(f, a, b, values, whole, tolerance, 30)
}
fn independent_erfcx(x: f64) -> f64 {
    assert!(x >= 0.0 && x.is_finite());
    let f = |u: f64| {
        if u == 1.0 {
            return 0.0;
        }
        let v = u / (1.0 - u);
        (-v * v - 2.0 * x * v).exp() / (1.0 - u).powi(2)
    };
    integral(&f, 0.0, 1.0, 1e-12) * 2.0 / std::f64::consts::PI.sqrt()
}
fn analytic_cdf(ratio: f64, tau: f64, rate: Option<f64>) -> f64 {
    let z = (ratio - 1.0) / (2.0 * tau.sqrt());
    let encounter = independent_erfcx(z);
    let probability = match rate {
        None => encounter,
        Some(p) => p * (encounter - independent_erfcx(z + tau.sqrt() / (1.0 - p))),
    };
    (-z * z).exp() * probability / ratio
}

#[test]
fn independent_delay_convolution_matches_integrated_closed_form() {
    assert!((independent_erfcx(0.0) - 1.0).abs() < 1e-11);
    for id in [11, 17, 42, 51] {
        let p = option3_parameters(id);
        let rates = p.pdc().unwrap();
        let rate = rates.activation_rate().nm3_per_ps()
            / (rates.activation_rate().nm3_per_ps() + rates.diffusion_rate().nm3_per_ps());
        for ratio in [1.5, 3.0] {
            for tau in TAUS {
                let gap: f64 = ratio - 1.0;
                // s=tau*sin(theta)^2 removes the square-root endpoint of the
                // delay CDF while the first-passage exponential suppresses s=0.
                let integrand = |theta: f64| {
                    let s = tau * theta.sin().powi(2);
                    if s == 0.0 || s >= tau {
                        return 0.0;
                    }
                    let density = gap * (-gap * gap / (4.0 * s)).exp()
                        / (2.0 * std::f64::consts::PI.sqrt() * s.powf(1.5));
                    density
                        * (1.0 - independent_erfcx((tau - s).sqrt() / (1.0 - rate)))
                        * tau
                        * (2.0 * theta).sin()
                };
                let fine =
                    integral(&integrand, 0.0, std::f64::consts::FRAC_PI_2, 1e-10) * rate / ratio;
                let finer =
                    integral(&integrand, 0.0, std::f64::consts::FRAC_PI_2, 1e-11) * rate / ratio;
                assert!((fine - finer).abs() < 1e-8);
                assert!(
                    (finer - analytic_cdf(ratio, tau, Some(rate))).abs() < 1e-8,
                    "channel {id} ratio {ratio} tau {tau}"
                );
            }
        }
    }
}

#[test]
#[ignore = "SYN-1 independent analytic probability gate; frozen manifest in README"]
fn syn_pair_matches_independent_analytic_law() {
    let n: usize = std::env::var("SYN_ANALYTIC_SAMPLES").map_or(100000, |s| s.parse().unwrap());
    assert!([100000, 200000, 1000000].contains(&n));
    let binomial = Binomial::new(n);
    let alpha = 0.001 / (3.0 * (2 * ROWS) as f64);
    let mut unresolved = 0;
    for (replicate, seed) in [20261108065, 20261108066].into_iter().enumerate() {
        let mut rng = RngStreams::new(seed).stream(0, Stage::Chemistry).unwrap();
        for id in IDS {
            let p = option3_parameters(id);
            let rate = p.pdc().map(|v| {
                v.activation_rate().nm3_per_ps()
                    / (v.activation_rate().nm3_per_ps() + v.diffusion_rate().nm3_per_ps())
            });
            for ratio in RATIOS {
                let m = isolated_pair(p, ratio * p.effective_radius().nm(), 0);
                let pair = MoleculePair::new(&m[0], &m[1]).unwrap();
                let scale =
                    p.effective_radius().nm().powi(2) / p.relative_diffusivity().nm2_per_ps();
                let mut counts = [0; 7];
                for _ in 0..n {
                    match sample_pair(pair, &p, PairContext::Initial, &mut rng) {
                        Err(g4_chemistry::ChemistryError::IrtDomain {
                            operation: "unresolved source overlap time inheritance",
                            ..
                        }) if ratio < 1.0 => counts[6] += 1,
                        Ok(ReactionTime::NoReaction) => counts[5] += 1,
                        result => {
                            let t = match result.unwrap() {
                                ReactionTime::Immediate => 0.0,
                                ReactionTime::After(t) => t.ps(),
                                _ => unreachable!(),
                            };
                            for (i, tau) in TAUS.iter().enumerate() {
                                if t <= tau * scale {
                                    counts[i] += 1;
                                }
                            }
                        }
                    }
                }
                for endpoint in 0..7 {
                    let expected = if ratio < 1.0 {
                        if endpoint < 5 {
                            p.contact_probability()
                        } else if endpoint == 5 {
                            0.0
                        } else {
                            1.0 - p.contact_probability()
                        }
                    } else if endpoint < 5 {
                        analytic_cdf(ratio, TAUS[endpoint], rate)
                    } else if endpoint == 5 {
                        1.0 - rate.unwrap_or(1.0) / ratio
                    } else {
                        0.0
                    };
                    let interval = binomial.interval(counts[endpoint], alpha);
                    // Account explicitly for the independently checked quadrature accuracy.
                    let (lo, hi) = (interval.0 - expected - 1e-8, interval.1 - expected + 1e-8);
                    let verdict = if lo > -0.01 && hi < 0.01 {
                        "PASS"
                    } else if lo > 0.01 || hi < -0.01 {
                        "FAIL"
                    } else {
                        "INCONCLUSIVE"
                    };
                    println!(
                        "analytic replicate={} channel={id} ratio={ratio} endpoint={endpoint} count={} n={n} oracle={expected} difference={} interval=[{lo},{hi}] {verdict}",
                        replicate + 1,
                        counts[endpoint],
                        counts[endpoint] as f64 / n as f64 - expected
                    );
                    unresolved += usize::from(verdict != "PASS");
                }
            }
            let m = isolated_pair(p, 0.5 * p.effective_radius().nm(), 0);
            assert!(matches!(
                sample_pair(
                    MoleculePair::new(&m[0], &m[1]).unwrap(),
                    &p,
                    PairContext::PostStart,
                    &mut rng
                ),
                Err(g4_chemistry::ChemistryError::IrtDomain {
                    operation: "unresolved source overlap time inheritance",
                    ..
                })
            ));
        }
    }
    assert_eq!(
        unresolved, 0,
        "SYN-1 analytic gate has unresolved endpoints"
    );
}
