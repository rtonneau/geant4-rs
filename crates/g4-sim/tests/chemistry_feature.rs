//! Default-feature public chemistry exports and coupled 1 ps boundary.
use g4_chemistry::{PhysChemEngine, SPECIES_CSV_HEADER, Species, SpeciesInstance};
use g4_core::rng::{RngStreams, Stage};
use g4_data::datadir::{DataDir, repo_data_dir};
use g4_physics::models::ElasticRotation;
use g4_physics::table::ProcessTable;
use g4_sim::RunConfig;
use g4_sim::run::{run_detailed, run_physchem_summary};

#[test]
fn coupled_summary_preserves_species_order_and_one_ps_output() {
    let names = [
        "H3Op", "°OH", "OHm", "e_aq", "H", "H2", "H2O2", "HO2°", "HO2m", "Oxy", "Om", "O2", "O2m",
        "O3", "O3m",
    ];
    assert_eq!(Species::ALL.map(|s| s.chemistry_name()), names);
    assert_eq!(SPECIES_CSV_HEADER, "species,x_nm,y_nm,z_nm,time_ps");
    let dir = DataDir::new(repo_data_dir()).unwrap();
    let table = ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap();
    let engine = PhysChemEngine::option3().unwrap();
    let mut config = RunConfig::new(1000.0, 8);
    config.master_seed = 20261026;
    config.threads = Some(1);
    config.batch_size = 7;
    config.record_physchem = true;
    let (physical, chemical) = run_physchem_summary(&table, &engine, &config).unwrap();
    assert_eq!(physical.n_histories, 8);
    assert_eq!(chemical.n_histories, 8);
    assert_eq!(chemical.time_ps, 1.0);
    assert_eq!(
        chemical
            .species
            .iter()
            .map(|s| s.species)
            .collect::<Vec<_>>(),
        names
    );
    let streams = RngStreams::new(config.master_seed);
    let mut counts = [0u64; 15];
    for history in run_detailed(&table, &config).unwrap() {
        let mut rng = streams.stream(history.id, Stage::PhysChem).unwrap();
        let converted = engine.convert_history(&history.physchem, &mut rng).unwrap();
        let species: &[SpeciesInstance] = &converted.species;
        assert!(!species.is_empty());
        for instance in species {
            assert_eq!(instance.time.ps(), 1.0);
            counts[instance.species.index()] += 1;
        }
    }
    assert_eq!(
        chemical
            .species
            .iter()
            .map(|s| s.alive_total)
            .collect::<Vec<_>>(),
        counts
    );
}
