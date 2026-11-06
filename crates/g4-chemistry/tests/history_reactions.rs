//! Product/unary history contracts against Geant4 11.4.1 G4DNAIRT.cc:212-281,
//! 393-508 and option3's executable product lists (including anomalous ID 53).
//! Exact identity, accounting and draw assertions are deterministic. These
//! scheduler fixtures do not validate many-body physics or establish R05/G2.

use g4_chemistry::diffusion::diffuse;
use g4_chemistry::history::{HistoryInput, PairHistory, PairHistoryOptions};
use g4_chemistry::irt::{ReactionTime, sample_independent_reaction_time};
use g4_chemistry::molecule::{Molecule, MoleculeId, MoleculePair};
use g4_chemistry::placement::place_pair_reaction;
use g4_chemistry::reaction_network::ReactionNetwork;
use g4_chemistry::reactions::{
    BulkParticipant, DEFAULT_REACTIONS_TOML, ReactionParticipant, ReactionTable,
};
use g4_chemistry::species::{SpeciesTable, atoms};
use g4_chemistry::{ChemistryError, Species, SpeciesInstance};
use g4_core::sampling::uniform;
use g4_core::testing::ScriptedRng;
use g4_core::units::Time;
use g4_core::vec3::Vec3;

fn network(rates: &[(u16, f64)]) -> ReactionNetwork {
    let mut id = 0;
    let text = DEFAULT_REACTIONS_TOML
        .lines()
        .map(|line| {
            if let Some(value) = line.strip_prefix("id = ") {
                id = value.parse().unwrap();
            }
            if line.starts_with("rate_per_s = ")
                && let Some((_, rate)) = rates.iter().find(|(channel, _)| *channel == id)
            {
                return format!("rate_per_s = {rate:?}");
            }
            line.to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n");
    ReactionTable::from_toml(&text)
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

fn inventory(records: impl IntoIterator<Item = Species>) -> [i64; 3] {
    let species = SpeciesTable::option3().unwrap();
    let mut total = [0; 3];
    for mobile in records {
        let (h, o) = atoms(mobile);
        for (sum, value) in total.iter_mut().zip([
            i64::from(h),
            i64::from(o),
            i64::from(species.get(mobile).charge),
        ]) {
            *sum += value;
        }
    }
    total
}

// Independently reconstruct inventory after each lineage event and compare the
// final live arena, including consumed solvent and reservoir atoms/charge.
fn assert_accounting(history: &PairHistory<'_>) {
    let initial = inventory(history.pre_irt_snapshot().iter().map(|p| p.species));
    let mut expected = initial;
    let mut alive: Vec<_> = (0..history.pre_irt_snapshot().len())
        .map(|id| MoleculeId(id as u64))
        .collect();
    for event in history.reactions() {
        let before = inventory(
            alive
                .iter()
                .map(|id| history.molecules()[id.0 as usize].instance().species),
        );
        for id in &event.participants {
            let index = alive
                .iter()
                .position(|p| p == id)
                .expect("each parent was alive");
            alive.remove(index);
        }
        alive.extend_from_slice(&event.products);
        let after = inventory(
            alive
                .iter()
                .map(|id| history.molecules()[id.0 as usize].instance().species),
        );
        assert_eq!(
            event.accounting.mobile_change,
            std::array::from_fn(|i| after[i] - before[i])
        );
        let bulk = match event.accounting.bulk_consumed {
            Some(BulkParticipant::H2OB) => [2, 1, 0],
            Some(BulkParticipant::H3OpB) => [3, 1, 1],
            Some(BulkParticipant::OHmB) => [1, 1, -1],
            Some(BulkParticipant::NoneM) | None => [0, 0, 0],
        };
        let water = i64::from(event.accounting.implicit_water);
        let exception = if event.reaction_id == 53 {
            [3, 0, 1]
        } else {
            [0; 3]
        };
        assert_eq!(event.accounting.source_exception, exception);
        expected = std::array::from_fn(|i| {
            expected[i] + bulk[i] - [2 * water, water, 0][i] + exception[i]
        });
        assert_eq!(after, expected, "after ID {}", event.reaction_id);
    }
    assert_eq!(
        expected,
        inventory(
            history
                .molecules()
                .iter()
                .filter(|p| p.is_alive())
                .map(|p| p.instance().species)
        )
    );
}

#[test]
fn every_source_channel_executes_with_ordered_product_ids_and_explicit_accounting() {
    let network = network(&[]);
    let mut product_counts = [false; 4];
    let mut exceptions = Vec::new();
    for reaction in network.iter() {
        let records: Vec<_> = reaction
            .source()
            .reactants
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if let ReactionParticipant::Mobile(species) = p {
                    Some(instance(*species, i as f64 * 1e-6))
                } else {
                    None
                }
            })
            .collect();
        let input = HistoryInput::new(&records).unwrap();
        // Tiny accepted contact/unary variate; all unary source rates fit 1 us.
        let mut rng = ScriptedRng::new(&[1e-12]);
        let mut history =
            PairHistory::new(&input, &[reaction], PairHistoryOptions::default(), &mut rng).unwrap();
        let time = history
            .next_event_time()
            .unwrap_or_else(|| panic!("missing ID {}", reaction.source().id));
        history.advance_to(time, &mut rng).unwrap();
        assert_eq!(history.reactions().len(), 1, "ID {}", reaction.source().id);
        let event = &history.reactions()[0];
        assert_eq!(event.reaction_id, reaction.source().id);
        assert_eq!(event.participants.len(), records.len());
        assert_eq!(event.products.len(), reaction.source().products.len());
        product_counts[event.products.len()] = true;
        for (index, id) in event.products.iter().enumerate() {
            assert_eq!(id.0 as usize, records.len() + index);
            let product = &history.molecules()[id.0 as usize];
            assert!(product.is_alive());
            assert_eq!(product.generation(), 0);
            assert_eq!(product.instance().time, time);
            assert_eq!(
                ReactionParticipant::Mobile(product.instance().species),
                reaction.source().products[index]
            );
            assert_eq!(
                product.instance().position,
                event.placement.products[index].position
            );
        }
        assert_eq!(
            history.molecules().len(),
            records.len() + event.products.len()
        );
        assert_eq!(history.pre_irt_snapshot(), records);
        assert_accounting(&history);
        if event.accounting.source_exception != [0; 3] {
            exceptions.push(event.reaction_id);
        }
    }
    assert_eq!(product_counts, [true; 4]);
    assert_eq!(exceptions, [53]);
}

#[test]
fn repeated_products_are_distinct_and_contacts_close_through_a_product_cascade() {
    let network = network(&[]);
    let input = HistoryInput::new(&[
        instance(Species::e_aq, 0.0),
        instance(Species::e_aq, 0.01),
        instance(Species::H3Op, 0.02),
    ])
    .unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let mut history = PairHistory::new(
        &input,
        &[network.get(9).unwrap(), network.get(8).unwrap()],
        PairHistoryOptions::default(),
        &mut rng,
    )
    .unwrap();
    history.close_contacts(&mut rng).unwrap();
    assert_eq!(
        history
            .reactions()
            .iter()
            .map(|r| r.reaction_id)
            .collect::<Vec<_>>(),
        [8, 9]
    );
    let first = &history.reactions()[0];
    assert_eq!(
        first.products,
        [MoleculeId(3), MoleculeId(4), MoleculeId(5)]
    );
    assert_eq!(
        first
            .placement
            .products
            .iter()
            .map(|p| p.species)
            .collect::<Vec<_>>(),
        [Species::OHm, Species::OHm, Species::H2]
    );
    assert_eq!(
        first.placement.products[1].position,
        input.snapshot()[0].position
    );
    assert_eq!(
        first.placement.products[2].position,
        input.snapshot()[1].position
    );
    assert_eq!(
        history.reactions()[1].participants,
        [MoleculeId(2), MoleculeId(3)]
    );
    assert!(history.reactions()[1].products.is_empty());
    assert_eq!(
        history
            .molecules()
            .iter()
            .filter(|p| p.is_alive())
            .map(|p| p.id())
            .collect::<Vec<_>>(),
        [MoleculeId(4), MoleculeId(5)]
    );
    assert!(history.reactions().iter().all(|r| r.time.ps() == 1.0));
    assert_eq!(history.pending_events(), 0);
    assert_eq!(rng.consumed(), 0);
    assert_accounting(&history);
}

#[test]
fn unary_channels_compete_in_source_id_order_and_do_not_create_bulk_molecules() {
    for rates in [[(55, 1e12), (68, 1e12)], [(55, 1e12), (68, 2e12)]] {
        let network = network(&rates);
        let input = HistoryInput::new(&[instance(Species::H, 3.0)]).unwrap();
        for ids in [[55, 68], [68, 55]] {
            let mut rng = ScriptedRng::new(&[0.5, 0.5]);
            let mut history = PairHistory::new(
                &input,
                &ids.map(|id| network.get(id).unwrap()),
                PairHistoryOptions::default(),
                &mut rng,
            )
            .unwrap();
            assert_eq!(history.pending_events(), 1); // Only the earliest unary can win.
            assert_eq!(rng.consumed(), 2);
            let time = history.next_event_time().unwrap();
            history.advance_to(time, &mut rng).unwrap();
            let event = &history.reactions()[0];
            let expected_id = if rates[1].1 > rates[0].1 { 68 } else { 55 };
            assert_eq!(event.reaction_id, expected_id);
            assert_eq!(event.participants, [MoleculeId(0)]);
            assert_eq!(event.generations, [0]);
            assert_eq!(
                event.placement.parent_positions,
                [Vec3::new(3.0, 0.0, 0.001); 2]
            );
            assert_eq!(history.molecules().len(), 1 + event.products.len());
            assert_eq!(rng.consumed(), 2); // Unary placement does not Brownian diffuse.
            assert_accounting(&history);
        }
    }
}

#[test]
fn disabled_and_out_of_window_unary_clocks_leave_the_parent_unmaterialized() {
    for rate in [0.0, 5.94] {
        let network = network(&[(55, rate)]);
        let records = [instance(Species::H, 4.0)];
        let input = HistoryInput::new(&records).unwrap();
        let mut rng = ScriptedRng::new(&[]);
        let mut history = PairHistory::new(
            &input,
            &[network.get(55).unwrap()],
            PairHistoryOptions::default(),
            &mut rng,
        )
        .unwrap();
        history.advance_to(Time::from_ps(1e6), &mut rng).unwrap();
        assert!(history.reactions().is_empty());
        assert_eq!(history.pending_events(), 0);
        assert_eq!(*history.molecules()[0].instance(), records[0]);
        assert_eq!(rng.consumed(), usize::from(rate != 0.0));
    }
}

#[test]
fn pair_and_unary_winners_invalidate_only_their_own_consumed_parents() {
    let baseline_network = network(&[]);
    let hh = baseline_network.get(1).unwrap();
    let records = [
        instance(Species::H, 0.0),
        instance(Species::H, 2.0 * hh.pair().unwrap().effective_radius().nm()),
    ];
    let input = HistoryInput::new(&records).unwrap();
    let mut probe_rng = ScriptedRng::new(&[0.1]);
    let probe =
        PairHistory::new(&input, &[hh], PairHistoryOptions::default(), &mut probe_rng).unwrap();
    let pair_delay = probe.next_event_time().unwrap().ps() - 1.0;
    for unary_wins in [false, true] {
        let unary_delay = pair_delay * if unary_wins { 0.25 } else { 2.0 };
        let rate = std::f64::consts::LN_2 * 1e12 / unary_delay;
        let network = network(&[(55, rate)]);
        let mut rng = ScriptedRng::new(&[0.1, 0.5, 0.75]);
        let mut history = PairHistory::new(
            &input,
            &[network.get(55).unwrap(), network.get(1).unwrap()],
            PairHistoryOptions::default(),
            &mut rng,
        )
        .unwrap();
        assert_eq!(history.pending_events(), 3);
        let time = history.next_event_time().unwrap();
        history.advance_to(time, &mut rng).unwrap();
        assert_eq!(history.reactions().len(), 1);
        assert_eq!(
            history.reactions()[0].reaction_id,
            if unary_wins { 55 } else { 1 }
        );
        assert_eq!(history.molecules()[1].is_alive(), unary_wins);
        history.advance_to(Time::from_ps(1e6), &mut rng).unwrap();
        assert_eq!(history.reactions().len(), if unary_wins { 2 } else { 1 });
        assert_eq!(rng.consumed(), if unary_wins { 3 } else { 9 });
        assert_eq!(history.pending_events(), 0);
        assert_accounting(&history);
    }
}

#[test]
fn unary_fallback_survives_an_earlier_pair_losing_its_other_parent() {
    let network = network(&[(55, 1e10)]);
    let ho = network.get(3).unwrap();
    let separation = 2.0 * ho.pair().unwrap().effective_radius().nm();
    let records = [
        instance(Species::H, 0.0),
        instance(Species::Oxy, separation),
        instance(Species::OH, separation + 0.01),
    ];
    let input = HistoryInput::new(&records).unwrap();
    let mut rng = ScriptedRng::new(&[0.1, 0.5]);
    let mut history = PairHistory::new(
        &input,
        &[ho, network.get(5).unwrap(), network.get(55).unwrap()],
        PairHistoryOptions::default(),
        &mut rng,
    )
    .unwrap();
    assert_eq!(history.pending_events(), 3);
    history.close_contacts(&mut rng).unwrap();
    assert_eq!(history.reactions()[0].reaction_id, 5);
    assert_eq!(
        history.reactions()[0].participants,
        [MoleculeId(2), MoleculeId(1)]
    );
    assert_eq!(*history.molecules()[0].instance(), records[0]);
    assert_eq!(history.pending_events(), 1);
    let fallback = history.next_event_time().unwrap();
    history.advance_to(fallback, &mut rng).unwrap();
    assert_eq!(history.reactions()[1].reaction_id, 55);
    assert_eq!(history.reactions()[1].participants, [MoleculeId(0)]);
    assert_eq!(rng.consumed(), 2);
    assert_accounting(&history);
}

#[test]
fn product_unary_clocks_start_at_birth_and_observation_splits_do_not_resample() {
    // H -> e_aq + H3O+, followed by e_aq -> H + OH-. Two different absolute
    // birth times prevent accidentally resetting product clocks to the 1 ps input.
    let network = network(&[(55, 1e12), (56, 1e12)]);
    let input = HistoryInput::new(&[instance(Species::H, 0.0)]).unwrap();
    let mut baseline = None;
    for split in [false, true] {
        let mut rng = ScriptedRng::new(&[0.5; 8]);
        let mut history = PairHistory::new(
            &input,
            &[network.get(56).unwrap(), network.get(55).unwrap()],
            PairHistoryOptions {
                end_time: Time::from_ps(2.5),
                ..Default::default()
            },
            &mut rng,
        )
        .unwrap();
        if split {
            history.advance_to(Time::from_ps(1.5), &mut rng).unwrap();
            history.advance_to(Time::from_ps(2.0), &mut rng).unwrap();
        }
        history.advance_to(Time::from_ps(2.5), &mut rng).unwrap();
        assert_eq!(history.reactions().len(), 2);
        let times: Vec<_> = history.reactions().iter().map(|r| r.time.ps()).collect();
        assert!((times[0] - (1.0 + std::f64::consts::LN_2)).abs() < 1e-14);
        assert!((times[1] - (1.0 + 2.0 * std::f64::consts::LN_2)).abs() < 1e-14);
        assert_eq!(history.reactions()[1].participants, [MoleculeId(1)]);
        assert_eq!(
            history.reactions()[1].products,
            [MoleculeId(3), MoleculeId(4)]
        );
        assert_eq!(rng.consumed(), 3);
        assert_accounting(&history);
        let result = (
            history.reactions().to_vec(),
            history.molecules().to_vec(),
            rng.consumed(),
        );
        if let Some(expected) = &baseline {
            assert_eq!(&result, expected);
        } else {
            baseline = Some(result);
        }
    }
}

#[test]
fn lazy_partner_diffusion_affects_only_clock_sampling_and_placement_uses_birth_delay() {
    // Oxy --59--> OH + OH at t>1; each new OH samples old Oxy through ID 5.
    // The old Oxy is source reactant B; stored positions never receive the
    // temporary Brownian displacement, and sibling clocks are sampled separately.
    let network = network(&[(59, 1e12)]);
    let reaction = network.get(5).unwrap();
    let records = [instance(Species::Oxy, 0.0), instance(Species::Oxy, 1.0)];
    let input = HistoryInput::new(&records).unwrap();
    // Two initial unary clocks, then two independent older-partner Brownian
    // samples followed by the pair clocks. First pair reacts; second escapes.
    let script = [
        0.5, 0.999999, 0.5, 0.5, 0.5, 0.5, 0.01, 0.5, 0.5, 0.5, 0.5, 0.99,
    ];
    let mut rng = ScriptedRng::new(&script);
    let mut history = PairHistory::new(
        &input,
        &[reaction, network.get(59).unwrap()],
        PairHistoryOptions::default(),
        &mut rng,
    )
    .unwrap();
    let birth = history.next_event_time().unwrap();
    history.advance_to(birth, &mut rng).unwrap();
    assert_eq!(history.reactions().len(), 1);
    assert_eq!(rng.consumed(), 12);
    assert_eq!(*history.molecules()[1].instance(), records[1]);
    assert_eq!(history.molecules()[1].generation(), 0);
    let new = history.molecules()[2];
    let mut expected_rng = ScriptedRng::new(&script);
    uniform(&mut expected_rng);
    uniform(&mut expected_rng);
    let old_position = diffuse(
        records[1].position,
        reaction.reactant_diffusivities()[1],
        Time::from_ps(birth.ps() - 1.0),
        &mut expected_rng,
    )
    .unwrap();
    assert_ne!(old_position, records[1].position);
    let mobile = Molecule::new(new.id(), *new.instance());
    let temporary = Molecule::new(
        MoleculeId(1),
        SpeciesInstance {
            position: old_position,
            ..records[1]
        },
    );
    let ReactionTime::After(delay) = sample_independent_reaction_time(
        MoleculePair::new(&mobile, &temporary).unwrap(),
        reaction.pair().unwrap(),
        &mut expected_rng,
    )
    .unwrap() else {
        panic!("finite separated product-pair fixture")
    };
    let expected_time = Time::from_ps(birth.ps() + delay.ps());
    assert_eq!(history.next_event_time(), Some(expected_time));
    // Skip the independently sampled second sibling, then use the original old
    // site and event-minus-birth delay, matching MakeReaction's stored tracks.
    for _ in 0..5 {
        uniform(&mut expected_rng);
    }
    let stored = Molecule::new(MoleculeId(1), records[1]);
    let expected_placement = place_pair_reaction(
        MoleculePair::new(&mobile, &stored).unwrap(),
        reaction,
        Time::from_ps(expected_time.ps() - birth.ps()),
        &mut expected_rng,
    )
    .unwrap();
    history.advance_to(expected_time, &mut rng).unwrap();
    assert_eq!(history.reactions().len(), 2);
    assert_eq!(
        history.reactions()[1].participants,
        [MoleculeId(2), MoleculeId(1)]
    );
    assert_eq!(history.reactions()[1].placement, expected_placement);
    assert_eq!(rng.consumed(), 18);
    assert_accounting(&history);
}

#[test]
fn old_source_reactant_a_uses_the_new_products_sampling_time_for_placement() {
    // OH --69--> Om; old H is source reactant A in H + Om -> OHm.
    // The placement delay must start at Om's birth, even though source A is older.
    let network = network(&[(69, 1e12)]);
    let reaction = network.get(4).unwrap();
    let records = [instance(Species::OH, 0.0), instance(Species::H, 1.0)];
    let input = HistoryInput::new(&records).unwrap();
    let script = [0.5, 0.5, 0.5, 0.5, 0.5, 0.01];
    let mut rng = ScriptedRng::new(&script);
    let mut history = PairHistory::new(
        &input,
        &[reaction, network.get(69).unwrap()],
        PairHistoryOptions::default(),
        &mut rng,
    )
    .unwrap();
    let birth = history.next_event_time().unwrap();
    history.advance_to(birth, &mut rng).unwrap();
    assert_eq!(*history.molecules()[1].instance(), records[1]);
    let product = history.molecules()[2];
    let mut expected_rng = ScriptedRng::new(&script);
    uniform(&mut expected_rng);
    let position = diffuse(
        records[1].position,
        reaction.reactant_diffusivities()[0],
        Time::from_ps(birth.ps() - 1.0),
        &mut expected_rng,
    )
    .unwrap();
    let moved = Molecule::new(
        MoleculeId(1),
        SpeciesInstance {
            position,
            ..records[1]
        },
    );
    let newborn = Molecule::new(product.id(), *product.instance());
    let ReactionTime::After(delay) = sample_independent_reaction_time(
        MoleculePair::new(&moved, &newborn).unwrap(),
        reaction.pair().unwrap(),
        &mut expected_rng,
    )
    .unwrap() else {
        panic!("separated product pair")
    };
    let time = Time::from_ps(birth.ps() + delay.ps());
    assert_eq!(history.next_event_time(), Some(time));
    let original = Molecule::new(MoleculeId(1), records[1]);
    let mut wrong_delay_rng = expected_rng.clone();
    let placement = place_pair_reaction(
        MoleculePair::new(&original, &newborn).unwrap(),
        reaction,
        Time::from_ps(time.ps() - birth.ps()),
        &mut expected_rng,
    )
    .unwrap();
    let wrong = place_pair_reaction(
        MoleculePair::new(&original, &newborn).unwrap(),
        reaction,
        Time::from_ps(time.ps() - 1.0),
        &mut wrong_delay_rng,
    )
    .unwrap();
    assert_ne!(placement, wrong);
    history.advance_to(time, &mut rng).unwrap();
    assert_eq!(
        history.reactions()[1].participants,
        [MoleculeId(1), MoleculeId(2)]
    );
    assert_eq!(history.reactions()[1].placement, placement);
    assert_eq!(rng.consumed(), 12); // Six initial draws, six placement; OHm is inert here.
    assert_accounting(&history);
}

#[test]
fn contact_products_and_unary_events_are_included_at_a_later_final_boundary() {
    let network = network(&[(69, 1e15)]);
    let input =
        HistoryInput::new(&[instance(Species::OH, 0.0), instance(Species::H, 0.0)]).unwrap();
    let descriptors = [network.get(4).unwrap(), network.get(69).unwrap()];
    let mut probe_rng = ScriptedRng::new(&[]);
    let probe = PairHistory::new(
        &input,
        &descriptors,
        PairHistoryOptions::default(),
        &mut probe_rng,
    )
    .unwrap();
    let boundary = probe.next_event_time().unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let mut history = PairHistory::new(
        &input,
        &descriptors,
        PairHistoryOptions {
            end_time: boundary,
            ..Default::default()
        },
        &mut rng,
    )
    .unwrap();
    history.advance_to(boundary, &mut rng).unwrap();
    assert_eq!(
        history
            .reactions()
            .iter()
            .map(|r| r.reaction_id)
            .collect::<Vec<_>>(),
        [69, 4]
    );
    assert!(history.reactions().iter().all(|r| r.time == boundary));
    assert_eq!(
        history.reactions()[1].placement.parent_positions,
        [Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.001)]
    );
    assert_eq!(rng.consumed(), 5); // One unary and four lazy-partner diffusion draws.
    assert_eq!(history.next_event_time(), None);
    assert_accounting(&history);
}

