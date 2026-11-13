//! L1 regression tests; these do not close the preregistered SYN-1/2/3 gates.
mod common;
mod syn_support;
use common::{isolated_pair, option3_parameters};
use g4_chemistry::irt::ReactionTime;
use g4_chemistry::molecule::{Molecule, MoleculePair};
use g4_chemistry::reaction_network::ReactionNetwork;
use g4_chemistry::reactions::ReactionTable;
use g4_chemistry::species::SpeciesTable;
use g4_chemistry::syn::{PairContext, place_pair, place_pair_diagnostic, sample_pair};
use g4_core::rng::{RngStreams, Stage};
use g4_core::units::Time;
use g4_core::vec3::Vec3;

fn network() -> ReactionNetwork {
    ReactionNetwork::compile(
        &ReactionTable::option3().unwrap(),
        &SpeciesTable::option3().unwrap(),
    )
    .unwrap()
}

#[test]
fn explicit_immobile_parent_diagnostic_preserves_the_fixed_site() {
    use g4_chemistry::syn::conditional_parent_positions;
    use g4_core::units::{Diffusivity, Length};
    let mut rng = RngStreams::new(2026110807)
        .stream(0, Stage::Chemistry)
        .unwrap();
    for dt in [0.0, 1.0, 1000.0] {
        for coincident in [false, true] {
            for fixed in 0..2 {
                let points = [
                    Vec3::new(1.0, 2.0, 3.0),
                    if coincident {
                        Vec3::new(1.0, 2.0, 3.0)
                    } else {
                        Vec3::new(-4.0, 5.0, 6.0)
                    },
                ];
                let d = std::array::from_fn(|i| {
                    Diffusivity::from_nm2_per_ps(if i == fixed { 0.0 } else { 0.001 })
                });
                let actual = conditional_parent_positions(
                    2,
                    points,
                    d,
                    Length::from_nm(0.5),
                    Time::from_ps(dt),
                    &mut rng,
                )
                .unwrap();
                assert_eq!(actual, [points[fixed]; 2]);
            }
        }
    }
    assert!(
        conditional_parent_positions(
            2,
            [Vec3::ZERO; 2],
            [Diffusivity::from_nm2_per_ps(0.0); 2],
            Length::from_nm(0.5),
            Time::from_ps(0.0),
            &mut rng
        )
        .is_err()
    );
}

#[test]
fn provisional_moment_oracle_and_student_quantiles_are_numerically_checked() {
    assert!(
        (syn_support::t_critical(58.0, 0.001 / (3.0 * 5076.0)) - 6.191773873380006).abs() < 1e-9
    );
    assert!((syn_support::t_critical(29.0, 0.05) - 2.045229642132703).abs() < 1e-10);
    let network = network();
    for id in syn_support::PLACEMENT_IDS {
        for ratio in syn_support::RATIOS {
            for dt in syn_support::DURATIONS {
                for orientation in 0..3 {
                    let a = syn_support::expected(
                        network.get(id).unwrap(),
                        ratio,
                        dt,
                        orientation,
                        4096,
                    );
                    let b = syn_support::expected(
                        network.get(id).unwrap(),
                        ratio,
                        dt,
                        orientation,
                        8192,
                    );
                    let (a, _) = syn_support::oracle_endpoints(&a);
                    let (b, _) = syn_support::oracle_endpoints(&b);
                    for (a, b) in a.iter().zip(b) {
                        assert!((a - b).abs() < 1e-11);
                    }
                }
            }
        }
    }
}

