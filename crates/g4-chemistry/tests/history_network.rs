//! Full option3 execution contracts, not a many-body physical validation gate.
//! Source baseline: Geant4 v11.4.1 G4EmDNAChemistry_option3.cc:146-664
//! and G4DNAIRT.cc:161-281,393-508 (read from the reference repository tag).

use g4_chemistry::history::{HistoryInput, HistoryOptions, run_history};
use g4_chemistry::reactions::{DEFAULT_REACTIONS_TOML, ReactionParticipant, ReactionTable};
use g4_chemistry::species::SpeciesTable;
use g4_chemistry::{ChemistryError, Species, SpeciesInstance};
use g4_core::rng::{RngStreams, Stage};
use g4_core::testing::ScriptedRng;
use g4_core::units::Time;
use g4_core::vec3::Vec3;

#[test]
fn mixed_species_full_network_preserves_lineage_and_every_observed_population() {
    let network = ReactionTable::option3()
        .unwrap()
        .compile(&SpeciesTable::option3().unwrap())
        .unwrap();
    assert_eq!(network.iter().count(), 72);
    let streams = RngStreams::new(20_261_106_006);
    let times = [1.0, 10.0, 100.0, 1_000.0, 100_000.0, 1_000_000.0].map(Time::from_ps);
    let mut pair_events = 0;
    let mut unary_events = 0;
    let mut descendants_reacted = 0;
    let mut electrostatic_regimes = [false; 3];
    let mut kinetic_regimes = [false; 2];
    for history_id in 0..64 {
        // Two of each configuration, including all unary parents, in a compact
        // spur. This synthetic input is not a physical irradiation distribution.
        let records: Vec<_> = Species::ALL
            .into_iter()
            .cycle()
            .take(30)
            .enumerate()
            .map(|(i, species)| SpeciesInstance {
                species,
                position: Vec3::new((i % 5) as f64, ((i / 5) % 3) as f64, (i / 15) as f64),
                time: Time::from_ps(1.0),
            })
            .collect();
        let input = HistoryInput::new(&records).unwrap();
        let mut rng = streams.stream(history_id, Stage::Chemistry).unwrap();
        let result = run_history(
            &input,
            &network,
            &times,
            HistoryOptions {
                max_trace_records: Some(10_000),
                ..Default::default()
            },
            &mut rng,
        )
        .unwrap();
        assert_eq!(input.snapshot(), records);
        assert_eq!(result.pre_irt_counts, [2; 15]);
        assert_eq!(result.diagnostics.time, times[5]);
        let trace = result.trace.as_ref().unwrap();
        assert_eq!(trace.len(), result.diagnostics.executed_reactions);
        let mut arena: Vec<_> = records.iter().map(|m| Some(m.species)).collect();
        let mut next_event = 0;
        let mut last_time = 1.0;
        for observation in &result.observations {
            while next_event < trace.len() && trace[next_event].time <= observation.time {
                let event = &trace[next_event];
                assert!(event.time.ps() >= last_time);
                last_time = event.time.ps();
                let descriptor = network.get(event.reaction_id).unwrap();
                let source = descriptor.source();
                let parents: Vec<_> = source
                    .reactants
                    .iter()
                    .filter_map(|p| match p {
                        ReactionParticipant::Mobile(s) => Some(*s),
                        ReactionParticipant::Bulk(_) => None,
                    })
                    .collect();
                assert_eq!(event.participants.len(), parents.len());
                assert_eq!(event.generations, vec![0; parents.len()]);
                for (id, species) in event.participants.iter().zip(parents) {
                    assert_eq!(
                        arena[id.0 as usize].take(),
                        Some(species),
                        "parent consumed once"
                    );
                    descendants_reacted += usize::from(id.0 >= records.len() as u64);
                }
                assert_eq!(event.products.len(), source.products.len());
                assert_eq!(event.placement.products.len(), source.products.len());
                for ((id, placed), expected) in event
                    .products
                    .iter()
                    .zip(&event.placement.products)
                    .zip(&source.products)
                {
                    assert_eq!(id.0 as usize, arena.len());
                    assert_eq!(ReactionParticipant::Mobile(placed.species), *expected);
                    assert!(
                        [placed.position.x, placed.position.y, placed.position.z]
                            .into_iter()
                            .all(f64::is_finite)
                    );
                    arena.push(Some(placed.species));
                }
                if let Some(pair) = descriptor.pair() {
                    pair_events += 1;
                    let charge = pair.onsager_radius().nm();
                    electrostatic_regimes[if charge < 0.0 {
                        0
                    } else if charge == 0.0 {
                        1
                    } else {
                        2
                    }] = true;
                    kinetic_regimes[usize::from(pair.pdc().is_some())] = true;
                } else {
                    unary_events += 1;
                }
                next_event += 1;
            }
            let mut counts = [0; 15];
            for species in arena.iter().flatten() {
                counts[species.index()] += 1;
            }
            assert_eq!(observation.counts, counts);
        }
        assert_eq!(next_event, trace.len());
        assert_eq!(result.post_contact_counts, result.observations[0].counts);
        assert_eq!(
            result.final_counts,
            result.observations.last().unwrap().counts
        );
        assert_eq!(arena.len(), result.diagnostics.arena_records);
    }
    assert!(pair_events > 0 && unary_events > 0 && descendants_reacted > 0);
    assert_eq!(electrostatic_regimes, [true; 3]);
    assert_eq!(kinetic_regimes, [true; 2]);
}

#[test]
fn unsupported_network_regimes_fail_before_history_sampling() {
    let species = SpeciesTable::option3().unwrap();
    let input = HistoryInput::new(&[]).unwrap();
    for (from, to, expected_id) in [
        ("rate_per_molar_s = 1.55e10", "rate_per_molar_s = 1e20", 11),
        ("source_class = \"I\"", "source_class = \"II\"", 1),
    ] {
        assert!(DEFAULT_REACTIONS_TOML.contains(from));
        let text = DEFAULT_REACTIONS_TOML.replacen(from, to, 1);
        let mut rng = ScriptedRng::new(&[]);
        let result = ReactionTable::from_toml(&text)
            .and_then(|table| table.compile(&species))
            .and_then(|network| {
                run_history(&input, &network, &[], HistoryOptions::default(), &mut rng)
            });
        match result.unwrap_err() {
            ChemistryError::InvalidPdcRate { reaction_id, .. }
            | ChemistryError::ReactionSourceMismatch { reaction_id } => {
                assert_eq!(reaction_id, expected_id)
            }
            other => panic!("unexpected rejection: {other:?}"),
        }
        assert_eq!(rng.consumed(), 0);
    }
}

#[test]
fn full_network_rejects_out_of_domain_horizons_before_any_draw() {
    let network = ReactionTable::option3()
        .unwrap()
        .compile(&SpeciesTable::option3().unwrap())
        .unwrap();
    let input = HistoryInput::new(&[SpeciesInstance {
        species: Species::H,
        position: Vec3::ZERO,
        time: Time::from_ps(1.0),
    }])
    .unwrap();
    for end in [0.0, 1_000_000.000_001, f64::INFINITY, f64::NAN] {
        let mut options = HistoryOptions::default();
        options.limits.end_time = Time::from_ps(end);
        let mut rng = ScriptedRng::new(&[]);
        assert!(matches!(
            run_history(&input, &network, &[], options, &mut rng),
            Err(ChemistryError::InvalidHistoryTime {
                field: "end time",
                ..
            })
        ));
        assert_eq!(rng.consumed(), 0);
    }
}
