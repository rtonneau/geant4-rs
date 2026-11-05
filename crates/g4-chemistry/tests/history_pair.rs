//! Isolated-pair regressions with products that do not react in these selected
//! networks. Exact fixtures assert scheduler semantics, not G2 acceptance.

use g4_chemistry::history::{HistoryInput, PairHistory, PairHistoryOptions};
use g4_chemistry::molecule::MoleculeId;
use g4_chemistry::reaction_network::ReactionNetwork;
use g4_chemistry::reactions::ReactionTable;
use g4_chemistry::species::SpeciesTable;
use g4_chemistry::{ChemistryError, Species, SpeciesInstance};
use g4_core::testing::ScriptedRng;
use g4_core::units::Time;
use g4_core::vec3::Vec3;

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

#[test]
fn empty_and_nonreacting_inputs_preserve_the_snapshot_and_rng() {
    let network = network();
    for records in [vec![], vec![instance(Species::H2, 4.0)]] {
        let input = HistoryInput::new(&records).unwrap();
        let mut rng = ScriptedRng::new(&[]);
        let mut history = PairHistory::new(
            &input,
            &[network.get(1).unwrap()],
            PairHistoryOptions::default(),
            &mut rng,
        )
        .unwrap();
        history.advance_to(Time::from_ps(100.0), &mut rng).unwrap();
        assert_eq!(input.snapshot(), records);
        assert_eq!(history.pre_irt_snapshot(), records);
        assert!(history.reactions().is_empty());
        assert_eq!(history.molecules().len(), records.len());
        assert_eq!(rng.consumed(), 0);
    }
}

