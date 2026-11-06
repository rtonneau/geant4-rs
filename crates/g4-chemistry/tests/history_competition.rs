//! Deterministic competition fixtures; products do not react in these selected
//! networks. These scheduler checks do not establish G2.

use g4_chemistry::history::{HistoryInput, PairHistory, PairHistoryOptions};
use g4_chemistry::irt::{ReactionTime, sample_independent_reaction_time};
use g4_chemistry::molecule::{Molecule, MoleculeId, MoleculePair};
use g4_chemistry::reaction_network::ReactionNetwork;
use g4_chemistry::reactions::ReactionTable;
use g4_chemistry::species::SpeciesTable;
use g4_chemistry::{Species, SpeciesInstance};
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
fn competing_channels_use_canonical_ids_and_retain_source_ordered_lineage() {
    let network = network();
    let records = [
        instance(Species::Oxy, 0.0),
        instance(Species::OH, 0.01),
        instance(Species::H, 0.02),
    ];
    let input = HistoryInput::new(&records).unwrap();
    for ids in [[3, 5], [5, 3]] {
        let mut rng = ScriptedRng::new(&[]);
        let mut history = PairHistory::new(
            &input,
            &ids.map(|id| network.get(id).unwrap()),
            PairHistoryOptions {
                max_reactions: 1,
                ..Default::default()
            },
            &mut rng,
        )
        .unwrap();
        assert_eq!(history.pending_events(), 2);
        history.close_contacts(&mut rng).unwrap();
        assert_eq!(history.reactions().len(), 1);
        let event = &history.reactions()[0];
        assert_eq!(event.reaction_id, 5); // IDs (0, 1), before (0, 2).
        assert_eq!(event.participants, [MoleculeId(1), MoleculeId(0)]);
        assert_eq!(event.generations, [0, 0]);
        assert_eq!(event.time.ps(), 1.0);
        assert_eq!(event.placement.products[0].species, Species::HO2);
        for (id, position) in event
            .participants
            .iter()
            .copied()
            .zip(event.placement.parent_positions)
        {
            let parent = &history.molecules()[id.0 as usize];
            assert!(!parent.is_alive());
            assert_eq!(parent.generation(), 1);
            assert_eq!(parent.instance().position, position);
            assert_eq!(parent.instance().time, event.time);
        }
        assert!(history.molecules()[2].is_alive());
        assert_eq!(*history.molecules()[2].instance(), records[2]);
        assert_eq!(history.molecules()[2].generation(), 0);
        assert_eq!(history.pending_events(), 0);
        assert_eq!(history.pre_irt_snapshot(), records);
        assert_eq!(rng.consumed(), 0);
    }
}

