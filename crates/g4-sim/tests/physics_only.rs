//! Required physical API and raw handoff coverage in both feature profiles.
use g4_core::manifest::Manifest;
use g4_data::datadir::{DataDir, VerifiedDataSet, repo_data_dir};
use g4_physics::models::ElasticRotation;
use g4_physics::physchem::PHYSCHEM_CSV_HEADER;
use g4_physics::table::VerifiedProcessTable;
use g4_sim::RunConfig;
use g4_sim::run::{CsvPhysChemWriter, run, run_detailed, run_with_sink};

#[test]
fn eight_ordered_physical_histories_preserve_ledgers_and_raw_handoff() {
    let root = repo_data_dir();
    let manifest = Manifest::load(&root.join("MANIFEST.toml")).unwrap();
    let dir = DataDir::new(root).unwrap();
    let data = VerifiedDataSet::load(&dir, &manifest).unwrap();
    let table = VerifiedProcessTable::option2_electron(&data, ElasticRotation::Geant4).unwrap();
    let mut config = RunConfig::new(1000.0, 8);
    config.master_seed = 20261026;
    config.record_physchem = true;
    config.threads = Some(1);
    config.batch_size = 7;
    let histories = run_detailed(table.table(), &config).unwrap();
    assert_eq!(histories.len(), 8);
    for (id, history) in histories.iter().enumerate() {
        assert_eq!(history.id, id as u64);
        let ledger = &history.outcome.ledger;
        assert!(ledger.deposited().is_finite());
        assert!(ledger.residual_relative().is_finite());
        assert!(ledger.residual_relative().abs() < 1e-12);
        assert!((ledger.deposited() - 1000.0).abs() < 1e-6);
        assert_eq!(ledger.untracked_secondaries, 0.0);
        assert_eq!(ledger.remaining_untracked, 0.0);
        assert!(!history.physchem.is_empty());
        assert!(
            history
                .physchem
                .iter()
                .all(|e| e.parent_history == history.id)
        );
    }
    let expected_header = "parent_history,kind,level_or_shell,x_nm,y_nm,z_nm,deposited_energy_eV";
    assert_eq!(PHYSCHEM_CSV_HEADER, expected_header);
    let mut csv = CsvPhysChemWriter::new(Vec::new(), "test memory").unwrap();
    let output = run_with_sink(&table, &config, Some(&mut csv)).unwrap();
    assert_eq!(
        csv.rows() as usize,
        histories.iter().map(|h| h.physchem.len()).sum::<usize>()
    );
    let expected_rows = histories
        .iter()
        .flat_map(|h| h.physchem.iter().map(|e| e.csv_row()));
    let expected = std::iter::once(expected_header.to_owned())
        .chain(expected_rows)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(String::from_utf8(csv.into_inner()).unwrap(), expected);
    assert_eq!(output.schema, "geant4-rs/dna-sim/run/v4");
    assert_eq!(output.summary.n_histories, 8);
    let plain = run(&table, &config).unwrap();
    assert_eq!(
        serde_json::to_value(plain.summary).unwrap(),
        serde_json::to_value(output.summary).unwrap()
    );
}