#[test]
fn contact_closure_consumes_each_pair_once_in_canonical_order() {
    let network = network();
    let records = [
        instance(Species::Oxy, 0.0),
        instance(Species::H, 0.01),
        instance(Species::H3Op, 100.0),
        instance(Species::OHm, 100.01),
    ];
    let input = HistoryInput::new(&records).unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let mut history = PairHistory::new(
        &input,
        // Deliberately reversed descriptor order; candidate order is by IDs.
        &[network.get(9).unwrap(), network.get(3).unwrap()],
        PairHistoryOptions::default(),
        &mut rng,
    )
    .unwrap();
    assert_eq!(history.pending_events(), 2);
    assert_eq!(history.molecules()[0].id(), MoleculeId(0));
    assert!(history.molecules().iter().all(|m| m.is_alive()));
    history.close_contacts(&mut rng).unwrap();
    assert_eq!(history.reactions().len(), 2);
    assert_eq!(history.reactions()[0].reaction_id, 3);
    assert_eq!(
        history.reactions()[0].participants,
        [MoleculeId(1), MoleculeId(0)]
    );
    assert_eq!(history.reactions()[1].reaction_id, 9);
    assert!(history.reactions().iter().all(|r| r.time.ps() == 1.0));
    assert_eq!(
        history.reactions()[0].placement.products[0].species,
        Species::OH
    );
    assert!(
        history
            .molecules()
            .iter()
            .take(records.len())
            .all(|m| !m.is_alive() && m.generation() == 1)
    );
    assert_eq!(history.pending_events(), 0);
    history.close_contacts(&mut rng).unwrap();
    assert_eq!(history.reactions().len(), 2);
    assert_eq!(history.pre_irt_snapshot(), records);
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn separated_identical_pair_uses_one_clock_and_inclusive_final_boundary() {
    let network = network();
    let reaction = network.get(1).unwrap();
    let radius = reaction.pair().unwrap().effective_radius().nm();
    let records = [
        instance(Species::H, 0.0),
        instance(Species::H, radius * 2.0),
    ];
    let input = HistoryInput::new(&records).unwrap();
    let mut rng = ScriptedRng::new(&[0.1]);
    let mut history =
        PairHistory::new(&input, &[reaction], PairHistoryOptions::default(), &mut rng).unwrap();
    assert_eq!(rng.consumed(), 1);
    let event_time = history.next_event_time().unwrap();
    assert!(event_time.ps() > 1.0);
    history.close_contacts(&mut rng).unwrap();
    assert!(history.reactions().is_empty());
    history.advance_to(event_time, &mut rng).unwrap();
    assert_eq!(history.reactions().len(), 1);
    assert_eq!(
        history.reactions()[0].participants,
        [MoleculeId(0), MoleculeId(1)]
    );
    assert_eq!(history.reactions()[0].time, event_time);
    assert_eq!(rng.consumed(), 7); // one clock, six conditional-placement draws

    let mut rng = ScriptedRng::new(&[0.1]);
    let mut bounded = PairHistory::new(
        &input,
        &[reaction],
        PairHistoryOptions {
            end_time: event_time,
            ..Default::default()
        },
        &mut rng,
    )
    .unwrap();
    bounded.advance_to(event_time, &mut rng).unwrap();
    assert_eq!(bounded.reactions(), history.reactions());
}

#[test]
fn no_reaction_or_beyond_horizon_keeps_survivors_unmaterialized() {
    let network = network();
    let reaction = network.get(1).unwrap();
    let radius = reaction.pair().unwrap().effective_radius().nm();
    let input = HistoryInput::new(&[
        instance(Species::H, 0.0),
        instance(Species::H, radius * 2.0),
    ])
    .unwrap();
    for draw in [0.1, 0.9] {
        let mut rng = ScriptedRng::new(&[draw]);
        let mut history = PairHistory::new(
            &input,
            &[reaction],
            PairHistoryOptions {
                end_time: Time::from_ps(1.0),
                ..Default::default()
            },
            &mut rng,
        )
        .unwrap();
        history.close_contacts(&mut rng).unwrap();
        assert!(history.reactions().is_empty());
        assert_eq!(history.pending_events(), 0);
        assert!(
            history
                .molecules()
                .iter()
                .all(|m| m.is_alive() && m.instance().time.ps() == 1.0)
        );
        assert_eq!(rng.consumed(), 1);
    }
}

#[test]
fn invalid_inputs_times_and_unsupported_slices_fail_before_draws() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, 0.0, 2.0] {
        let mut record = instance(Species::H, 0.0);
        record.time = Time::from_ps(bad);
        assert!(matches!(
            HistoryInput::new(&[record]),
            Err(ChemistryError::InvalidHistoryTime { .. })
        ));
    }
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut record = instance(Species::H, 0.0);
        record.position.y = bad;
        assert!(matches!(
            HistoryInput::new(&[record]),
            Err(ChemistryError::InvalidIrtInput { .. })
        ));
    }
    let network = network();
    let input = HistoryInput::new(&[instance(Species::H, 0.0)]).unwrap();
    let mut rng = ScriptedRng::new(&[]);
    for bad in [f64::NAN, f64::INFINITY, -1.0, 0.0, 1_000_001.0] {
        assert!(matches!(
            PairHistory::new(
                &input,
                &[],
                PairHistoryOptions {
                    end_time: Time::from_ps(bad),
                    ..Default::default()
                },
                &mut rng
            ),
            Err(ChemistryError::InvalidHistoryTime { .. })
        ));
    }
    assert!(
        PairHistory::new(
            &input,
            &[network.get(53).unwrap()],
            PairHistoryOptions::default(),
            &mut rng
        )
        .is_ok()
    ); // A unary descriptor with no matching mobile parent consumes no draws.
    assert!(matches!(
        PairHistory::new(
            &input,
            &[network.get(1).unwrap(); 2],
            PairHistoryOptions::default(),
            &mut rng
        ),
        Err(ChemistryError::DuplicateHistoryReaction { reaction_id: 1 })
    ));
    let mut history =
        PairHistory::new(&input, &[], PairHistoryOptions::default(), &mut rng).unwrap();
    history.advance_to(Time::from_ps(2.0), &mut rng).unwrap();
    for bad in [1.0, -1.0, f64::NAN, f64::INFINITY, 1_000_001.0] {
        assert!(matches!(
            history.advance_to(Time::from_ps(bad), &mut rng),
            Err(ChemistryError::InvalidHistoryTime { .. })
        ));
    }
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn capacities_fail_explicitly_without_consuming_an_event() {
    let network = network();
    let input =
        HistoryInput::new(&[instance(Species::H, 0.0), instance(Species::H, 0.01)]).unwrap();
    let mut rng = ScriptedRng::new(&[]);
    for options in [
        PairHistoryOptions {
            max_molecules: 1,
            ..Default::default()
        },
        PairHistoryOptions {
            max_pending_events: 0,
            ..Default::default()
        },
    ] {
        assert!(matches!(
            PairHistory::new(&input, &[network.get(1).unwrap()], options, &mut rng),
            Err(ChemistryError::HistoryCapacity { .. })
        ));
    }
    let mut history = PairHistory::new(
        &input,
        &[network.get(1).unwrap()],
        PairHistoryOptions {
            max_reactions: 0,
            ..Default::default()
        },
        &mut rng,
    )
    .unwrap();
    assert!(matches!(
        history.close_contacts(&mut rng),
        Err(ChemistryError::HistoryCapacity {
            resource: "reactions",
            ..
        })
    ));
    assert_eq!(history.pending_events(), 1);
    assert!(history.molecules().iter().all(|m| m.is_alive()));
    assert!(history.reactions().is_empty());
    assert!(matches!(
        history.close_contacts(&mut rng),
        Err(ChemistryError::HistoryAlreadyFailed)
    ));
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn unrepresentable_absolute_delay_and_failed_placement_are_errors() {
    let network = network();
    let reaction = network.get(1).unwrap();
    let radius = reaction.pair().unwrap().effective_radius().nm();
    // The elapsed clock is positive, but adding it to 1 ps would erase it.
    let adjacent = f64::from_bits(radius.to_bits() + 1);
    let input =
        HistoryInput::new(&[instance(Species::H, 0.0), instance(Species::H, adjacent)]).unwrap();
    let mut rng = ScriptedRng::new(&[0.1]);
    assert!(matches!(
        PairHistory::new(&input, &[reaction], PairHistoryOptions::default(), &mut rng),
        Err(ChemistryError::InvalidHistoryTime {
            field: "absolute pair time",
            ..
        })
    ));
    assert_eq!(rng.consumed(), 1);

    let mut coincident = instance(Species::H, 0.0);
    coincident.position.z = 1e300;
    let input = HistoryInput::new(&[coincident; 2]).unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let mut history =
        PairHistory::new(&input, &[reaction], PairHistoryOptions::default(), &mut rng).unwrap();
    // The source's coincident-parent 0.001 nm offset cannot be represented.
    assert!(matches!(
        history.close_contacts(&mut rng),
        Err(ChemistryError::PlacementDomain { .. })
    ));
    assert_eq!(history.pending_events(), 1);
    assert!(history.molecules().iter().all(|m| m.is_alive()));
    assert!(matches!(
        history.advance_to(Time::from_ps(2.0), &mut rng),
        Err(ChemistryError::HistoryAlreadyFailed)
    ));
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn descriptor_order_and_intermediate_advances_preserve_pair_events_and_rng() {
    let network = network();
    let hh = network.get(1).unwrap();
    let oo = network.get(7).unwrap();
    let input = HistoryInput::new(&[
        instance(Species::H, 0.0),
        instance(Species::H, hh.pair().unwrap().effective_radius().nm() * 2.0),
        instance(Species::Oxy, 100.0),
        instance(Species::Oxy, 100.01),
    ])
    .unwrap();
    let mut direct_rng = ScriptedRng::new(&[0.1]);
    let mut direct = PairHistory::new(
        &input,
        &[hh, oo],
        PairHistoryOptions::default(),
        &mut direct_rng,
    )
    .unwrap();
    let mut split_rng = ScriptedRng::new(&[0.1]);
    let mut split = PairHistory::new(
        &input,
        &[oo, hh],
        PairHistoryOptions::default(),
        &mut split_rng,
    )
    .unwrap();
    split.close_contacts(&mut split_rng).unwrap();
    assert_eq!(split.reactions()[0].reaction_id, 7);
    assert_eq!(split_rng.consumed(), 1);
    let later_time = split.next_event_time().unwrap();
    let before = Time::from_ps(1.0 + (later_time.ps() - 1.0) * 0.5);
    split.advance_to(before, &mut split_rng).unwrap();
    assert_eq!(split.current_time(), before);
    assert_eq!(split.next_event_time(), Some(later_time));
    assert_eq!(split_rng.consumed(), 1);
    assert_eq!(split.molecules()[0].instance().time.ps(), 1.0);
    split.advance_to(later_time, &mut split_rng).unwrap();
    direct.advance_to(later_time, &mut direct_rng).unwrap();
    assert_eq!(split.reactions(), direct.reactions());
    assert_eq!(split.molecules(), direct.molecules());
    assert_eq!(split_rng.consumed(), direct_rng.consumed());
}
