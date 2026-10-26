//! M6-B scoring and end-to-end checks.
//!
//! Thresholds were fixed before the first execution of this file. Statistical validation of the
//! channel and displacement samplers remains in `dna-chemistry/tests/{channel_statistics,
//! displacement_statistics}.rs`: their headers define the two Bonferroni families (six channel
//! chi-square tests at family alpha 1e-3; displacement mean-square components at family alpha
//! 1e-3). This file introduces no multiple-comparison p-value family.
//!
//! A-priori physical sanity for a fast-electron track at 10 keV, molecules/100 eV at 1 ps:
//! e_aq 2--6, OH 3--8, H3O+ 2--6, H 0.2--2, H2 0.1--1.5. Reasoning: M3 makes about 4.9
//! ionisations and 4.8 terminal electrons per 100 eV; most unrecombined ionisations yield one
//! H3O+ and OH, while electron-hole recombination reduces both charged yields and creates H/OH/H2
//! fragments. Excitation adds OH/H and only minority branches form H2. These deliberately broad
//! bounds are a sanity check, not a Geant4 or experimental validation.

use dna_chemistry::PhysChemEngine;
use dna_chemistry::Species;
use dna_core::manifest::Manifest;
use dna_core::physchem::{PhysChemEvent, PhysChemKind};
use dna_core::rng::{RngStreams, Stage};
use dna_core::testing::ScriptedRng;
use dna_core::units::Energy;
use dna_core::vec3::Vec3;
use dna_data::datadir::{DataDir, repo_data_dir};
use dna_physics::models::{ElasticRotation, ProcessKind};
use dna_physics::table::ProcessTable;
use dna_physics::transport::Termination;
use dna_sim::config::RunConfig;
use dna_sim::physchem_scoring::PhysChemScorer;
use dna_sim::run::{run_detailed, run_physchem_summary};

fn setup() -> Option<ProcessTable> {
    let root = repo_data_dir();
    if !root.join("dna").is_dir() || !root.join("MANIFEST.toml").is_file() {
        eprintln!("SKIPPED: Geant4 data not found at {}", root.display());
        return None;
    }
    Manifest::load(&root.join("MANIFEST.toml")).unwrap();
    Some(
        ProcessTable::option2_electron(&DataDir::new(&root).unwrap(), ElasticRotation::Geant4)
            .unwrap(),
    )
}

fn score(
    table: &ProcessTable,
    threads: usize,
    histories: u64,
) -> dna_sim::physchem_scoring::PhysChemSummary {
    let mut config = RunConfig::new(10_000.0, histories);
    config.master_seed = 20261016;
    config.batch_size = 7;
    config.threads = Some(threads);
    run_physchem_summary(table, &PhysChemEngine::option3().unwrap(), &config)
        .unwrap()
        .1
}

fn g(summary: &dna_sim::physchem_scoring::PhysChemSummary, species: Species) -> f64 {
    summary.species[species.index()]
        .g_ratio_of_totals
        .unwrap()
        .value
}

#[test]
fn physchem_counts_and_radius_use_survivors_at_global_one_ps() {
    // Primary starts at zero; this ionisation is 100 nm down its track. One electron
    // is consumed at 101 nm and the other survives at (0, 6, 8), exactly 10 nm away.
    let event = |kind, position| PhysChemEvent {
        kind,
        level_or_shell: 0,
        position,
        deposited_energy: Energy::from_ev(10.0),
        parent_history: 0,
    };
    let parent = Vec3::new(100.0, 0.0, 0.0);
    let events = [
        event(PhysChemKind::Solvation, Vec3::new(101.0, 0.0, 0.0)),
        event(PhysChemKind::Solvation, Vec3::new(0.0, 6.0, 8.0)),
        event(PhysChemKind::Ionisation, parent),
    ];
    // p(1 nm) = 0.509159... > 0.50; H2Ovib channel 0.20 makes OH + H.
    let mut rng = ScriptedRng::new(&[0.50, 0.20, 0.3, 0.4, 0.5, 0.6]);
    let chemistry = PhysChemEngine::option3()
        .unwrap()
        .convert_history(&events, &mut rng)
        .unwrap();
    assert_eq!(chemistry.stats.recombinations, 1);
    assert!(chemistry.species.iter().all(|s| s.time.ps() == 1.0));
    let mut scorer = PhysChemScorer::default();
    scorer.add(&chemistry, 100.0);
    let summary = scorer.finish();
    assert_eq!(summary.time_ps, 1.0);
    let electron = &summary.species[Species::e_aq.index()];
    assert_eq!(electron.produced_by_event, [0, 0, 0, 2]);
    assert_eq!(electron.alive_total, 1);
    assert_eq!(electron.alive_per_history.value, 1.0);
    assert_eq!(electron.rms_radius_nm, Some(10.0));
    assert_eq!(electron.rms_product_displacement_nm, None);
    for species in [Species::OH, Species::H] {
        let product = chemistry
            .species
            .iter()
            .find(|s| s.species == species)
            .unwrap();
        let score = &summary.species[species.index()];
        assert_eq!(score.alive_total, 1);
        let radius = product.position.norm();
        let displacement = (product.position - parent).norm();
        assert!((score.rms_radius_nm.unwrap() - radius).abs() < 1e-12);
        assert!((score.rms_product_displacement_nm.unwrap() - displacement).abs() < 1e-12);
        assert!(radius > 90.0 && displacement < 10.0);
    }
}

