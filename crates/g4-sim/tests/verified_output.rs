//! Output identity belongs to the captured models, never an independently supplied manifest.
use g4_core::manifest::Manifest;
use g4_core::sha256::{Sha256, hex};
use g4_data::datadir::{DataDir, VerifiedDataSet, repo_data_dir};
use g4_physics::models::ElasticRotation;
use g4_physics::table::{ProcessTable, VerifiedProcessTable};
use g4_sim::RunConfig;
use g4_sim::run::{run, run_detailed, run_unverified_for_testing};
use std::sync::OnceLock;

fn verified() -> &'static VerifiedProcessTable {
    static TABLE: OnceLock<VerifiedProcessTable> = OnceLock::new();
    TABLE.get_or_init(|| {
        let root = repo_data_dir();
        let manifest = Manifest::load(&root.join("MANIFEST.toml")).unwrap();
        let data = VerifiedDataSet::load(&DataDir::new(root).unwrap(), &manifest).unwrap();
        VerifiedProcessTable::option2_electron(&data, ElasticRotation::Geant4).unwrap()
    })
}

fn config() -> RunConfig {
    let mut config = RunConfig::new(1000.0, 8);
    config.master_seed = 20261026;
    config.threads = Some(1);
    config
}

#[test]
fn verified_and_legacy_inputs_preserve_histories_and_scores() {
    let table = verified();
    let legacy = ProcessTable::option2_electron_unverified(
        &DataDir::new(repo_data_dir()).unwrap(),
        ElasticRotation::Geant4,
    )
    .unwrap();
    let mut config = config();
    config.record_physchem = true;
    assert_eq!(
        run_detailed(table.table(), &config).unwrap(),
        run_detailed(&legacy, &config).unwrap(),
    );
    let verified = run(table, &config).unwrap();
    let synthetic = run_unverified_for_testing(&legacy, &config).unwrap();
    assert_eq!(verified.summary, synthetic.summary);
    assert_ne!(verified.schema, synthetic.schema);
    let json = serde_json::to_value(&synthetic).unwrap();
    assert_eq!(json["provenance"]["inputs"]["verification"], "unverified");
    assert_eq!(json["provenance"]["manifest_sha256"], "");
    assert_eq!(json["provenance"]["dataset"], "");
    assert_eq!(json["provenance"]["inputs"]["supplied_manifest_sha256"], "");
    assert!(json["provenance"]["inputs"].get("files").is_none());
    assert!(synthetic.provenance().chemistry.is_none());
    let path = std::env::temp_dir().join(format!("g4rs-unverified-{}.json", std::process::id()));
    synthetic.write_json(&path).unwrap();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        synthetic.to_json_pretty().unwrap()
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn relocated_and_frozen_outputs_keep_identity_but_changed_bytes_need_a_new_manifest() {
    let original = verified();
    let root = std::env::temp_dir().join(format!("g4rs-output-identity-{}", std::process::id()));
    std::fs::create_dir_all(root.join("dna")).unwrap();
    let manifest = original.identity();
    for file in manifest.files() {
        std::fs::copy(repo_data_dir().join(&file.path), root.join(&file.path)).unwrap();
    }
    let manifest_text = std::fs::read_to_string(repo_data_dir().join("MANIFEST.toml")).unwrap();
    std::fs::write(root.join("MANIFEST.toml"), &manifest_text).unwrap();
    let relocated_manifest = Manifest::load(&root.join("MANIFEST.toml")).unwrap();
    let dir = DataDir::new(&root).unwrap();
    let data = VerifiedDataSet::load(&dir, &relocated_manifest).unwrap();
    let frozen = VerifiedProcessTable::option2_electron(&data, ElasticRotation::Geant4).unwrap();
    let before = run(original, &config()).unwrap();
    let relocated = run(&frozen, &config()).unwrap();
    assert_eq!(
        before.to_json_pretty().unwrap(),
        relocated.to_json_pretty().unwrap()
    );

    // A harmless whitespace change is still different input bytes. The old identity must fail.
    let file = manifest
        .files()
        .iter()
        .find(|f| f.path == "dna/sigma_elastic_e_champion.dat")
        .unwrap();
    let mut bytes = std::fs::read(root.join(&file.path)).unwrap();
    bytes.push(b'\n');
    std::fs::write(root.join(&file.path), &bytes).unwrap();
    assert!(VerifiedDataSet::load(&dir, manifest).is_err());
    let after = run(&frozen, &config()).unwrap();
    assert_eq!(
        before.to_json_pretty().unwrap(),
        after.to_json_pretty().unwrap()
    );

    let updated_text = manifest_text
        .replace(&file.sha256, &hex(&Sha256::digest(&bytes)))
        .replace(
            &format!("bytes = {}", file.bytes),
            &format!("bytes = {}", bytes.len()),
        );
    let updated_manifest = Manifest::from_toml_str(&updated_text).unwrap();
    let updated_data = VerifiedDataSet::load(&dir, &updated_manifest).unwrap();
    let updated =
        VerifiedProcessTable::option2_electron(&updated_data, ElasticRotation::Geant4).unwrap();
    let updated_output = run(&updated, &config()).unwrap();
    assert_ne!(
        before.provenance().manifest_sha256,
        updated_output.provenance().manifest_sha256
    );
    assert_ne!(
        before.provenance().inputs,
        updated_output.provenance().inputs
    );
    assert_eq!(before.summary, updated_output.summary);
    assert!(
        !updated_output
            .to_json_pretty()
            .unwrap()
            .contains(&root.display().to_string())
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(feature = "chemistry")]
#[test]
fn coupled_output_uses_actual_engine_identity_and_optional_source_claim() {
    use g4_chemistry::{PhysChemEngine, option3_data_sha256};
    use g4_sim::run::{run_unverified_with_physchem_for_testing, run_with_physchem};

    let default = PhysChemEngine::option3().unwrap();
    let custom = PhysChemEngine::new(
        default.species_table().clone(),
        default.channel_table().clone(),
        default.params().clone(),
    )
    .unwrap();
    let changed = custom.clone().with_recombination_enabled(false);
    let embedded_changed = default.clone().with_recombination_enabled(false);
    let default_output = run_with_physchem(verified(), &default, &config()).unwrap();
    let mut hashes = Vec::new();
    for engine in [&default, &custom, &changed, &embedded_changed] {
        let output = run_with_physchem(verified(), engine, &config()).unwrap();
        let identity = output.provenance().chemistry.as_ref().unwrap();
        assert_eq!(identity.semantic_sha256, engine.semantic_sha256());
        assert_eq!(
            identity.source_file_sha256.as_deref(),
            engine.source_file_sha256()
        );
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["schema"], "geant4-rs/dna-sim/run/v4");
        assert_eq!(json["provenance"]["inputs"]["verification"], "verified");
        assert!(json.get("chemistry_data_sha256").is_none());
        if engine.source_file_sha256().is_some() {
            assert_eq!(
                identity.source_file_sha256.as_deref(),
                Some(option3_data_sha256().as_str())
            );
        }
        hashes.push(identity.semantic_sha256.clone());
    }
    assert_eq!(hashes[0], hashes[1]);
    assert_ne!(hashes[0], hashes[2]);
    assert_eq!(hashes[2], hashes[3]);
    let custom_output = run_with_physchem(verified(), &custom, &config()).unwrap();
    assert_eq!(default_output.summary, custom_output.summary);
    assert_eq!(default_output.physchem, custom_output.physchem);
    let unverified =
        run_unverified_with_physchem_for_testing(verified().table(), &changed, &config()).unwrap();
    let json = serde_json::to_value(&unverified).unwrap();
    assert_eq!(json["schema"], "geant4-rs/dna-sim/run-unverified/v1");
    assert_eq!(json["provenance"]["inputs"]["verification"], "unverified");
    assert_eq!(json["provenance"]["manifest_sha256"], "");
    assert_eq!(
        json["provenance"]["chemistry"]["semantic_sha256"],
        changed.semantic_sha256()
    );
    assert!(json["provenance"]["chemistry"]["source_file_sha256"].is_null());
    let path = std::env::temp_dir().join(format!("g4rs-coupled-{}.json", std::process::id()));
    custom_output.write_json(&path).unwrap();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        custom_output.to_json_pretty().unwrap()
    );
    std::fs::remove_file(path).unwrap();
}
