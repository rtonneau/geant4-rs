//! V13-style determinism and output tests of the run layer, on the real tables.
//!
//! * Per-history results are bitwise identical for 1, 2 and 8 threads and for any batch size.
//! * A history depends only on `(seed, id)`: a sub-range of ids reproduces the same histories.
//! * The JSON output carries the provenance (seed, manifest hash, models).

use dna_core::manifest::Manifest;
use dna_data::datadir::{DataDir, repo_data_dir};
use dna_physics::models::ElasticRotation;
use dna_physics::table::ProcessTable;
use dna_sim::config::RunConfig;
use dna_sim::history::HistoryResult;
use dna_sim::run::{run, run_detailed, run_summary};

fn setup() -> Option<(ProcessTable, Manifest)> {
    let root = repo_data_dir();
    if !root.join("dna").is_dir() || !root.join("MANIFEST.toml").is_file() {
        eprintln!("SKIPPED: Geant4 data not found at {}", root.display());
        return None;
    }
    let dir = DataDir::new(&root).unwrap();
    let table = ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap();
    let manifest = Manifest::load(&root.join("MANIFEST.toml")).unwrap();
    Some((table, manifest))
}

/// Every field of a result as raw bits, so that `-0.0 != 0.0` and NaN payloads would be noticed.
fn bits(r: &HistoryResult) -> Vec<u64> {
    let o = &r.outcome;
    let l = &o.ledger;
    let v = |v: dna_core::vec3::Vec3| [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()];
    let mut out = vec![
        r.id,
        l.e_in.to_bits(),
        l.deposited_excitation.to_bits(),
        l.deposited_binding.to_bits(),
        l.secondaries.to_bits(),
        l.remaining_untracked.to_bits(),
        o.termination as u64,
        o.steps,
        o.counts.elastic,
        o.counts.excitation,
        o.counts.ionisation,
        o.n_secondaries,
        o.track_length_nm.to_bits(),
        o.first_step.map_or(0, |f| f.length_nm.to_bits()),
        o.first_step.map_or(9, |f| f.process.index() as u64),
        o.quirks.shell2_redraws,
        o.quirks.excitation_noop,
        o.quirks.ionisation_noop_below_binding,
        o.quirks.transfer_clamped_to_zero,
        o.quirks.k_shell_deexcitation_skipped,
        o.quirks.ionisation_without_secondary,
        u64::from(r.spectrum.underflow),
        u64::from(r.spectrum.overflow),
        r.spectrum.energy_sum_ev.to_bits(),
    ];
    out.extend(o.excitation_levels);
    out.extend(o.ionisation_shells);
    out.extend(v(o.start_position));
    out.extend(v(o.start_direction));
    out.extend(v(o.end_position));
    out.extend(r.spectrum.counts.iter().map(|&c| u64::from(c)));
    out
}

fn all_bits(rs: &[HistoryResult]) -> Vec<Vec<u64>> {
    rs.iter().map(bits).collect()
}

#[test]
fn per_history_results_are_bitwise_identical_for_1_2_and_8_threads() {
    let Some((table, _)) = setup() else { return };
    for (energy, n) in [(1.0e3, 300u64), (1.0e4, 60)] {
        let mut config = RunConfig::new(energy, n);
        config.master_seed = 2026;
        config.threads = Some(1);
        let reference = run_detailed(&table, &config).unwrap();
        assert_eq!(reference.len() as u64, n);
        assert!(reference.iter().enumerate().all(|(i, r)| r.id == i as u64));
        for threads in [2, 8] {
            config.threads = Some(threads);
            let parallel = run_detailed(&table, &config).unwrap();
            assert_eq!(
                all_bits(&reference),
                all_bits(&parallel),
                "E = {energy} eV, {threads} threads"
            );
        }
        // the global pool too
        config.threads = None;
        assert_eq!(
            all_bits(&reference),
            all_bits(&run_detailed(&table, &config).unwrap())
        );
    }
}