#[test]
fn three_way_identical_contact_tie_has_three_pairs_and_one_winner() {
    let network = network();
    let input = HistoryInput::new(&[instance(Species::H, 0.0); 3]).unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let mut history = PairHistory::new(
        &input,
        &[network.get(1).unwrap()],
        PairHistoryOptions {
            max_pending_events: 3,
            max_reactions: 1,
            ..Default::default()
        },
        &mut rng,
    )
    .unwrap();
    assert_eq!(history.pending_events(), 3); // N(N-1)/2, never self-pairs or reversals.
    history.close_contacts(&mut rng).unwrap();
    assert_eq!(history.reactions().len(), 1);
    assert_eq!(
        history.reactions()[0].participants,
        [MoleculeId(0), MoleculeId(1)]
    );
    assert_eq!(history.reactions()[0].generations, [0, 0]);
    assert!(history.molecules()[2].is_alive());
    assert_eq!(history.molecules()[2].generation(), 0);
    assert_eq!(history.next_event_time(), None);
    history
        .advance_to(Time::from_ps(1_000_000.0), &mut rng)
        .unwrap();
    assert_eq!(history.reactions().len(), 1);
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn a_later_inserted_earlier_clock_wins_and_stale_clocks_do_not_place_parents() {
    let network = network();
    let reaction = network.get(1).unwrap();
    let radius = reaction.pair().unwrap().effective_radius().nm();
    let records = [
        instance(Species::H, 0.0),
        instance(Species::H, 2.0 * radius),
        instance(Species::H, -2.0 * radius),
    ];
    let input = HistoryInput::new(&records).unwrap();
    let mut rng = ScriptedRng::new(&[0.3, 0.1, 0.1]);
    let mut history = PairHistory::new(
        &input,
        &[reaction],
        PairHistoryOptions {
            max_reactions: 1,
            ..Default::default()
        },
        &mut rng,
    )
    .unwrap();
    assert_eq!(rng.consumed(), 3);
    assert_eq!(history.pending_events(), 3);
    // Independently sample only (0, 2) to pin the absolute winning clock.
    let first = Molecule::new(MoleculeId(0), records[0]);
    let second = Molecule::new(MoleculeId(2), records[2]);
    let mut expected_rng = ScriptedRng::new(&[0.1]);
    let ReactionTime::After(delay) = sample_independent_reaction_time(
        MoleculePair::new(&first, &second).unwrap(),
        reaction.pair().unwrap(),
        &mut expected_rng,
    )
    .unwrap() else {
        panic!("separated fixture must have a finite clock")
    };
    let expected_time = Time::from_ps(1.0 + delay.ps());
    assert_eq!(history.next_event_time(), Some(expected_time));
    history.advance_to(expected_time, &mut rng).unwrap();
    assert_eq!(history.reactions().len(), 1);
    assert_eq!(
        history.reactions()[0].participants,
        [MoleculeId(0), MoleculeId(2)]
    );
    assert_eq!(history.reactions()[0].time, expected_time);
    assert_eq!(history.next_event_time(), None);
    history
        .advance_to(Time::from_ps(1_000_000.0), &mut rng)
        .unwrap();
    assert_eq!(rng.consumed(), 9); // Three initial clocks, one six-draw placement.
    assert_eq!(*history.molecules()[1].instance(), records[1]);
    assert!(history.molecules()[1].is_alive());
}

#[test]
fn unaffected_clock_and_placement_survive_competition_and_intermediate_advances() {
    let network = network();
    let hh = network.get(1).unwrap();
    let oo = network.get(7).unwrap();
    let records = [
        instance(Species::H, 0.0),
        instance(Species::H, 0.01),
        instance(Species::H, 2.0 * hh.pair().unwrap().effective_radius().nm()),
        instance(Species::Oxy, 100.0),
        instance(
            Species::Oxy,
            100.0 + 2.0 * oo.pair().unwrap().effective_radius().nm(),
        ),
    ];
    let input = HistoryInput::new(&records).unwrap();
    let mut baseline_rng = ScriptedRng::new(&[0.1]);
    let mut baseline = PairHistory::new(
        &input,
        &[oo],
        PairHistoryOptions::default(),
        &mut baseline_rng,
    )
    .unwrap();
    let expected_time = baseline.next_event_time().unwrap();
    baseline
        .advance_to(expected_time, &mut baseline_rng)
        .unwrap();
    let mut expected_trace = None;
    for split in [false, true] {
        let mut rng = ScriptedRng::new(&[0.1; 3]);
        let mut history = PairHistory::new(
            &input,
            &[oo, hh],
            PairHistoryOptions {
                max_reactions: 2,
                ..Default::default()
            },
            &mut rng,
        )
        .unwrap();
        if split {
            history.close_contacts(&mut rng).unwrap();
            assert_eq!(history.next_event_time(), Some(expected_time));
            let intermediate = Time::from_ps(1.0 + (expected_time.ps() - 1.0) * 0.5);
            history.advance_to(intermediate, &mut rng).unwrap();
            assert_eq!(rng.consumed(), 3);
            assert_eq!(*history.molecules()[3].instance(), records[3]);
        }
        history
            .advance_to(Time::from_ps(1_000_000.0), &mut rng)
            .unwrap();
        assert_eq!(history.reactions().len(), 2);
        let later = &history.reactions()[1];
        let baseline_later = &baseline.reactions()[0];
        assert_eq!(later.reaction_id, baseline_later.reaction_id);
        assert_eq!(later.time, baseline_later.time);
        assert_eq!(later.participants, baseline_later.participants);
        assert_eq!(later.generations, baseline_later.generations);
        assert_eq!(later.placement, baseline_later.placement);
        assert_eq!(later.accounting, baseline_later.accounting);
        // The unrelated earlier event created one product, shifting later IDs.
        assert_eq!(later.products[0].0, baseline_later.products[0].0 + 1);
        assert_eq!(rng.consumed(), baseline_rng.consumed() + 2);
        assert_eq!(history.pending_events(), 0);
        assert!(history.molecules()[2].is_alive());
        assert_eq!(*history.molecules()[2].instance(), records[2]);
        if let Some((reactions, molecules)) = &expected_trace {
            assert_eq!(history.reactions(), reactions);
            assert_eq!(history.molecules(), molecules);
        } else {
            expected_trace = Some((history.reactions().to_vec(), history.molecules().to_vec()));
        }
    }
}
