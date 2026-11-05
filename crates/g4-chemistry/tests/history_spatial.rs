//! Deterministic search semantics and resource measurements, not cutoff convergence.

use g4_chemistry::history::{
    CandidateSearch, HistoryInput, HistoryOptions, PairHistory, PairHistoryOptions, run_history,
    source_cutoff,
};
use g4_chemistry::reaction_network::ReactionNetwork;
use g4_chemistry::reactions::ReactionTable;
use g4_chemistry::species::SpeciesTable;
use g4_chemistry::{Species, SpeciesInstance};
use g4_core::rng::{RngStreams, Stage};
use g4_core::testing::ScriptedRng;
use g4_core::units::Time;
use g4_core::vec3::Vec3;
use rand::Rng;

fn network() -> ReactionNetwork {
    ReactionTable::option3()
        .unwrap()
        .compile(&SpeciesTable::option3().unwrap())
        .unwrap()
}

fn instance(species: Species, x: f64) -> SpeciesInstance {
    SpeciesInstance {
        species,
        position: Vec3::new(x, 0.0, 0.0),
        time: Time::from_ps(1.0),
    }
}

fn indexed(multiplier: f64) -> CandidateSearch {
    CandidateSearch::Indexed {
        cutoff_multiplier: multiplier,
    }
}

#[test]
fn cutoff_uses_absolute_final_time_and_scales_the_complete_source_expression() {
    // Geant4 11.4.1 constructor: 1.45 nm + 2 sqrt(8 * 9.46e9 nm²/s * timeMax).
    let at_one = source_cutoff(Time::from_ps(1.0), 1.0).unwrap().nm();
    assert!((at_one - 2.000199963649581).abs() < 1e-12);
    let at_microsecond = source_cutoff(Time::from_ps(1e6), 1.0).unwrap().nm();
    assert!((at_microsecond - 551.649963649581).abs() < 1e-9);
    assert_eq!(
        source_cutoff(Time::from_ps(1e6), 4.0).unwrap().nm(),
        4.0 * at_microsecond
    );
    for multiplier in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::MAX] {
        assert!(source_cutoff(Time::from_ps(1e6), multiplier).is_err());
    }
    for end in [0.0, 1e6 + 1.0, f64::NAN, f64::INFINITY] {
        assert!(source_cutoff(Time::from_ps(end), 1.0).is_err());
    }
}

#[test]
fn initial_search_preserves_whole_source_bins_and_newer_parent_query_direction() {
    let network = network();
    // At 1 ps R=2.0002 nm: [0,10] has four 2.5 nm cells. The newer
    // molecule at 4.4 queries cell 0, but a newer molecule at 0 excludes cell 1.
    // A radial filter, symmetric query or query from the older ID breaks this.
    for (positions, expected) in [([0.0, 4.4, 10.0], 1), ([4.4, 0.0, 10.0], 0)] {
        let input = HistoryInput::new(&positions.map(|x| instance(Species::H, x))).unwrap();
        let mut rng = ScriptedRng::new(&[]);
        let history = PairHistory::with_search(
            &input,
            &[network.get(1).unwrap()],
            PairHistoryOptions {
                end_time: Time::from_ps(1.0),
                ..Default::default()
            },
            indexed(1.0),
            &mut rng,
        )
        .unwrap();
        assert_eq!(history.diagnostics().sampled_pair_channels, expected);
        assert_eq!(history.diagnostics().candidate_pairs, expected);
        assert_eq!(rng.consumed(), expected);
        assert_eq!(history.pending_events(), 0);
    }
}