#[test]
fn scores_are_identical_for_any_thread_count_and_batch_size() {
    let Some((table, _)) = setup() else { return };
    let mut config = RunConfig::new(2.0e3, 400);
    config.master_seed = 77;
    let mut json = Vec::new();
    for (threads, batch) in [(1, 1024), (2, 7), (8, 64), (3, 1), (8, 400)] {
        config.threads = Some(threads);
        config.batch_size = batch;
        let s = run_summary(&table, &config).unwrap();
        json.push(serde_json::to_string(&s).unwrap());
    }
    assert!(json.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn a_history_depends_only_on_seed_and_id() {
    let Some((table, _)) = setup() else { return };
    let mut config = RunConfig::new(1.0e3, 40);
    config.master_seed = 5;
    let all = run_detailed(&table, &config).unwrap();
    config.first_history_id = 10;
    config.n_histories = 15;
    let part = run_detailed(&table, &config).unwrap();
    assert_eq!(all_bits(&all[10..25]), all_bits(&part));
    // another seed changes the results
    config.master_seed = 6;
    let other = run_detailed(&table, &config).unwrap();
    assert_ne!(all_bits(&part), all_bits(&other));
}

#[test]
fn summary_is_internally_consistent() {
    let Some((table, _)) = setup() else { return };
    let mut config = RunConfig::new(5.0e3, 500);
    config.master_seed = 9;
    let detailed = run_detailed(&table, &config).unwrap();
    let s = run_summary(&table, &config).unwrap();

    assert_eq!(s.n_histories, 500);
    let steps: u64 = detailed.iter().map(|r| r.outcome.steps).sum();
    assert_eq!(
        s.processes.iter().map(|p| p.interactions).sum::<u64>(),
        steps
    );
    let sp = &s.secondary_spectrum;
    assert_eq!(
        sp.counts.iter().sum::<u64>() + sp.underflow + sp.overflow,
        sp.n_secondaries
    );
    assert_eq!(
        sp.n_secondaries,
        detailed
            .iter()
            .map(|r| r.outcome.n_secondaries)
            .sum::<u64>()
    );
    assert_eq!(sp.edges_ev.len(), sp.counts.len() + 1);
    assert_eq!(
        s.first_step.first_process_counts.iter().sum::<u64>()
            + s.first_step.histories_without_steps,
        500
    );
    // mean energy balance: E_in = deposited + secondaries + remaining
    let e = &s.energy;
    let total = e.deposited_total_ev.mean + e.secondaries_ev.mean + e.remaining_untracked_ev.mean;
    assert!((total - e.e_in_ev).abs() < 1e-9 * e.e_in_ev);
    assert!(e.max_abs_ledger_residual_relative < 1e-12);
    // the mean over the secondary spectrum equals the secondary energy per secondary
    let per_history_secondary_sum: f64 =
        detailed.iter().map(|r| r.outcome.ledger.secondaries).sum();
    let mean_sec = sp.mean_energy_ev.unwrap();
    assert!((mean_sec * sp.n_secondaries as f64 - per_history_secondary_sum).abs() < 1e-6);
    // mean path per interaction x interactions = total track length
    let total_len: f64 = detailed.iter().map(|r| r.outcome.track_length_nm).sum();
    let from_processes: f64 = s
        .processes
        .iter()
        .filter_map(|p| {
            p.mean_path_per_interaction_nm
                .map(|m| m * p.interactions as f64)
        })
        .next()
        .unwrap();
    assert!((from_processes - total_len).abs() < 1e-9 * total_len);
    assert_eq!(s.terminations.interaction_limit, 0);
    assert_eq!(
        s.terminations.below_lowest_window + s.terminations.no_inelastic_channel,
        500
    );
}

#[test]
fn json_output_carries_provenance_models_and_config() {
    let Some((table, manifest)) = setup() else {
        return;
    };
    let mut config = RunConfig::new(1.0e3, 20);
    config.master_seed = 4242;
    let out = run(&table, &config, &manifest).unwrap();
    let text = out.to_json_pretty().unwrap();
    let j: serde_json::Value = serde_json::from_str(&text).unwrap();

    assert_eq!(j["schema"], "geant4-rs/dna-sim/run/v1");
    assert_eq!(j["provenance"]["master_seed"], 4242);
    assert_eq!(j["provenance"]["manifest_sha256"], manifest.hash());
    assert_eq!(j["provenance"]["geant4_reference_version"], "11.4.1");
    assert_eq!(j["provenance"]["dataset"], "G4EMLOW");
    assert_eq!(j["elastic_rotation"], "Geant4");
    assert_eq!(j["config"]["primary_energy_ev"], 1000.0);

    let models = j["models"].as_array().unwrap();
    let names: Vec<&str> = models
        .iter()
        .map(|m| m["model"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "G4DNAChampionElasticModel",
            "G4DNABornExcitationModel1",
            "G4DNABornIonisationModel1"
        ]
    );
    let windows: Vec<(f64, f64)> = models
        .iter()
        .map(|m| {
            (
                m["low_ev"].as_f64().unwrap(),
                m["high_ev"].as_f64().unwrap(),
            )
        })
        .collect();
    assert_eq!(windows, [(7.4, 1.0e6), (9.0, 1.0e6), (11.0, 1.0e6)]);
    assert!(
        models
            .iter()
            .all(|m| m["data_files"].as_array().is_some_and(|f| !f.is_empty()))
    );
    assert_eq!(j["summary"]["n_histories"], 20);
    assert!(j["summary"]["quirks"]["shell2_redraws"].is_u64());

    // the file writer round-trips
    let path = std::env::temp_dir().join(format!("g4rs-run-{}.json", std::process::id()));
    out.write_json(&path).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
    std::fs::remove_file(path).unwrap();
}