#[test]
fn physchem_cascade_follows_all_electrons_without_a_secondary_floor() {
    let Some(table) = setup() else { return };
    assert_eq!(table.primary_energy_range().min().ev(), 2.0);
    let mut config = RunConfig::new(1_000.0, 8);
    config.master_seed = 20261017;
    config.record_physchem = true;
    assert!(config.transport_secondaries);
    assert_eq!(config.tracking_cut_ev, None);
    assert_eq!(config.max_interactions, None);
    let histories = run_detailed(&table, &config).unwrap();
    let mut sub_floor_solvations = 0;
    for history in histories {
        let outcome = &history.outcome;
        assert_eq!(outcome.termination, Termination::Completed);
        assert_eq!(outcome.n_tracks, 1 + outcome.n_secondaries);
        assert_eq!(
            outcome.n_tracks,
            outcome.solvations + outcome.counts.get(ProcessKind::Attachment)
        );
        assert_eq!(outcome.ledger.untracked_secondaries, 0.0);
        assert_eq!(outcome.ledger.remaining_untracked, 0.0);
        sub_floor_solvations += history
            .physchem
            .iter()
            .filter(|event| {
                event.kind == PhysChemKind::Solvation && event.deposited_energy.ev() < 2.0
            })
            .count();
    }
    assert!(sub_floor_solvations > 0);
    assert!(run_detailed(&table, &RunConfig::new(2.0, 1)).is_ok());
    assert!(run_detailed(&table, &RunConfig::new(1.99, 1)).is_err());
}

#[test]
fn initial_yields_and_spatial_scores_are_physically_sane() {
    let Some(table) = setup() else { return };
    let summary = score(&table, 4, 30);
    for (species, low, high) in [
        (Species::e_aq, 2.0, 6.0),
        (Species::OH, 3.0, 8.0),
        (Species::H3Op, 2.0, 6.0),
        (Species::H, 0.2, 2.0),
        (Species::H2, 0.1, 1.5),
    ] {
        let value = g(&summary, species);
        eprintln!(
            "G({}) = {value:.4} molecules/100 eV",
            species.chemistry_name()
        );
        assert!(
            (low..=high).contains(&value),
            "{}: {value}",
            species.chemistry_name()
        );
    }
    assert_eq!(summary.time_ps, 1.0);
    assert!(summary.recombinations > 0);
    for species in &summary.species {
        if species.alive_total > 0 {
            assert!(species.rms_radius_nm.unwrap().is_finite());
        }
        if let Some(rms) = species.rms_product_displacement_nm {
            assert!(rms.is_finite());
        }
    }
    assert!(summary.pair_distances.iter().all(|p| p.count > 0));
}

#[test]
fn physchem_scores_are_bitwise_thread_count_independent() {
    let Some(table) = setup() else { return };
    let one = score(&table, 1, 12);
    assert_eq!(one, score(&table, 2, 12));
    assert_eq!(one, score(&table, 8, 12));
}

#[test]
fn recombination_ablation_and_event_order_sensitivity_are_reported() {
    let Some(table) = setup() else { return };
    let mut config = RunConfig::new(10_000.0, 30);
    config.master_seed = 20261018;
    config.record_physchem = true;
    let histories = run_detailed(&table, &config).unwrap();
    let streams = RngStreams::new(config.master_seed);
    let baseline = PhysChemEngine::option3().unwrap();
    let disabled = PhysChemEngine::option3()
        .unwrap()
        .with_recombination_enabled(false);
    let mut totals = [[0u64; 15]; 3];
    let mut recombinations = [0u64; 3];
    let mut ions = [0u64; 3];
    for history in histories {
        let mut reversed = history.physchem.clone();
        reversed.reverse();
        for (i, (engine, events)) in [
            (&baseline, history.physchem.as_slice()),
            (&disabled, history.physchem.as_slice()),
            (&baseline, reversed.as_slice()),
        ]
        .into_iter()
        .enumerate()
        {
            let mut rng = streams.stream(history.id, Stage::PhysChem).unwrap();
            let chemistry = engine.convert_history(events, &mut rng).unwrap();
            recombinations[i] += chemistry.stats.recombinations;
            ions[i] += chemistry.stats.events[1];
            for (a, b) in totals[i].iter_mut().zip(chemistry.stats.species_counts) {
                *a += b;
            }
        }
    }
    eprintln!(
        "recombination fractions: baseline {:.4}, off {:.4}, reversed {:.4}",
        recombinations[0] as f64 / ions[0] as f64,
        recombinations[1] as f64 / ions[1] as f64,
        recombinations[2] as f64 / ions[2] as f64
    );
    assert_eq!(recombinations[1], 0);
    assert_ne!(
        totals[0], totals[1],
        "recombination ablation must change yields"
    );
    assert_ne!(
        totals[0], totals[2],
        "water-event ordering must be measured, not assumed irrelevant"
    );
}