#[test]
fn product_capacity_is_checked_before_consuming_parents_and_failure_is_terminal() {
    let network = network(&[]);
    let input = HistoryInput::new(&[instance(Species::e_aq, 0.0); 2]).unwrap();
    let mut rng = ScriptedRng::new(&[]);
    let mut history = PairHistory::new(
        &input,
        &[network.get(8).unwrap()],
        PairHistoryOptions {
            max_molecules: 4,
            ..Default::default()
        },
        &mut rng,
    )
    .unwrap();
    assert_eq!(
        history.close_contacts(&mut rng),
        Err(ChemistryError::HistoryCapacity {
            resource: "molecules",
            limit: 4,
            required: 5
        })
    );
    assert!(history.reactions().is_empty());
    assert!(history.molecules().iter().all(|p| p.is_alive()));
    assert_eq!(history.pending_events(), 1);
    assert_eq!(
        history.close_contacts(&mut rng),
        Err(ChemistryError::HistoryAlreadyFailed)
    );
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn unrepresentable_unary_absolute_time_is_an_error_before_execution() {
    let network = network(&[(53, 1e300)]);
    let input = HistoryInput::new(&[instance(Species::O3m, 0.0)]).unwrap();
    let mut rng = ScriptedRng::new(&[0.5]);
    assert!(matches!(
        PairHistory::new(
            &input,
            &[network.get(53).unwrap()],
            PairHistoryOptions::default(),
            &mut rng
        ),
        Err(ChemistryError::InvalidHistoryTime {
            field: "absolute unary time",
            ..
        })
    ));
    assert_eq!(rng.consumed(), 1);
}
