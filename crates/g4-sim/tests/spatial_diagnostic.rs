//! Validation of the preregistered schema, inference and observational instrumentation.
//! Numeric tolerances below (1e-12) test arithmetic, not an M6 scientific acceptance margin.

use g4_chemistry::Species;
use g4_core::rng::{HistoryRng, RngStreams, Stage};
use g4_core::vec3::Vec3;
use g4_sim::spatial_diagnostic::*;
use std::collections::BTreeMap;
use std::io::Cursor;

fn physics_table() -> g4_physics::table::ProcessTable {
    g4_physics::table::ProcessTable::option2_electron(
        &g4_data::datadir::DataDir::new(g4_data::datadir::repo_data_dir()).unwrap(),
        g4_physics::models::ElasticRotation::Geant4,
    )
    .unwrap()
}

#[test]
fn spatial_diagnostic_preserves_transport_chemistry_rng_and_lineage() {
    use g4_core::physchem::{PhysChemEvent, PhysChemKind, PhysChemSite};
    use g4_core::units::Energy;
    use g4_physics::models::ProcessKind;
    use g4_physics::transport::{HistoryObserver, Primary, TransportConfig, transport_electron};

    #[derive(Default)]
    struct Events(Vec<PhysChemEvent>);
    impl HistoryObserver for Events {
        fn on_physchem(&mut self, event: &PhysChemEvent) {
            self.0.push(*event);
        }
    }
    struct Observed {
        events: Events,
        sites: Vec<PhysChemSite>,
        spatial: RustSpatialObserver,
    }
    impl HistoryObserver for Observed {
        fn on_physchem(&mut self, event: &PhysChemEvent) {
            self.events.on_physchem(event);
        }
        fn on_physchem_site(&mut self, site: &PhysChemSite) {
            self.sites.push(*site);
            self.spatial.on_physchem_site(site);
        }
    }
    let table = physics_table();
    let engine = g4_chemistry::PhysChemEngine::option3().unwrap();
    let streams = RngStreams::new(DIAGNOSTIC_SEEDS[0]);
    for history in 0..8 {
        let primary = Primary {
            energy: Energy::from_ev(1_000.0),
            position: Vec3::ZERO,
            direction: Vec3::new(0.0, 0.0, 1.0),
            history_id: history,
        };
        let mut plain_rng = streams.stream(history, Stage::Physics).unwrap();
        let mut observed_rng = plain_rng.clone();
        let mut events = Events::default();
        let mut observed = Observed {
            events: Events::default(),
            sites: Vec::new(),
            spatial: RustSpatialObserver::new(DIAGNOSTIC_SEEDS[0], history),
        };
        let plain = transport_electron(
            &table,
            &TransportConfig::default(),
            primary,
            &mut plain_rng,
            &mut events,
        )
        .unwrap();
        let measured = transport_electron(
            &table,
            &TransportConfig::default(),
            primary,
            &mut observed_rng,
            &mut observed,
        )
        .unwrap();
        assert_eq!(plain, measured);
        assert_eq!(events.0, observed.events.0);
        assert_eq!(format!("{plain:?}"), format!("{measured:?}"));
        for _ in 0..64 {
            assert_eq!(
                plain_rng.uniform_open01().to_bits(),
                observed_rng.uniform_open01().to_bits()
            );
        }
        assert_eq!(observed.sites.len(), events.0.len());
        assert_eq!(
            observed.sites.len() as u64,
            plain.counts.get(ProcessKind::Excitation) + plain.counts.get(ProcessKind::Ionisation)
                - plain.quirks.ionisation_noop_below_binding
                + plain.counts.get(ProcessKind::Attachment)
                + plain.solvations
        );
        let mut track_parents = BTreeMap::new();
        for (ordinal, site) in observed.sites.iter().enumerate() {
            assert_eq!(site.source_event, ordinal as u64);
            assert_eq!(site.event, events.0[ordinal]);
            assert_eq!(site.event.parent_history, history);
            if let Some(parent) = site.parent_track_id {
                assert!(parent < site.physical_track_id);
            } else {
                assert_eq!(site.physical_track_id, 0);
            }
            if let Some(previous) =
                track_parents.insert(site.physical_track_id, site.parent_track_id)
            {
                assert_eq!(previous, site.parent_track_id);
            }
            if site.event.kind != PhysChemKind::Solvation {
                assert_eq!(site.origin, site.event.position);
            }
        }
        assert_eq!(track_parents.len() as u64, plain.n_tracks);
        let mut plain_rng = streams.stream(history, Stage::PhysChem).unwrap();
        let mut observed_rng = plain_rng.clone();
        let plain = engine.convert_history(&events.0, &mut plain_rng).unwrap();
        let measured = engine
            .convert_history_with_observer(&events.0, &mut observed_rng, &mut observed.spatial)
            .unwrap();
        assert_eq!(plain, measured);
        assert_eq!(
            format!("{plain:?}"),
            format!("{measured:?}"),
            "round-trip float formatting must be bitwise unchanged"
        );
        for _ in 0..64 {
            assert_eq!(
                plain_rng.uniform_open01().to_bits(),
                observed_rng.uniform_open01().to_bits()
            );
        }
        let rows = observed.spatial.finish().unwrap();
        let live: Vec<_> = rows
            .iter()
            .filter(|r| r.boundary == Boundary::FinalSurvivors && r.moments.count == 1)
            .collect();
        assert_eq!(live.len(), plain.species.len());
        for (row, species) in live.iter().zip(&plain.species) {
            assert_eq!(row.category, Category::Species(species.species));
            assert_eq!(
                row.moments,
                SpatialMoments::from_position(species.position).unwrap()
            );
            let birth = rows
                .iter()
                .find(|r| {
                    r.identity.entity == row.identity.entity
                        && matches!(
                            r.boundary,
                            Boundary::SolvatedElectronCreation | Boundary::DissociationProducts
                        )
                })
                .unwrap();
            assert_eq!(birth.identity, row.identity);
        }
        for site in &observed.sites {
            let physical = rows
                .iter()
                .find(|r| {
                    r.boundary == Boundary::PhysicalSites
                        && r.identity.source_event == Some(site.source_event)
                })
                .unwrap();
            assert_eq!(
                physical.moments,
                SpatialMoments::from_position(site.origin).unwrap()
            );
            let boundary = if site.event.kind == PhysChemKind::Solvation {
                Boundary::SolvatedElectronCreation
            } else {
                Boundary::PreDecayWater
            };
            let molecular = rows
                .iter()
                .find(|r| {
                    r.boundary == boundary && r.identity.source_event == Some(site.source_event)
                })
                .unwrap();
            assert_eq!(
                molecular.identity.parent,
                Some(EntityId {
                    namespace: Namespace::Physical,
                    id: site.physical_track_id as i64,
                })
            );
            assert_eq!(
                molecular.moments,
                SpatialMoments::from_position(site.event.position).unwrap()
            );
        }
        let births = rows
            .iter()
            .filter(|r| {
                r.moments.count == 1
                    && r.category == Category::Species(Species::e_aq)
                    && matches!(
                        r.boundary,
                        Boundary::SolvatedElectronCreation | Boundary::DissociationProducts
                    )
            })
            .count();
        assert_eq!(
            births as u64 - plain.stats.recombinations,
            plain.stats.species_counts[Species::e_aq.index()]
        );
        DiagnosticDataset::from_records(
            rows,
            Implementation::Rust,
            &SamplePlan::new(BTreeMap::from([(DIAGNOSTIC_SEEDS[0], vec![history])])).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn spatial_diagnostic_preserves_retests_and_killed_birth_identity() {
    use g4_core::physchem::{PhysChemEvent, PhysChemKind, PhysChemSite};
    use g4_core::units::Energy;
    use g4_physics::transport::HistoryObserver;
    let events: Vec<_> = [
        PhysChemKind::Ionisation,
        PhysChemKind::Ionisation,
        PhysChemKind::Solvation,
    ]
    .into_iter()
    .map(|kind| PhysChemEvent {
        kind,
        level_or_shell: 0,
        position: Vec3::new(0.0, 0.0, 10.0),
        deposited_energy: Energy::from_ev(0.0),
        parent_history: 0,
    })
    .collect();
    let mut observer = RustSpatialObserver::new(10, 0);
    for (i, event) in events.iter().enumerate() {
        observer.on_physchem_site(&PhysChemSite {
            event: *event,
            source_event: i as u64,
            physical_track_id: 0,
            parent_track_id: None,
            origin: event.position,
        });
    }
    let engine = g4_chemistry::PhysChemEngine::option3().unwrap();
    let mut rng = RngStreams::new(10).stream(0, Stage::PhysChem).unwrap();
    let mut plain_rng = rng.clone();
    let plain = engine.convert_history(&events, &mut plain_rng).unwrap();
    let measured = engine
        .convert_history_with_observer(&events, &mut rng, &mut observer)
        .unwrap();
    assert_eq!(plain, measured);
    assert_eq!(plain.stats.recombinations, 1);
    assert_eq!(
        plain_rng.uniform_open01().to_bits(),
        rng.uniform_open01().to_bits()
    );
    let rows = observer.finish().unwrap();
    assert!(
        rows.iter()
            .any(|r| r.boundary == Boundary::DissociationProducts
                && r.identity.zero_time_pass == Some(1))
    );
    assert!(
        rows.iter()
            .any(|r| r.boundary == Boundary::SolvatedElectronCreation
                && r.identity.entity.unwrap().id == 2)
    );
    assert!(!rows.iter().any(|r| r.boundary == Boundary::FinalSurvivors
        && r.identity.entity.is_some_and(|id| id.id == 2)));
    for row in rows
        .iter()
        .filter(|r| r.boundary == Boundary::DissociationProducts && r.moments.count == 1)
    {
        assert_eq!(
            row.identity.parent,
            Some(EntityId {
                namespace: Namespace::Molecular,
                id: row.identity.source_event.unwrap() as i64,
            })
        );
        assert!(row.identity.creation_ordinal.unwrap() >= events.len() as u64);
    }
}

#[test]
fn spatial_diagnostic_normal_summaries_and_csv_are_thread_independent() {
    use g4_sim::config::RunConfig;
    use g4_sim::run::{run_physchem_summary, run_physchem_summary_with_diagnostics};
    let table = physics_table();
    let engine = g4_chemistry::PhysChemEngine::option3().unwrap();
    let mut config = RunConfig::new(1_000.0, 12);
    config.master_seed = DIAGNOSTIC_SEEDS[0];
    config.threads = Some(1);
    config.batch_size = 4;
    let baseline =
        serde_json::to_vec(&run_physchem_summary(&table, &engine, &config).unwrap()).unwrap();
    let mut first_csv = None;
    for threads in [1, 4] {
        config.threads = Some(threads);
        config.batch_size = if threads == 1 { 4 } else { 7 };
        let mut writer = CsvSpatialDiagnosticWriter::new(Vec::new()).unwrap();
        let result =
            run_physchem_summary_with_diagnostics(&table, &engine, &config, Some(&mut writer))
                .unwrap();
        assert_eq!(serde_json::to_vec(&result).unwrap(), baseline);
        assert_eq!(
            serde_json::to_vec(&run_physchem_summary(&table, &engine, &config).unwrap()).unwrap(),
            baseline
        );
        let csv = writer.into_inner();
        DiagnosticDataset::read_csv(
            Cursor::new(&csv),
            Implementation::Rust,
            &SamplePlan::uniform(&[config.master_seed], config.n_histories).unwrap(),
        )
        .unwrap();
        if let Some(expected) = &first_csv {
            assert_eq!(&csv, expected);
        } else {
            first_csv = Some(csv);
        }
    }
    config.direction = [1.0, 0.0, 0.0];
    let mut writer = CsvSpatialDiagnosticWriter::new(Vec::new()).unwrap();
    assert!(
        run_physchem_summary_with_diagnostics(&table, &engine, &config, Some(&mut writer)).is_err()
    );
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}

fn active_category(boundary: Boundary) -> Category {
    match boundary {
        Boundary::PhysicalSites | Boundary::PreDecayWater => Category::Excitation(0),
        Boundary::SolvatedElectronCreation => Category::Species(Species::e_aq),
        Boundary::DissociationProducts | Boundary::FinalSurvivors => Category::Species(Species::H),
    }
}

fn records(
    implementation: Implementation,
    plan: &SamplePlan,
    positions: impl Fn(u64, u64, Boundary, Category) -> Vec<Vec3>,
) -> Vec<DiagnosticRecord> {
    let mut rows = Vec::new();
    for (seed, histories) in plan.groups() {
        for history in histories {
            let mut ordinal = 0;
            for boundary in Boundary::ALL {
                for category in boundary.categories() {
                    let points = positions(*seed, *history, boundary, category);
                    if points.is_empty() {
                        rows.push(DiagnosticRecord::empty(
                            implementation,
                            *seed,
                            *history,
                            boundary,
                            category,
                        ));
                    }
                    for position in points {
                        let molecular = boundary != Boundary::PhysicalSites;
                        rows.push(DiagnosticRecord {
                            implementation,
                            seed: *seed,
                            history: *history,
                            boundary,
                            category,
                            identity: Identity {
                                entity: Some(EntityId {
                                    namespace: if molecular {
                                        Namespace::Molecular
                                    } else {
                                        Namespace::Event
                                    },
                                    id: ordinal as i64,
                                }),
                                physical_track_id: (!molecular).then_some(17),
                                parent: molecular.then_some(EntityId {
                                    namespace: if boundary == Boundary::DissociationProducts {
                                        Namespace::Molecular
                                    } else {
                                        Namespace::Physical
                                    },
                                    id: -42,
                                }),
                                source_event: Some(ordinal),
                                creation_ordinal: Some(ordinal),
                                zero_time_pass: (boundary == Boundary::DissociationProducts)
                                    .then_some(0),
                                processing_ordinal: (boundary == Boundary::DissociationProducts)
                                    .then_some(ordinal),
                            },
                            stage: boundary.stage(),
                            time_ps: molecular.then_some(1.0),
                            state: if boundary == Boundary::PreDecayWater {
                                LifeState::Stopped
                            } else {
                                LifeState::Alive
                            },
                            moments: SpatialMoments::from_position(position).unwrap(),
                        });
                        ordinal += 1;
                    }
                }
            }
        }
    }
    rows
}

fn constant_records(
    implementation: Implementation,
    plan: &SamplePlan,
    radii: [f64; 5],
) -> Vec<DiagnosticRecord> {
    records(implementation, plan, |_, _, b, c| {
        if c == active_category(b) {
            vec![Vec3::new(0.0, 0.0, radii[b.index()])]
        } else {
            vec![]
        }
    })
}

fn dataset(
    rows: Vec<DiagnosticRecord>,
    implementation: Implementation,
    plan: &SamplePlan,
) -> DiagnosticDataset {
    DiagnosticDataset::from_records(rows, implementation, plan).unwrap()
}

fn small_plan() -> SamplePlan {
    SamplePlan::uniform(&[10, 20], 2).unwrap()
}

#[test]
fn spatial_diagnostic_schema_round_trips_records_and_streamed_dataset() {
    let plan = small_plan();
    let mut rows = constant_records(Implementation::Rust, &plan, [2.0; 5]);
    // The sign of a Geant4 molecular track ID must survive, and absent physical time stays absent.
    rows.iter_mut()
        .find(|r| r.boundary == Boundary::SolvatedElectronCreation)
        .unwrap()
        .identity
        .entity
        .as_mut()
        .unwrap()
        .id = -7;
    for record in &rows {
        let text = record.csv_row().unwrap();
        assert_eq!(DiagnosticRecord::from_csv_row(&text).unwrap(), *record);
        assert_eq!(text.split(',').count(), csv_header().split(',').count());
    }
    let expected = dataset(rows.clone(), Implementation::Rust, &plan);
    let mut csv = Vec::new();
    write_csv(&mut csv, &rows).unwrap();
    assert_eq!(
        DiagnosticDataset::read_csv(Cursor::new(csv), Implementation::Rust, &plan).unwrap(),
        expected
    );
    assert_eq!(expected.histories().len(), 4);
    assert_eq!(
        Boundary::ALL.map(|b| b.categories().len()),
        [12, 1, 11, 7, 7]
    );
}

#[test]
fn spatial_diagnostic_stream_reader_skips_comment_lines() {
    let plan = small_plan();
    let rows = constant_records(Implementation::Rust, &plan, [2.0; 5]);
    let expected = dataset(rows.clone(), Implementation::Rust, &plan);
    let mut body = Vec::new();
    write_csv(&mut body, &rows).unwrap();
    let mut csv = b"# mode physchem_spatial_diagnostic\n# metadata before header\n".to_vec();
    csv.extend_from_slice(&body);
    csv.extend_from_slice(b"# complete: true\n");

    assert_eq!(
        DiagnosticDataset::read_csv(Cursor::new(csv), Implementation::Rust, &plan).unwrap(),
        expected
    );
}

#[test]
fn spatial_diagnostic_single_implementation_precision_is_descriptive_and_reproducible() {
    let plan = small_plan();
    let data = dataset(
        constant_records(Implementation::Rust, &plan, [2.0; 5]),
        Implementation::Rust,
        &plan,
    );
    let precision = data.pooled_rms_precision().unwrap();
    assert_eq!(precision, data.pooled_rms_precision().unwrap());
    assert_eq!(precision.pooled_rms_nm, 2.0);
    assert_eq!(precision.lower_nm, 2.0);
    assert_eq!(precision.upper_nm, 2.0);
    assert_eq!(precision.relative_half_width, 0.0);
    let empty = dataset(
        records(Implementation::Rust, &plan, |_, _, _, _| vec![]),
        Implementation::Rust,
        &plan,
    );
    assert!(empty.pooled_rms_precision().is_err());
}

#[test]
fn spatial_diagnostic_rejects_missing_category_history_seed_and_duplicate_rows() {
    let plan = small_plan();
    let rows = constant_records(Implementation::Rust, &plan, [2.0; 5]);
    let check = |rows| {
        DiagnosticDataset::from_records(rows, Implementation::Rust, &plan)
            .unwrap_err()
            .to_string()
    };
    let mut missing_category = rows.clone();
    missing_category.remove(1);
    assert!(check(missing_category).contains("missing category"));
    let missing_history = rows
        .iter()
        .filter(|r| !(r.seed == 10 && r.history == 1))
        .cloned()
        .collect::<Vec<_>>();
    assert!(check(missing_history).contains("missing seed/history"));
    let missing_seed = rows
        .iter()
        .filter(|r| r.seed != 10)
        .cloned()
        .collect::<Vec<_>>();
    assert!(check(missing_seed).contains("missing seed/history"));
    let mut duplicate = rows.clone();
    duplicate.insert(0, duplicate[0].clone());
    assert!(check(duplicate).contains("duplicate entity"));
    let mut duplicate_empty = rows.clone();
    duplicate_empty.insert(1, duplicate_empty[1].clone());
    assert!(check(duplicate_empty).contains("duplicate empty"));
    let mut duplicate_block = rows.clone();
    duplicate_block.extend(
        rows.iter()
            .filter(|r| r.seed == 10 && r.history == 0)
            .cloned(),
    );
    assert!(check(duplicate_block).contains("duplicate or noncontiguous"));
    let mut empty_and_populated = rows.clone();
    empty_and_populated.insert(
        0,
        DiagnosticRecord::empty(
            Implementation::Rust,
            10,
            0,
            Boundary::PhysicalSites,
            Category::Excitation(0),
        ),
    );
    assert!(check(empty_and_populated).contains("empty and populated"));
}

#[test]
fn spatial_diagnostic_deduplicates_entity_across_categories() {
    let plan = SamplePlan::uniform(&[10], 1).unwrap();
    let mut rows = constant_records(Implementation::Rust, &plan, [2.0; 5]);
    let mut duplicate = rows[0].clone();
    duplicate.category = Category::Excitation(1);
    rows[1] = duplicate;
    let error = DiagnosticDataset::from_records(rows, Implementation::Rust, &plan).unwrap_err();
    assert!(error.to_string().contains("duplicate entity"));
}

#[test]
fn spatial_diagnostic_rejects_bad_header_fields_populations_and_metadata() {
    let plan = small_plan();
    let rows = constant_records(Implementation::Rust, &plan, [2.0; 5]);
    assert!(
        DiagnosticDataset::read_csv(Cursor::new("wrong header\n"), Implementation::Rust, &plan)
            .is_err()
    );
    let row = &rows[0];
    let serialized = row.csv_row().unwrap();
    for malformed in [
        serialized.replacen("rust", "Rust", 1),
        serialized.replacen("excitation_0", "excitation_5", 1),
        serialized.replacen("physical_sites", "unknown", 1),
        format!("{serialized},extra"),
        serialized.replacen("rust", " rust", 1),
        serialized.replacen(",NA,", ",,", 1),
        serialized.replacen(",event,0,", ",NA,0,", 1),
    ] {
        assert!(
            DiagnosticRecord::from_csv_row(&malformed).is_err(),
            "{malformed}"
        );
    }
    let mut bad = row.clone();
    bad.moments.sum_r2_nm2 = f64::NAN;
    assert!(bad.validate().is_err());
    bad = row.clone();
    bad.moments.sum_z2_nm2 = -1.0;
    assert!(bad.validate().is_err());
    bad = row.clone();
    bad.time_ps = Some(f64::INFINITY);
    assert!(bad.validate().is_err());
    bad = row.clone();
    bad.stage = DiagnosticStage::Physicochemical;
    assert!(bad.validate().is_err());
    bad = row.clone();
    bad.category = Category::Species(Species::H2O2);
    assert!(bad.validate().is_err());
    bad = row.clone();
    bad.identity.source_event = None;
    assert!(bad.validate().is_err());
    bad = row.clone();
    bad.identity.processing_ordinal = Some(1);
    assert!(bad.validate().is_err());
    bad = rows
        .iter()
        .find(|r| r.boundary == Boundary::SolvatedElectronCreation)
        .unwrap()
        .clone();
    bad.time_ps = Some(2.0);
    assert!(bad.validate().is_err());
    bad.time_ps = Some(1.0);
    bad.state = LifeState::Killed;
    assert!(bad.validate().is_err());
    let mut wrong_impl = rows.clone();
    wrong_impl[0].implementation = Implementation::Geant4;
    assert!(DiagnosticDataset::from_records(wrong_impl, Implementation::Rust, &plan).is_err());
    let mut unexpected = rows;
    unexpected[0].seed = 999;
    assert!(DiagnosticDataset::from_records(unexpected, Implementation::Rust, &plan).is_err());
}

#[test]
fn spatial_diagnostic_radial_edges_underflow_overflow_and_quantile_bounds() {
    let edges = radial_edges_nm();
    assert_eq!(edges[0], 1.0);
    assert_eq!(edges[16], 100_000_000.0);
    assert_eq!(radial_bin(0.0).unwrap(), 0);
    assert_eq!(radial_bin(0.999).unwrap(), 0);
    for (i, edge) in edges.iter().enumerate() {
        assert_eq!(radial_bin(*edge).unwrap(), i + 1);
        assert_eq!(radial_bin(f64::from_bits(edge.to_bits() - 1)).unwrap(), i);
    }
    assert_eq!(radial_bin(1e9).unwrap(), 17);
    for radius in [f64::NAN, f64::INFINITY, -1.0] {
        assert!(radial_bin(radius).is_err());
    }
    let mut moments = SpatialMoments::default();
    for radius in [0.0, 0.5, 1.0, 1e8] {
        moments
            .merge(&SpatialMoments::from_position(Vec3::new(radius, 0.0, 0.0)).unwrap())
            .unwrap();
    }
    assert_eq!(moments.radial_bins[0], 2);
    assert_eq!(moments.radial_bins[1], 1);
    assert_eq!(moments.radial_bins[17], 1);
    assert_eq!(
        moments.radial_quantile(0.5).unwrap(),
        Some(RadialBounds {
            lower_nm: 0.0,
            upper_nm: Some(1.0)
        })
    );
    assert_eq!(
        moments.radial_quantile(1.0).unwrap(),
        Some(RadialBounds {
            lower_nm: 1e8,
            upper_nm: None
        })
    );
    assert_eq!(
        SpatialMoments::default().radial_quantile(0.9).unwrap(),
        None
    );
    assert!(moments.radial_quantile(f64::NAN).is_err());
}

#[test]
fn spatial_diagnostic_checks_moment_identities_empty_sentinel_and_overflow() {
    let good = SpatialMoments::from_position(Vec3::new(3.0, 4.0, -12.0)).unwrap();
    assert_eq!(good.sum_r2_nm2, 169.0);
    assert_eq!(good.sum_x2_plus_y2_nm2, 25.0);
    assert_eq!(good.sum_z_nm, -12.0);
    assert_eq!(good.max_radius_nm, 13.0);
    for position in [Vec3::new(f64::NAN, 0.0, 0.0), Vec3::new(1e308, 0.0, 0.0)] {
        assert!(SpatialMoments::from_position(position).is_err());
    }
    let mut bad = good;
    bad.sum_r2_nm2 = 170.0;
    assert!(bad.validate().is_err());
    bad = good;
    bad.radial_bins[0] = 2;
    assert!(bad.validate().is_err());
    bad = SpatialMoments {
        max_radius_nm: 1.0,
        ..SpatialMoments::default()
    };
    assert!(bad.validate().is_err());
    bad = good;
    bad.max_radius_nm = 0.0;
    assert!(bad.validate().is_err());
    bad = good;
    bad.sum_z_nm = 1e308;
    assert!(bad.validate().is_err());
    let mut huge = SpatialMoments {
        count: u64::MAX,
        radial_bins: [0; RADIAL_BIN_COUNT],
        ..SpatialMoments::default()
    };
    huge.radial_bins[0] = u64::MAX;
    let saved = huge;
    assert!(
        huge.merge(&SpatialMoments::from_position(Vec3::new(0.0, 0.0, 0.0)).unwrap())
            .is_err()
    );
    assert_eq!(huge, saved);
}

#[test]
fn spatial_diagnostic_long_history_accumulation_accepts_floating_point_roundoff() {
    // Decimal components generate different rounding in the independently summed
    // radial and component moments. A fixed ulp tolerance would reject valid tracks.
    let point = SpatialMoments::from_position(Vec3::new(0.1, 0.2, -0.3)).unwrap();
    let mut moments = SpatialMoments::default();
    for _ in 0..200_000 {
        moments.merge(&point).unwrap();
    }
    assert_eq!(moments.count, 200_000);
    // Conservative arithmetic bound for 200,000 sequential floating-point additions.
    assert!((moments.rms_radius_nm().unwrap() - 0.14_f64.sqrt()).abs() < 1e-10);
    let mut corrupt = moments;
    corrupt.sum_r2_nm2 *= 1.001;
    assert!(corrupt.validate().is_err());
}

#[test]
fn spatial_diagnostic_primary_is_particle_pooled_not_mean_history_or_species_rms() {
    let plan = SamplePlan::uniform(&[10], 2).unwrap();
    let r = dataset(
        records(Implementation::Rust, &plan, |_, history, b, c| {
            if b == Boundary::FinalSurvivors {
                match (history, c) {
                    (0, Category::Species(Species::e_aq)) => vec![Vec3::new(1.0, 0.0, 0.0)],
                    (1, Category::Species(Species::H)) => vec![Vec3::new(0.0, 0.0, -3.0); 9],
                    _ => vec![],
                }
            } else if c == active_category(b) {
                vec![Vec3::new(0.0, 0.0, 2.0)]
            } else {
                vec![]
            }
        }),
        Implementation::Rust,
        &plan,
    );
    let g = dataset(
        constant_records(Implementation::Geant4, &plan, [2.0; 5]),
        Implementation::Geant4,
        &plan,
    );
    let analysis = analyze(&r, &g).unwrap();
    assert_close(analysis.primary_ratio, (82.0_f64 / 10.0).sqrt() / 2.0);
    assert!((analysis.primary_ratio - (1.0 + 3.0) / 2.0 / 2.0).abs() > 0.4);
    let final_summary = &analysis.rust.boundaries[4];
    assert_eq!(final_summary.moments.count, 10);
    assert_eq!(final_summary.count_per_history, 5.0);
    assert_close(final_summary.mean_z_nm.unwrap(), -2.7);
    assert_close(
        final_summary.longitudinal_rms_nm.unwrap(),
        (81.0_f64 / 10.0).sqrt(),
    );
    assert_close(
        final_summary.transverse_rms_nm.unwrap(),
        (1.0_f64 / 10.0).sqrt(),
    );
    assert_eq!(final_summary.max_radius_nm, Some(3.0));
    assert_close(
        final_summary.radial_fractions.unwrap().into_iter().sum(),
        1.0,
    );
    assert_eq!(
        analysis.rust.histories[0].boundaries[4].rms_radius_nm(),
        Some(1.0)
    );
    assert_eq!(
        analysis.rust.histories[1].boundaries[4].rms_radius_nm(),
        Some(3.0)
    );
    let electrons =
        &analysis.rust.categories[&(Boundary::FinalSurvivors, Category::Species(Species::e_aq))];
    assert_eq!(electrons.rms_radius_nm, Some(1.0));
    assert_eq!(electrons.count_per_history, 0.5);
    let empty =
        &analysis.rust.categories[&(Boundary::FinalSurvivors, Category::Species(Species::OH))];
    assert_eq!(empty.rms_radius_nm, None);
    assert_eq!(empty.max_radius_nm, None);
    assert_eq!(empty.radial_fractions, None);
}

#[test]
fn spatial_diagnostic_percentiles_use_interpolated_sorted_rank() {
    assert_eq!(percentile(&[30.0, 0.0, 20.0, 10.0], 0.25).unwrap(), 7.5);
    assert_close(percentile(&[30.0, 0.0, 20.0, 10.0], 0.975).unwrap(), 29.25);
    assert_eq!(percentile(&[7.0], 0.025).unwrap(), 7.0);
    assert_eq!(percentile(&[3.0, 2.0], 0.0).unwrap(), 2.0);
    assert_eq!(percentile(&[3.0, 2.0], 1.0).unwrap(), 3.0);
    assert!(percentile(&[], 0.5).is_err());
    assert!(percentile(&[f64::NAN], 0.5).is_err());
    assert!(percentile(&[1.0], f64::NAN).is_err());
    assert!(percentile(&[1.0], -0.1).is_err());
}

#[test]
fn spatial_diagnostic_constant_samples_give_exact_intervals_and_first_detected_wording() {
    let plan = small_plan();
    let r = dataset(
        constant_records(Implementation::Rust, &plan, [2.0, 2.0, 4.0, 6.0, 8.0]),
        Implementation::Rust,
        &plan,
    );
    let g = dataset(
        constant_records(Implementation::Geant4, &plan, [2.0; 5]),
        Implementation::Geant4,
        &plan,
    );
    let analysis = analyze(&r, &g).unwrap();
    assert_eq!(analysis.bootstrap_replicates, 20_000);
    assert_eq!(analysis.analysis_seed, ANALYSIS_SEED);
    assert_eq!(analysis.primary_ratio, 4.0);
    assert_eq!(
        analysis.primary_interval,
        RatioInterval {
            lower: 4.0,
            upper: 4.0
        }
    );
    assert_eq!(analysis.simultaneous_log_radius, 0.0);
    assert_eq!(
        analysis.first_detected_divergence(),
        Some(Boundary::PreDecayWater)
    );
    assert_eq!(
        analysis.divergence_statement(),
        "first detected divergence: pre_decay_water; this does not establish causation or equivalence at earlier boundaries."
    );
    assert_eq!(analysis.relative_half_width, 0.0);
    for boundary in &analysis.boundaries {
        assert_close(boundary.interval.lower, boundary.ratio);
        assert_close(boundary.interval.upper, boundary.ratio);
    }
    assert!(
        !RatioInterval {
            lower: 0.8,
            upper: 1.0
        }
        .excludes_one()
    );
    assert!(
        !RatioInterval {
            lower: 1.0,
            upper: 1.2
        }
        .excludes_one()
    );
    let equal = dataset(
        constant_records(Implementation::Rust, &plan, [2.0; 5]),
        Implementation::Rust,
        &plan,
    );
    let analysis = analyze(&equal, &g).unwrap();
    assert_eq!(analysis.first_detected_divergence(), None);
    assert!(
        analysis
            .divergence_statement()
            .contains("inconclusive, not equivalence")
    );
}

fn varied_samples() -> (DiagnosticDataset, DiagnosticDataset) {
    let plan = SamplePlan::new(BTreeMap::from([
        (10, vec![0, 1]),
        (20, vec![0, 1, 2]),
        (30, vec![0]),
    ]))
    .unwrap();
    let rows = |implementation| {
        records(implementation, &plan, |seed, history, b, c| {
            if c != active_category(b) {
                return vec![];
            }
            let radius = 1.0 + seed as f64 / 10.0 + (history + 1) as f64 * (b.index() + 1) as f64;
            vec![Vec3::new(radius, 0.0, 0.0); (history + 1) as usize]
        })
    };
    (
        dataset(rows(Implementation::Rust), Implementation::Rust, &plan),
        dataset(rows(Implementation::Geant4), Implementation::Geant4, &plan),
    )
}

#[test]
fn spatial_diagnostic_bootstrap_repeats_but_does_not_pair_equal_seed_history_labels() {
    let (r, g) = varied_samples();
    let first = analyze(&r, &g).unwrap();
    assert_eq!(first, analyze(&r, &g).unwrap());
    assert_eq!(first.primary_ratio, 1.0);
    // A paired resample of these identical observations would have ratio 1 in every replicate.
    assert!(first.primary_interval.lower < 0.9 && first.primary_interval.upper > 1.1);
    let changed = analyze_with_seed(&r, &g, ANALYSIS_SEED + 1).unwrap();
    assert_ne!(first.primary_interval, changed.primary_interval);
    assert_ne!(
        first.simultaneous_log_radius,
        changed.simultaneous_log_radius
    );
}

// Independent small oracle: build integer multiplicities of (seed, history) first,
// then recompute all five pooled ratios from the retained input moments.
fn oracle_draw(dataset: &DiagnosticDataset, rng: &mut HistoryRng) -> [f64; 5] {
    let groups: Vec<_> = dataset.plan().groups().iter().collect();
    let index = |rng: &mut HistoryRng, length: usize| {
        let base = 1_u64 << 52;
        loop {
            let value = (rng.uniform_open01() * base as f64).floor() as u64;
            if value < base - base % length as u64 {
                break (value % length as u64) as usize;
            }
        }
    };
    let mut multiplicities = BTreeMap::new();
    for _ in 0..groups.len() {
        let (seed, ids) = groups[index(rng, groups.len())];
        for _ in 0..ids.len() {
            *multiplicities
                .entry((*seed, ids[index(rng, ids.len())]))
                .or_insert(0_u64) += 1;
        }
    }
    std::array::from_fn(|boundary| {
        let mut count = 0;
        let mut squared_radius = 0.0;
        for history in dataset.histories() {
            let weight = *multiplicities
                .get(&(history.seed, history.history))
                .unwrap_or(&0);
            count += history.boundaries[boundary].count * weight;
            squared_radius += history.boundaries[boundary].sum_r2_nm2 * weight as f64;
        }
        (squared_radius / count as f64).sqrt()
    })
}

#[test]
fn spatial_diagnostic_hierarchy_and_max_log_interval_match_multiplicity_oracle() {
    let (r, g) = varied_samples();
    let analysis = analyze(&r, &g).unwrap();
    let streams = RngStreams::new(ANALYSIS_SEED);
    let mut rr = streams.stream(0, Stage::Physics).unwrap();
    let mut gr = streams.stream(1, Stage::Physics).unwrap();
    let mut primary = Vec::new();
    let mut maxima = Vec::new();
    for _ in 0..20_000 {
        let rust = oracle_draw(&r, &mut rr);
        let geant4 = oracle_draw(&g, &mut gr);
        let ratios: [f64; 5] = std::array::from_fn(|i| rust[i] / geant4[i]);
        primary.push(ratios[4]);
        maxima.push(
            ratios
                .into_iter()
                .map(|ratio| ratio.ln().abs())
                .fold(0.0, f64::max),
        );
    }
    primary.sort_by(f64::total_cmp);
    maxima.sort_by(f64::total_cmp);
    let quantile = |sorted: &[f64], p: f64| {
        let rank = 19_999.0 * p;
        let index = rank as usize;
        sorted[index] + (rank - index as f64) * (sorted[index + 1] - sorted[index])
    };
    assert_close(analysis.primary_interval.lower, quantile(&primary, 0.025));
    assert_close(analysis.primary_interval.upper, quantile(&primary, 0.975));
    let q = quantile(&maxima, 0.95);
    assert_close(analysis.simultaneous_log_radius, q);
    for boundary in analysis.boundaries {
        assert_close(boundary.interval.lower, (-q).exp());
        assert_close(boundary.interval.upper, q.exp());
    }
}

#[test]
fn spatial_diagnostic_undefined_boundaries_fail_instead_of_shrinking_the_family() {
    let plan = SamplePlan::uniform(&[10], 2).unwrap();
    let g = dataset(
        constant_records(Implementation::Geant4, &plan, [2.0; 5]),
        Implementation::Geant4,
        &plan,
    );
    for boundary in Boundary::ALL {
        let mut radii = [2.0; 5];
        radii[boundary.index()] = 0.0;
        let r = dataset(
            constant_records(Implementation::Rust, &plan, radii),
            Implementation::Rust,
            &plan,
        );
        assert!(
            matches!(analyze(&r, &g), Err(DiagnosticError::UndefinedRatio { boundary: b, .. }) if b == boundary)
        );
    }
    // The observed ratio exists, but a resample selecting only the empty history does not.
    let r = dataset(
        records(Implementation::Rust, &plan, |_, history, b, c| {
            if c == active_category(b) && (b != Boundary::DissociationProducts || history == 1) {
                vec![Vec3::new(2.0, 0.0, 0.0)]
            } else {
                vec![]
            }
        }),
        Implementation::Rust,
        &plan,
    );
    let err = analyze(&r, &g).unwrap_err();
    assert!(
        matches!(&err, DiagnosticError::UndefinedRatio { boundary: Boundary::DissociationProducts, context } if context.contains("bootstrap replicate"))
    );
    assert!(err.to_string().contains("no interval was constructed"));
}

#[test]
fn spatial_diagnostic_sample_plan_and_precision_rule_are_preregistered() {
    assert_eq!(
        DIAGNOSTIC_SEEDS,
        [20261021, 20261022, 20261023, 20261024, 20261025]
    );
    assert_eq!(ANALYSIS_SEED, 20261026);
    assert_eq!(SamplePlan::production(40).unwrap().history_count(), 200);
    assert_eq!(SamplePlan::production(160).unwrap().history_count(), 800);
    assert!(SamplePlan::production(41).is_err());
    assert!(SamplePlan::uniform(&[], 2).is_err());
    assert!(SamplePlan::uniform(&[1], 0).is_err());
    assert!(SamplePlan::uniform(&[1, 1], 2).is_err());
    assert!(SamplePlan::new(BTreeMap::from([(1, vec![0, 0])])).is_err());
    let plan = small_plan();
    let r = dataset(
        constant_records(Implementation::Rust, &plan, [2.0; 5]),
        Implementation::Rust,
        &plan,
    );
    let g = dataset(
        constant_records(Implementation::Geant4, &plan, [2.0; 5]),
        Implementation::Geant4,
        &plan,
    );
    let mut analysis = analyze(&r, &g).unwrap();
    assert!(analysis.precision_decision().is_err());
    // Exercise the stopping policy without spending 200-history production bootstrap work.
    for (level, next) in [
        (40, PrecisionDecision::ExpandToHistoriesPerSeed(80)),
        (80, PrecisionDecision::ExpandToHistoriesPerSeed(160)),
        (160, PrecisionDecision::CapReached),
    ] {
        analysis.rust.plan = SamplePlan::production(level).unwrap();
        analysis.geant4.plan = SamplePlan::production(level).unwrap();
        analysis.relative_half_width = 0.04;
        assert_eq!(
            analysis.precision_decision().unwrap(),
            PrecisionDecision::TargetReached
        );
        analysis.relative_half_width = 0.04001;
        for ratio in [0.7, 1.0, 1.3] {
            analysis.primary_ratio = ratio;
            assert_eq!(analysis.precision_decision().unwrap(), next);
        }
    }
}
