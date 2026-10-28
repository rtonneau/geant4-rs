//! R04 analytic family. Changing the sampler time scale, mass at infinity,
//! A+A factor, or charged mapping must fail these independent-anchor checks.
mod common;

use common::{isolated_pair, option3_parameters};
use g4_chemistry::irt::{ReactionTime, sample_independent_reaction_time};
use g4_chemistry::molecule::MoleculePair;
use g4_chemistry::reaction_parameters::TwoBodyReactionParameters;
use g4_chemistry::reactions::{ObservedRate, ReactionTable};
use g4_chemistry::species::SpeciesTable;
use g4_core::rng::{RngStreams, Stage};
use g4_core::units::{Diffusivity, Length};
use serde::Deserialize;

const N: usize = 100_000;
const TAU: [f64; 5] = [0.01, 0.1, 1.0, 10.0, 100.0];

#[derive(Deserialize)]
struct Anchors {
    family_alpha: f64,
    endpoints: usize,
    z_threshold: f64,
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
fn anchors() -> Anchors {
    serde_json::from_str(include_str!("fixtures/pair_analytic_anchors.json")).unwrap()
}
fn parameters(case: &Case) -> TwoBodyReactionParameters {
    if !case.name.starts_with("syn-") {
        return option3_parameters(case.id);
    }
    // Synthetic dimensional embedding: each D=1/2, each radius=a/2.
    // Charged a equals the unit Onsager length, hence rc/a=+/-1.
    let table = ReactionTable::option3().unwrap();
    let mut reaction = table.get(case.id).unwrap().clone();
    reaction.rate = ObservedRate::Bimolecular {
        per_molar_s: case.rate_per_molar_s,
    };
    let species = SpeciesTable::option3().unwrap();
    let props = option3_parameters(case.id).reactants().map(|s| {
        let mut p = *species.get(s);
        p.diffusivity = Diffusivity::from_nm2_per_ps(0.5);
        p.radius = Length::from_nm(case.radius / 2.0);
        p
    });
    TwoBodyReactionParameters::from_species(&reaction, props).unwrap()
}
fn accepted(count: usize, p: f64, threshold: f64) -> bool {
    let difference = (count as f64 / N as f64 - p).abs();
    difference <= threshold * (p * (1.0 - p) / N as f64).sqrt()
}
fn counts(case: &Case, ordinal: usize, distance: usize, time_bias: f64) -> [usize; 6] {
    let parameters = parameters(case);
    let radius = parameters.reaction_radius().nm();
    assert!((radius / case.radius - 1.0).abs() < 1e-14);
    assert!((parameters.relative_diffusivity().nm2_per_ps() / case.diffusion - 1.0).abs() < 1e-14);
    let molecules = isolated_pair(parameters, radius * case.points[distance].ratio, 0);
    let pair = MoleculePair::new(&molecules[0], &molecules[1]).unwrap();
    let streams = RngStreams::new(20_261_027_401);
    let mut counts = [0; 6];
    for i in 0..N {
        let history = ((ordinal * 3 + distance) * N + i) as u64;
        let mut rng = streams.stream(history, Stage::Chemistry).unwrap();
        let tau = match sample_independent_reaction_time(pair, &parameters, &mut rng).unwrap() {
            ReactionTime::Immediate => 0.0,
            ReactionTime::After(t) => time_bias * t.ps() * case.diffusion / (radius * radius),
            ReactionTime::NoReaction => {
                counts[5] += 1;
                continue;
            }
        };
        for (j, observation) in TAU.iter().enumerate() {
            counts[j] += usize::from(tau <= *observation);
        }
    }
    counts
}
#[test]
fn analytic_family() {
    let anchors = anchors();
    assert_eq!(anchors.family_alpha, 0.001);
    assert_eq!(anchors.endpoints, 216);
    assert_eq!(anchors.cases.len(), 12);
    let mut failures = Vec::new();
    for (ordinal, case) in anchors.cases.iter().enumerate() {
        assert_eq!(case.points.len(), 3);
        let mut passed = true;
        for (distance, point) in case.points.iter().enumerate() {
            let counts = counts(case, ordinal, distance, 1.0);
            for (j, (&count, &p)) in counts.iter().zip(&point.probabilities).enumerate() {
                assert!((0.0..=1.0).contains(&p));
                let difference = count as f64 / N as f64 - p;
                let se = (p * (1.0 - p) / N as f64).sqrt();
                let z = if se > 0.0 {
                    difference / se
                } else if difference == 0.0 {
                    0.0
                } else {
                    f64::INFINITY
                };
                let ok = accepted(count, p, anchors.z_threshold);
                eprintln!(
                    "{} r/a={} endpoint={} count={count}/{N} p={p:.16} z={z:.6} threshold={:.8} abs_diff={:.8} {}",
                    case.name,
                    point.ratio,
                    j,
                    anchors.z_threshold,
                    difference.abs(),
                    if ok { "PASS" } else { "FAIL" }
                );
                if !ok {
                    failures.push(format!("{} distance={distance} endpoint={j}", case.name));
                    passed = false;
                }
            }
        }
        eprintln!(
            "ANALYTIC {}: {}",
            case.name,
            if passed { "PASS" } else { "FAIL" }
        );
    }
    assert!(failures.is_empty(), "analytic failures: {failures:?}");
}
#[test]
fn rejects_deliberately_slow_sampler() {
    let anchors = anchors();
    let case = &anchors.cases[1];
    let counts = counts(case, 1, 1, 4.0);
    assert!(
        counts
            .iter()
            .zip(case.points[1].probabilities)
            .any(|(&n, p)| !accepted(n, p, anchors.z_threshold)),
        "fourfold time bias escaped analytic gate"
    );
}