#[test]
fn contact_overlap_and_no_event_are_distinct() {
    let mut rng = RngStreams::new(2026110801)
        .stream(0, Stage::Chemistry)
        .unwrap();
    for id in [1, 2, 8, 9, 11, 17, 42, 51] {
        let p = option3_parameters(id);
        for ratio in [0.0, 0.5, 1.0, 3.0] {
            let m = isolated_pair(p, ratio * p.effective_radius().nm(), 0);
            let pair = MoleculePair::new(&m[0], &m[1]).unwrap();
            if ratio < 1.0 {
                assert!(sample_pair(pair, &p, PairContext::PostStart, &mut rng).is_err());
                let outcome = sample_pair(pair, &p, PairContext::Initial, &mut rng);
                assert!(matches!(outcome, Ok(ReactionTime::Immediate) | Err(_)));
            } else {
                for _ in 0..100 {
                    let outcome = sample_pair(pair, &p, PairContext::PostStart, &mut rng).unwrap();
                    if ratio == 1.0 && p.pdc().is_none() {
                        assert_eq!(outcome, ReactionTime::Immediate);
                    }
                    if ratio == 1.0 && p.pdc().is_some() {
                        assert!(!matches!(outcome, ReactionTime::Immediate));
                    }
                }
            }
        }
    }
}

#[test]
fn invalid_positions_times_and_species_fail_explicitly() {
    let p = option3_parameters(2);
    let m = isolated_pair(p, p.effective_radius().nm(), 0);
    let mut rng = RngStreams::new(2).stream(0, Stage::Chemistry).unwrap();
    for (time, x) in [(2.0, 0.0), (f64::NAN, 0.0), (1.0, f64::INFINITY)] {
        let mut instance = *m[0].instance();
        instance.time = Time::from_ps(time);
        instance.position.x = x;
        let a = Molecule::new(m[0].id(), instance);
        assert!(
            sample_pair(
                MoleculePair::new(&a, &m[1]).unwrap(),
                &p,
                PairContext::Initial,
                &mut rng
            )
            .is_err()
        );
    }
    assert!(
        sample_pair(
            MoleculePair::new(&m[0], &m[1]).unwrap(),
            &option3_parameters(1),
            PairContext::Initial,
            &mut rng
        )
        .is_err()
    );
}

#[test]
fn ordinary_placement_keeps_parents_and_colocates_source_ordered_products() {
    let network = network();
    let mut rng = RngStreams::new(3).stream(0, Stage::Chemistry).unwrap();
    for id in [1, 2, 8, 9, 11, 17, 42, 51] {
        let reaction = network.get(id).unwrap();
        let p = *reaction.pair().unwrap();
        let m = isolated_pair(p, 3.0 * p.effective_radius().nm(), 0);
        let out = place_pair(MoleculePair::new(&m[0], &m[1]).unwrap(), reaction, &mut rng).unwrap();
        assert_eq!(out.parent_positions, m.map(|m| m.instance().position));
        assert_eq!(out.products.len(), reaction.source().products.len());
        for (product, source) in out.products.iter().zip(&reaction.source().products) {
            assert_eq!(
                g4_chemistry::reactions::ReactionParticipant::Mobile(product.species),
                *source
            );
            assert_eq!(product.position, out.products[0].position);
            assert!(product.position.x >= 0.0 && product.position.x <= m[1].instance().position.x);
            assert_eq!(product.position.y, 0.0);
        }
        let coincident = isolated_pair(p, 0.0, 0);
        assert!(
            place_pair(
                MoleculePair::new(&coincident[0], &coincident[1]).unwrap(),
                reaction,
                &mut rng
            )
            .is_err()
        );
    }
}

#[test]
fn direct_placement_preserves_source_contact_and_axial_anomaly() {
    let network = network();
    let reaction = network.get(2).unwrap();
    let p = *reaction.pair().unwrap();
    let mut rng = RngStreams::new(4).stream(0, Stage::Chemistry).unwrap();
    for dt in [1.0, 1000.0, 1e6] {
        let mut m = isolated_pair(p, 3.0 * p.effective_radius().nm(), 0);
        let mut b = *m[1].instance();
        b.position = Vec3::new(0.0, 0.0, b.position.x);
        m[1] = Molecule::new(m[1].id(), b);
        let out = place_pair_diagnostic(
            MoleculePair::new(&m[0], &m[1]).unwrap(),
            reaction,
            Time::from_ps(dt),
            &mut rng,
        )
        .unwrap();
        let delta = out.parent_positions[0] - out.parent_positions[1];
        assert!((delta.norm() - p.effective_radius().nm()).abs() < 1e-12);
        assert_eq!(delta.y, 0.0); // setPhi cannot change a z-axis vector.
        assert!(delta.x >= 0.0); // explicit source diagnostic, not physical invariance.
    }
}

