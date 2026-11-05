//! Count-only observations are inclusive and do not materialize survivor sites.
//! Exact deterministic contracts for R05; no claim of G2 or physical validation.

use g4_chemistry::history::{
    HistoryInput, HistoryOptions, PairHistory, PairHistoryOptions, SpeciesCounts, run_history,
};
use g4_chemistry::reaction_network::ReactionNetwork;
use g4_chemistry::reactions::ReactionTable;
use g4_chemistry::species::SpeciesTable;
use g4_chemistry::{ChemistryError, Species, SpeciesInstance};
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

fn counts(populations: &[(Species, usize)]) -> SpeciesCounts {
    let mut counts = [0; Species::ALL.len()];
    for &(species, count) in populations {
        counts[species.index()] = count;
    }
    counts
}

fn options(end: f64) -> HistoryOptions {
    HistoryOptions {
        limits: PairHistoryOptions {
            end_time: Time::from_ps(end),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn pre_irt_and_post_contact_counts_are_distinct_immutable_boundaries() {
    let network = network();
    let records = [instance(Species::H, 0.0), instance(Species::H, 0.01)];
    let input = HistoryInput::new(&records).unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let result = run_history(
        &input,
        &network,
        &[Time::from_ps(1.0); 2],
        options(1.0),
        &mut rng,
    )
    .unwrap();
    assert_eq!(input.snapshot(), records);
    assert_eq!(result.pre_irt_counts, counts(&[(Species::H, 2)]));
    assert_eq!(result.post_contact_counts, counts(&[(Species::H2, 1)]));
    assert_eq!(result.final_counts, result.post_contact_counts);
    assert_eq!(result.observations.len(), 2);
    for observation in &result.observations {
        assert_eq!(observation.time.ps(), 1.0);
        assert_eq!(observation.counts, result.post_contact_counts);
    }
    assert_eq!(result.diagnostics.executed_reactions, 1);
    assert!(result.trace.is_none());
    assert_eq!(result.diagnostics.trace_records, 0);
}

#[test]
fn observation_includes_every_tied_event_and_the_final_boundary() {
    let network = network();
    let reaction = network.get(1).unwrap();
    let radius = reaction.pair().unwrap().effective_radius().nm();
    // Two identical pairs in parallel planes: their clocks are exactly equal.
    // Four cross-pair draws escape. Only the two chosen clocks execute.
    let mut records = [
        instance(Species::H, 0.0),
        instance(Species::H, 2.0 * radius),
        instance(Species::H, 0.0),
        instance(Species::H, 2.0 * radius),
    ];
    records[2].position.y = 1e6;
    records[3].position.y = 1e6;
    let input = HistoryInput::new(&records).unwrap();
    let mut rng = ScriptedRng::new(&[0.1, 0.9, 0.9, 0.9, 0.9, 0.1]);
    let mut history =
        PairHistory::new(&input, &[reaction], PairHistoryOptions::default(), &mut rng).unwrap();
    let boundary = history.next_event_time().unwrap();
    let before = Time::from_ps(f64::from_bits(boundary.ps().to_bits() - 1));
    assert_eq!(
        history.observe_counts(before, &mut rng).unwrap().counts,
        counts(&[(Species::H, 4)])
    );
    assert_eq!(
        history.observe_counts(boundary, &mut rng).unwrap().counts,
        counts(&[(Species::H2, 2)])
    );
    assert_eq!(history.reactions().len(), 2);
    assert!(history.reactions().iter().all(|r| r.time == boundary));

    // The public runner also keeps an event exactly at the configured horizon.
    let input = HistoryInput::new(&records[..2]).unwrap();
    let mut rng = ScriptedRng::new(&[0.1]);
    let result = run_history(
        &input,
        &network,
        &[before, boundary],
        options(boundary.ps()),
        &mut rng,
    )
    .unwrap();
    assert_eq!(result.observations[0].counts, counts(&[(Species::H, 2)]));
    assert_eq!(result.observations[1].counts, counts(&[(Species::H2, 1)]));
    assert_eq!(result.final_counts, counts(&[(Species::H2, 1)]));
    assert_eq!(result.diagnostics.time, boundary);
}

#[test]
fn contact_observation_waits_for_product_cascade_closure() {
    let network = network();
    let input = HistoryInput::new(&[
        instance(Species::e_aq, 0.0),
        instance(Species::e_aq, 0.01),
        instance(Species::H3Op, 0.02),
    ])
    .unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let mut history = PairHistory::new(
        &input,
        &[network.get(8).unwrap(), network.get(9).unwrap()],
        PairHistoryOptions::default(),
        &mut rng,
    )
    .unwrap();
    let observation = history
        .observe_counts(Time::from_ps(1.0), &mut rng)
        .unwrap();
    assert_eq!(
        observation.counts,
        counts(&[(Species::H2, 1), (Species::OHm, 1)])
    );
    assert_eq!(history.reactions().len(), 2);
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn intermediate_observations_preserve_arena_lineage_and_rng_bits() {
    let network = network();
    let radius = network
        .get(1)
        .unwrap()
        .pair()
        .unwrap()
        .effective_radius()
        .nm();
    let input = HistoryInput::new(&[
        instance(Species::H, 0.0),
        instance(Species::H, 2.0 * radius),
        instance(Species::OH, 2.0),
        instance(Species::OH, 2.02),
        instance(Species::O3m, 5.0),
        instance(Species::O2, 100.0),
    ])
    .unwrap();
    let descriptors: Vec<_> = network.iter().collect();
    let mut delayed_reactions = 0;
    for seed in 0..16 {
        let mut direct_rng = RngStreams::new(2026110504)
            .stream(seed, Stage::Chemistry)
            .unwrap();
        let mut observed_rng = direct_rng.clone();
        let mut direct = PairHistory::new(
            &input,
            &descriptors,
            PairHistoryOptions::default(),
            &mut direct_rng,
        )
        .unwrap();
        let mut observed = PairHistory::new(
            &input,
            &descriptors,
            PairHistoryOptions::default(),
            &mut observed_rng,
        )
        .unwrap();
        for time in [1.0, 1.0, 1.1, 2.0, 10.0, 100.0, 1e3, 1e4, 1e5, 1e6] {
            observed
                .observe_counts(Time::from_ps(time), &mut observed_rng)
                .unwrap();
        }
        direct
            .advance_to(Time::from_ps(1e6), &mut direct_rng)
            .unwrap();
        assert_eq!(observed.molecules(), direct.molecules(), "seed {seed}");
        // Float comparisons above also verify scientific equality; bitwise fields
        // catch a sign-of-zero change that PartialEq would accept.
        for (left, right) in observed.molecules().iter().zip(direct.molecules()) {
            let a = left.instance();
            let b = right.instance();
            for (a, b) in [
                (a.position.x, b.position.x),
                (a.position.y, b.position.y),
                (a.position.z, b.position.z),
                (a.time.ps(), b.time.ps()),
            ] {
                assert_eq!(a.to_bits(), b.to_bits());
            }
        }
        assert_eq!(observed.reactions(), direct.reactions());
        delayed_reactions += direct
            .reactions()
            .iter()
            .filter(|r| r.time.ps() > 1.0)
            .count();
        assert_eq!(observed.pending_events(), direct.pending_events());
        for _ in 0..128 {
            assert_eq!(observed_rng.next_u64(), direct_rng.next_u64());
        }
    }
    assert!(
        delayed_reactions > 0,
        "exercise positive-time placement across the grid"
    );
}

#[test]
fn trace_toggle_and_observation_grid_leave_completed_run_and_rng_unchanged() {
    let network = network();
    let input = HistoryInput::new(&[
        instance(Species::H, 0.0),
        instance(Species::H, 0.01),
        instance(Species::O3m, 5.0),
    ])
    .unwrap();
    let mut no_trace_rng = RngStreams::new(2026110504)
        .stream(31, Stage::Chemistry)
        .unwrap();
    let mut trace_rng = no_trace_rng.clone();
    let no_trace = run_history(&input, &network, &[], options(1e6), &mut no_trace_rng).unwrap();
    let trace = run_history(
        &input,
        &network,
        &[1.0, 10.0, 1e3, 1e6].map(Time::from_ps),
        HistoryOptions {
            max_trace_records: Some(100),
            ..options(1e6)
        },
        &mut trace_rng,
    )
    .unwrap();
    assert!(no_trace.trace.is_none());
    let records = trace.trace.as_ref().unwrap();
    assert!(!records.is_empty());
    assert_eq!(records.len(), trace.diagnostics.executed_reactions);
    assert_eq!(trace.diagnostics.trace_records, records.len());
    assert_eq!(no_trace.diagnostics.trace_records, 0);
    assert_eq!(no_trace.final_counts, trace.final_counts);
    assert_eq!(no_trace.pre_irt_counts, trace.pre_irt_counts);
    assert_eq!(no_trace.post_contact_counts, trace.post_contact_counts);
    assert_eq!(
        no_trace.diagnostics.executed_reactions,
        trace.diagnostics.executed_reactions
    );
    assert_eq!(
        no_trace.diagnostics.arena_records,
        trace.diagnostics.arena_records
    );
    assert_eq!(
        no_trace.diagnostics.peak_pending_events,
        trace.diagnostics.peak_pending_events
    );
    for _ in 0..128 {
        assert_eq!(no_trace_rng.next_u64(), trace_rng.next_u64());
    }
}

#[test]
fn empty_and_nonreacting_histories_preserve_counts_and_survivor_materialization() {
    let network = network();
    for records in [vec![], vec![instance(Species::H2, 9.0)]] {
        let input = HistoryInput::new(&records).unwrap();
        let mut rng = ScriptedRng::new(&[]);
        let expected = counts(&[(Species::H2, records.len())]);
        let result = run_history(
            &input,
            &network,
            &[1.0, 100.0, 1e6].map(Time::from_ps),
            HistoryOptions {
                max_trace_records: Some(0),
                ..options(1e6)
            },
            &mut rng,
        )
        .unwrap();
        assert_eq!(result.pre_irt_counts, expected);
        assert_eq!(result.post_contact_counts, expected);
        assert_eq!(result.final_counts, expected);
        assert!(result.observations.iter().all(|o| o.counts == expected));
        assert!(result.trace.unwrap().is_empty());
        assert_eq!(rng.consumed(), 0);
        let mut history =
            PairHistory::new(&input, &[], PairHistoryOptions::default(), &mut rng).unwrap();
        history
            .observe_counts(Time::from_ps(1e6), &mut rng)
            .unwrap();
        assert_eq!(
            history
                .molecules()
                .iter()
                .map(|m| *m.instance())
                .collect::<Vec<_>>(),
            records
        );
        assert_eq!(rng.consumed(), 0);
    }
}

#[test]
fn invalid_observation_schedule_is_rejected_before_any_rng_draw() {
    let network = network();
    let input = HistoryInput::new(&[instance(Species::H, 0.0)]).unwrap();
    for times in [
        vec![f64::NAN],
        vec![f64::INFINITY],
        vec![0.0],
        vec![-1.0],
        vec![11.0],
        vec![2.0, 1.0],
    ] {
        let mut rng = ScriptedRng::new(&[]);
        assert!(matches!(
            run_history(
                &input,
                &network,
                &times.into_iter().map(Time::from_ps).collect::<Vec<_>>(),
                options(10.0),
                &mut rng
            ),
            Err(ChemistryError::InvalidHistoryTime {
                field: "observation time",
                ..
            })
        ));
        assert_eq!(rng.consumed(), 0);
    }
}

#[test]
fn resource_failures_return_only_typed_incomplete_errors_and_diagnostics() {
    let network = network();
    let input =
        HistoryInput::new(&[instance(Species::H, 0.0), instance(Species::H, 0.01)]).unwrap();
    for (resource, limits, trace_limit, observations) in [
        (
            "molecules",
            PairHistoryOptions {
                max_molecules: 1,
                ..Default::default()
            },
            None,
            2,
        ),
        (
            "pending events",
            PairHistoryOptions {
                max_pending_events: 0,
                ..Default::default()
            },
            None,
            2,
        ),
        (
            "reactions",
            PairHistoryOptions {
                max_reactions: 0,
                ..Default::default()
            },
            None,
            2,
        ),
        ("trace records", PairHistoryOptions::default(), Some(0), 2),
        ("observations", PairHistoryOptions::default(), None, 0),
    ] {
        let mut rng = ScriptedRng::new(&[]);
        let error = run_history(
            &input,
            &network,
            &[Time::from_ps(1.0)],
            HistoryOptions {
                limits,
                max_trace_records: trace_limit,
                max_observations: observations,
            },
            &mut rng,
        )
        .unwrap_err();
        let ChemistryError::IncompleteHistory { cause, diagnostics } = error else {
            panic!("{resource}: expected incomplete history, got {error:?}");
        };
        assert!(
            matches!(*cause, ChemistryError::HistoryCapacity { resource: actual, .. } if actual == resource)
        );
        assert_eq!(diagnostics.executed_reactions, 0);
        assert_eq!(diagnostics.trace_records, 0);
        assert_eq!(diagnostics.time.ps(), 1.0);
    }
}

#[test]
fn reaction_and_trace_budgets_remain_independent_after_partial_execution() {
    let network = network();
    // The second H+H contact exhausts either budget after one recorded reaction.
    let input = HistoryInput::new(&[
        instance(Species::H, 0.0),
        instance(Species::H, 0.01),
        instance(Species::H, 100.0),
        instance(Species::H, 100.01),
    ])
    .unwrap();
    for (max_reactions, max_trace_records, expected_resource, expected_records) in
        [(1, None, "reactions", 0), (10, Some(1), "trace records", 1)]
    {
        let mut rng = ScriptedRng::new(&[]);
        let error = run_history(
            &input,
            &network,
            &[],
            HistoryOptions {
                limits: PairHistoryOptions {
                    max_reactions,
                    ..Default::default()
                },
                max_trace_records,
                ..Default::default()
            },
            &mut rng,
        )
        .unwrap_err();
        let ChemistryError::IncompleteHistory { cause, diagnostics } = error else {
            panic!("{error:?}");
        };
        assert!(
            matches!(*cause, ChemistryError::HistoryCapacity { resource, limit: 1, required: 2 } if resource == expected_resource)
        );
        assert_eq!(diagnostics.executed_reactions, 1);
        assert_eq!(diagnostics.trace_records, expected_records);
        assert_eq!(diagnostics.arena_records, 5);
        assert_eq!(diagnostics.pending_events, 1);
        assert_eq!(diagnostics.peak_pending_events, 2);
    }
}

#[test]
fn numerical_failure_has_diagnostics_and_failed_engine_cannot_observe_counts() {
    let network = network();
    let mut molecule = instance(Species::H, 0.0);
    molecule.position.z = 1e300;
    let input = HistoryInput::new(&[molecule; 2]).unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let error = run_history(&input, &network, &[], options(1.0), &mut rng).unwrap_err();
    let ChemistryError::IncompleteHistory { cause, diagnostics } = error else {
        panic!("{error:?}");
    };
    assert!(matches!(*cause, ChemistryError::PlacementDomain { .. }));
    assert_eq!(diagnostics.executed_reactions, 0);
    assert_eq!(diagnostics.pending_events, 1);
    let mut history = PairHistory::new(
        &input,
        &[network.get(1).unwrap()],
        PairHistoryOptions::default(),
        &mut rng,
    )
    .unwrap();
    assert!(matches!(
        history.observe_counts(Time::from_ps(1.0), &mut rng),
        Err(ChemistryError::PlacementDomain { .. })
    ));
    assert!(matches!(
        history.observe_counts(Time::from_ps(1.0), &mut rng),
        Err(ChemistryError::HistoryAlreadyFailed)
    ));
}
