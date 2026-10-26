//! Option3 unary clocks, Geant4 11.4.1 `G4DNAIRT::Sampling` lines 250-281.
//!
//! Preregistered competing-clock rule: N = 100,000 per mobile-species family,
//! family alpha = 0.001. At t = ln(2)/sum(k), test the minimum-time CDF
//! against 1/2 and each winner fraction against k_i/sum(k). Hoeffding's
//! two-sided bound with Bonferroni across the (channels + 1) checks controls
//! each family's false rejection probability at <= alpha without fitting.

use g4_chemistry::ChemistryError;
use g4_chemistry::irt::ReactionTime;
use g4_chemistry::reaction_network::UnaryReactionParameters;
use g4_chemistry::reactions::{BulkParticipant, DEFAULT_REACTIONS_TOML, ReactionTable};
use g4_chemistry::species::SpeciesTable;
use g4_chemistry::unary::sample_unary_reaction_time;
use g4_core::rng::{RngStreams, Stage};
use g4_core::testing::ScriptedRng;

fn network() -> g4_chemistry::reaction_network::ReactionNetwork {
    ReactionTable::option3()
        .unwrap()
        .compile(&SpeciesTable::option3().unwrap())
        .unwrap()
}

fn elapsed(result: ReactionTime) -> f64 {
    let ReactionTime::After(time) = result else {
        panic!("enabled unary channel must yield an elapsed delay");
    };
    time.ps()
}

fn custom_rate(literal: &str) -> UnaryReactionParameters {
    let source =
        DEFAULT_REACTIONS_TOML.replace("rate_per_s = 2.66e3", &format!("rate_per_s = {literal}"));
    ReactionTable::from_toml(&source)
        .unwrap()
        .compile(&SpeciesTable::option3().unwrap())
        .unwrap()
        .get(53)
        .unwrap()
        .unary()
        .copied()
        .unwrap()
}

#[test]
fn all_20_source_channels_have_independent_seconds_based_clock_anchors() {
    let network = network();
    let channels: Vec<_> = network.iter().filter_map(|r| r.unary().copied()).collect();
    assert_eq!(channels.len(), 20);
    for channel in channels {
        assert!((53..=72).contains(&channel.reaction_id()));
        assert!(!channel.is_disabled());
        let mut rng = ScriptedRng::new(&[0.5, 0.75]);
        let observed = elapsed(sample_unary_reaction_time(&channel, &mut rng).unwrap());
        let expected = std::f64::consts::LN_2 * 1.0e12 / channel.rate_per_s();
        assert!(
            (observed / expected - 1.0).abs() < 1e-12,
            "ID {}",
            channel.reaction_id()
        );
        assert!(observed.is_finite() && observed > 0.0);
        assert_eq!(rng.consumed(), 1, "ID {}", channel.reaction_id());
    }
    assert_eq!(
        network.get(53).unwrap().unary().unwrap().bulk(),
        BulkParticipant::NoneM
    );
}

#[test]
fn disabled_clock_preserves_descriptor_and_does_not_draw() {
    let channel = custom_rate("0.0");
    assert!(channel.is_disabled());
    let mut rng = ScriptedRng::new(&[]);
    assert_eq!(
        sample_unary_reaction_time(&channel, &mut rng).unwrap(),
        ReactionTime::NoReaction
    );
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn invalid_rates_and_unrepresentable_delays_are_typed_errors() {
    for literal in ["-1.0", "nan", "inf", "-inf", "1e309"] {
        let source = DEFAULT_REACTIONS_TOML
            .replace("rate_per_s = 2.66e3", &format!("rate_per_s = {literal}"));
        assert!(ReactionTable::from_toml(&source).is_err(), "{literal}");
    }
    let tiny = custom_rate("1e-300");
    let mut rng = ScriptedRng::new(&[0.5]);
    assert_eq!(
        sample_unary_reaction_time(&tiny, &mut rng),
        Err(ChemistryError::UnaryClockDomain { reaction_id: 53 })
    );
    assert_eq!(rng.consumed(), 1);

    let huge = custom_rate("1.7976931348623157e308");
    let mut rng = ScriptedRng::new(&[0.5]);
    let delay = elapsed(sample_unary_reaction_time(&huge, &mut rng).unwrap());
    assert!(delay.is_finite() && delay > 0.0);
}

#[test]
fn competing_channels_match_total_hazard_and_branching_per_family() {
    const N: usize = 100_000;
    const FAMILY_ALPHA: f64 = 0.001;
    let mut families: Vec<Vec<UnaryReactionParameters>> = Vec::new();
    for channel in network().iter().filter_map(|r| r.unary().copied()) {
        if let Some(family) = families
            .iter_mut()
            .find(|f| f[0].mobile() == channel.mobile())
        {
            family.push(channel);
        } else {
            families.push(vec![channel]);
        }
    }
    let competing: Vec<_> = families.into_iter().filter(|f| f.len() >= 2).collect();
    assert!(!competing.is_empty());

    for (family_index, channels) in competing.iter().enumerate() {
        let total_rate: f64 = channels.iter().map(|c| c.rate_per_s()).sum();
        let half_life_ps = std::f64::consts::LN_2 * 1.0e12 / total_rate;
        let mut rng = RngStreams::new(0xD1A0_2026)
            .stream(family_index as u64, Stage::Chemistry)
            .unwrap();
        let mut before_half_life = 0;
        let mut winner_counts = vec![0usize; channels.len()];
        for _ in 0..N {
            let (winner, delay) = channels
                .iter()
                .enumerate()
                .map(|(i, c)| (i, elapsed(sample_unary_reaction_time(c, &mut rng).unwrap())))
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap();
            before_half_life += usize::from(delay <= half_life_ps);
            winner_counts[winner] += 1;
        }
        let epsilon =
            ((2.0 * (channels.len() + 1) as f64 / FAMILY_ALPHA).ln() / (2.0 * N as f64)).sqrt();
        let cdf = before_half_life as f64 / N as f64;
        assert!(
            (cdf - 0.5).abs() < epsilon,
            "family {:?}: CDF {cdf}",
            channels[0].mobile()
        );
        for (channel, count) in channels.iter().zip(winner_counts) {
            let observed = count as f64 / N as f64;
            let expected = channel.rate_per_s() / total_rate;
            assert!(
                (observed - expected).abs() < epsilon,
                "family {:?}, ID {}: observed {observed}, expected {expected}",
                channel.mobile(),
                channel.reaction_id()
            );
        }
    }
}