// Independent quadrature oracle: integrate the first-passage density convolved
// with P(E/|N(0,2)| <= x)=1-exp(x*x)*erfc(x). No inverse-erfc sampling is used.
fn scaled_erfc(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.3275911 * x);
    t * (0.254829592
        + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))))
}
fn cdf(r: f64, t: f64, activation: Option<f64>) -> f64 {
    let g = r - 1.0;
    let rate = activation.unwrap_or(1.0);
    if activation.is_none() {
        let x = g / (2.0 * t.sqrt());
        return (-x * x).exp() * scaled_erfc(x) / r;
    }
    let delay = |s: f64| {
        if s <= 0.0 {
            0.0
        } else {
            1.0 - scaled_erfc(s.sqrt() / (1.0 - rate))
        }
    };
    if g == 0.0 {
        return rate * delay(t);
    }
    let n = 20000;
    let h = t / n as f64;
    let f = |s: f64| {
        if s == 0.0 || s == t {
            0.0
        } else {
            g * (-g * g / (4.0 * s)).exp() / (2.0 * std::f64::consts::PI.sqrt() * s.powf(1.5))
                * delay(t - s)
        }
    };
    let sum: f64 = (1..n)
        .map(|i| f(i as f64 * h) * if i % 2 == 0 { 2.0 } else { 4.0 })
        .sum();
    rate / r * sum * h / 3.0
}

#[test]
fn type_zero_and_type_one_laws_match_independent_quadrature() {
    const N: usize = 20000;
    for id in [1, 2, 8, 9, 11, 17, 42, 51] {
        let p = option3_parameters(id);
        let rate = p.pdc().map(|v| {
            v.activation_rate().nm3_per_ps()
                / (v.activation_rate().nm3_per_ps() + v.diffusion_rate().nm3_per_ps())
        });
        for ratio in [1.0, 1.5, 3.0] {
            let m = isolated_pair(p, ratio * p.effective_radius().nm(), 0);
            let pair = MoleculePair::new(&m[0], &m[1]).unwrap();
            let mut rng = RngStreams::new(2026110802 + id as u64)
                .stream((ratio * 10.0) as u64, Stage::Chemistry)
                .unwrap();
            let scale = p.effective_radius().nm().powi(2) / p.relative_diffusivity().nm2_per_ps();
            let mut counts = [0usize; 6];
            for _ in 0..N {
                match sample_pair(pair, &p, PairContext::Initial, &mut rng).unwrap() {
                    ReactionTime::NoReaction => counts[5] += 1,
                    result => {
                        let t = match result {
                            ReactionTime::After(t) => t.ps(),
                            _ => 0.0,
                        };
                        for (i, tau) in [0.01, 0.1, 1.0, 10.0, 100.0].iter().enumerate() {
                            if t <= tau * scale {
                                counts[i] += 1;
                            }
                        }
                    }
                }
            }
            for (i, tau) in [0.01, 0.1, 1.0, 10.0, 100.0].iter().enumerate() {
                let expected = cdf(ratio, *tau, rate);
                assert!(
                    (counts[i] as f64 / N as f64 - expected).abs() < 0.03,
                    "channel {id} r/a {ratio} tau {tau}: expected {expected}, count {}",
                    counts[i]
                );
            }
            assert!(
                (counts[5] as f64 / N as f64 - (1.0 - rate.unwrap_or(1.0) / ratio)).abs() < 0.03
            );
        }
    }
}