#[test]
fn coincident_pairs_are_unique_and_stale_entries_do_not_react() {
    let network = network();
    let input = HistoryInput::new(&[instance(Species::H, -8.0); 4]).unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let mut history = PairHistory::with_search(
        &input,
        &[network.get(1).unwrap()],
        PairHistoryOptions::default(),
        indexed(1.0),
        &mut rng,
    )
    .unwrap();
    assert_eq!(history.diagnostics().sampled_pair_channels, 6);
    assert_eq!(history.diagnostics().scheduled_events, 6);
    assert_eq!(history.pending_events(), 6);
    history.close_contacts(&mut rng).unwrap();
    assert_eq!(history.reactions().len(), 2);
    assert_eq!(
        history.reactions()[0]
            .participants
            .iter()
            .map(|id| id.0)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(
        history.reactions()[1]
            .participants
            .iter()
            .map(|id| id.0)
            .collect::<Vec<_>>(),
        [2, 3]
    );
    assert_eq!(history.diagnostics().sampled_pair_channels, 6);
    assert_eq!(history.diagnostics().spatial_entries, 6);
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn compact_histories_match_all_pairs_including_products_unary_and_rng() {
    let network = network();
    let input = HistoryInput::new(&[
        instance(Species::e_aq, 0.0),
        instance(Species::e_aq, 0.01),
        instance(Species::H3Op, 0.02),
        instance(Species::OH, 0.8),
        instance(Species::H, -0.5),
        instance(Species::O3m, 2.0),
    ])
    .unwrap();
    let descriptors: Vec<_> = network.iter().collect();
    for seed in 0..16 {
        let mut oracle_rng = RngStreams::new(2026110605)
            .stream(seed, Stage::Chemistry)
            .unwrap();
        let initial_rng = oracle_rng.clone();
        let mut oracle = PairHistory::with_search(
            &input,
            &descriptors,
            PairHistoryOptions::default(),
            CandidateSearch::AllPairs,
            &mut oracle_rng,
        )
        .unwrap();
        oracle
            .advance_to(Time::from_ps(1e6), &mut oracle_rng)
            .unwrap();
        for multiplier in [1.0, 2.0, 4.0] {
            let mut rng = initial_rng.clone();
            let mut reversed = descriptors.clone();
            reversed.reverse();
            let mut history = PairHistory::with_search(
                &input,
                &reversed,
                PairHistoryOptions::default(),
                indexed(multiplier),
                &mut rng,
            )
            .unwrap();
            for t in [1.0, 10.0, 1e3, 1e6] {
                history.observe_counts(Time::from_ps(t), &mut rng).unwrap();
            }
            assert_eq!(history.reactions(), oracle.reactions());
            assert_eq!(history.molecules(), oracle.molecules());
            assert_eq!(
                history.diagnostics().sampled_pair_channels,
                oracle.diagnostics().sampled_pair_channels
            );
            assert_eq!(rng.next_u64(), oracle_rng.clone().next_u64());
        }
    }
}

#[test]
fn invalid_search_fails_before_rng_and_empty_runner_reports_configuration() {
    let network = network();
    let input = HistoryInput::new(&[]).unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let error = run_history(
        &input,
        &network,
        &[],
        HistoryOptions {
            search: indexed(f64::NAN),
            ..Default::default()
        },
        &mut rng,
    );
    assert!(error.is_err());
    assert_eq!(rng.consumed(), 0);
    for search in [
        CandidateSearch::AllPairs,
        indexed(1.0),
        indexed(2.0),
        indexed(4.0),
    ] {
        let result = run_history(
            &input,
            &network,
            &[],
            HistoryOptions {
                search,
                ..Default::default()
            },
            &mut rng,
        )
        .unwrap();
        assert_eq!(result.diagnostics.search, search);
        assert_eq!(result.diagnostics.end_time.ps(), 1e6);
        assert_eq!(
            result.diagnostics.cutoff,
            match search {
                CandidateSearch::AllPairs => None,
                CandidateSearch::Indexed { cutoff_multiplier } =>
                    Some(source_cutoff(Time::from_ps(1e6), cutoff_multiplier).unwrap()),
            }
        );
        assert_eq!(result.diagnostics.candidate_pairs, 0);
    }
    assert_eq!(HistoryOptions::default().search, indexed(1.0));
}

#[test]
fn unrepresentable_source_grid_fails_explicitly_before_sampling() {
    let network = network();
    for (records, multiplier) in [
        (
            [
                instance(Species::H, -f64::MAX),
                instance(Species::H, f64::MAX),
            ],
            1.0,
        ),
        ([instance(Species::H, 0.0), instance(Species::H, 1e15)], 1.0),
        (
            [instance(Species::H, 0.0), instance(Species::H, 1.0)],
            f64::MIN_POSITIVE,
        ),
    ] {
        let input = HistoryInput::new(&records).unwrap();
        let mut rng = ScriptedRng::new(&[]);
        let error = run_history(
            &input,
            &network,
            &[],
            HistoryOptions {
                search: indexed(multiplier),
                ..Default::default()
            },
            &mut rng,
        )
        .unwrap_err();
        let g4_chemistry::ChemistryError::IncompleteHistory { diagnostics, cause } = error else {
            panic!("expected incomplete history");
        };
        assert!(matches!(
            *cause,
            g4_chemistry::ChemistryError::Invalid {
                file: "history spatial search",
                ..
            }
        ));
        assert_eq!(diagnostics.executed_reactions, 0);
        assert_eq!(diagnostics.sampled_pair_channels, 0);
        assert_eq!(rng.consumed(), 0);
    }
}

#[test]
fn finite_cutoff_omits_candidates_without_reclassifying_oracle_escapes() {
    let network = network();
    let input =
        HistoryInput::new(&[instance(Species::H, 0.0), instance(Species::H, 20.0)]).unwrap();
    for (search, sampled) in [(indexed(1.0), 0), (CandidateSearch::AllPairs, 1)] {
        let mut rng = ScriptedRng::new(&[]);
        let history = PairHistory::with_search(
            &input,
            &[network.get(1).unwrap()],
            PairHistoryOptions {
                end_time: Time::from_ps(1.0),
                ..Default::default()
            },
            search,
            &mut rng,
        )
        .unwrap();
        assert_eq!(history.diagnostics().sampled_pair_channels, sampled);
        assert_eq!(rng.consumed(), sampled);
        assert_eq!(history.pending_events(), 0);
    }
}

#[test]
fn indexed_capacity_failure_reports_attempted_candidates_without_successful_counts() {
    let network = network();
    let input = HistoryInput::new(&[instance(Species::H, 0.0); 2]).unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let error = run_history(
        &input,
        &network,
        &[],
        HistoryOptions {
            limits: PairHistoryOptions {
                max_pending_events: 0,
                ..Default::default()
            },
            ..Default::default()
        },
        &mut rng,
    )
    .unwrap_err();
    let g4_chemistry::ChemistryError::IncompleteHistory { diagnostics, cause } = error else {
        panic!("expected incomplete history");
    };
    assert!(matches!(
        *cause,
        g4_chemistry::ChemistryError::HistoryCapacity {
            resource: "pending events",
            limit: 0,
            required: 1
        }
    ));
    assert_eq!(diagnostics.search, indexed(1.0));
    assert_eq!(diagnostics.candidate_pairs, 1);
    assert_eq!(diagnostics.sampled_pair_channels, 1);
    assert_eq!(diagnostics.scheduled_events, 0);
    assert_eq!(diagnostics.executed_reactions, 0);
    assert_eq!(diagnostics.spatial_entries, 2);
    assert!(diagnostics.peak_storage_bytes > 0);
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn sparse_high_population_reports_bounded_storage_events_and_candidates() {
    let network = network();
    // 10,000 mobile H atoms in 5,000 isolated contacting pairs. At 1 ps only
    // their within-pair contacts are candidates. Products remain indexed.
    let records: Vec<_> = (0..10_000)
        .map(|i| {
            let pair = i / 2;
            let mut record = instance(
                Species::H,
                f64::from(pair % 100) * 20.0 + f64::from(i % 2) * 0.01,
            );
            record.position.y = f64::from(pair / 100) * 20.0;
            record
        })
        .collect();
    let input = HistoryInput::new(&records).unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let result = run_history(
        &input,
        &network,
        &[],
        HistoryOptions {
            limits: PairHistoryOptions {
                end_time: Time::from_ps(1.0),
                ..Default::default()
            },
            ..Default::default()
        },
        &mut rng,
    )
    .unwrap();
    let d = result.diagnostics;
    println!("10,000 H / 5,000 contact pairs: {d:?}");
    assert_eq!(d.executed_reactions, 5_000);
    assert_eq!(d.sampled_pair_channels, 5_000);
    assert_eq!(d.scheduled_events, 5_000);
    assert_eq!(d.peak_pending_events, 5_000);
    assert_eq!(d.arena_records, 15_000);
    assert_eq!(d.spatial_entries, 15_000);
    assert_eq!(result.final_counts[Species::H2.index()], 5_000);
    assert!(d.candidate_pairs < 20_000);
    assert!(d.peak_candidate_buffer <= 2);
    assert!(d.peak_storage_bytes > 1_000_000 && d.peak_storage_bytes < 10_000_000);
    // The full network also samples H unary channels 55 and 68 at birth.
    assert_eq!(rng.consumed(), 20_000);
}
