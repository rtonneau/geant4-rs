//! V13-style determinism and output tests of the run layer, on the real tables.
//!
//! * Per-history results are bitwise identical for 1, 2 and 8 threads and for any batch size.
//! * A history depends only on `(seed, id)`: a sub-range of ids reproduces the same histories.
//! * The JSON output carries the provenance (seed, manifest hash, models).

use g4_core::manifest::Manifest;
use g4_data::datadir::{DataDir, repo_data_dir};
use g4_physics::models::{ElasticRotation, ProcessKind};
use g4_physics::table::ProcessTable;
use g4_sim::config::RunConfig;
use g4_sim::history::HistoryResult;
use g4_sim::run::{CsvPhysChemWriter, run, run_detailed, run_summary, run_summary_with_sink};

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
    let v = |v: g4_core::vec3::Vec3| [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()];
    let mut out = vec![
        r.id,
        l.e_in.to_bits(),
        l.deposited_excitation.to_bits(),
        l.deposited_vibrational.to_bits(),
        l.deposited_binding.to_bits(),
        l.deposited_attachment.to_bits(),
        l.deposited_solvation.to_bits(),
        l.escaped.to_bits(),
        l.untracked_secondaries.to_bits(),
        l.remaining_untracked.to_bits(),
        o.termination as u64,
        o.steps,
        o.solvations,
        o.n_secondaries,
        o.n_tracks,
        o.track_length_nm.to_bits(),
        o.total_track_length_nm.to_bits(),
        o.first_step.map_or(0, |f| f.length_nm.to_bits()),
        o.first_step.map_or(9, |f| f.process.index() as u64),
        o.quirks.shell2_redraws,
        o.quirks.excitation_noop,
        o.quirks.ionisation_noop_below_binding,
        o.quirks.transfer_clamped_to_zero,
        o.quirks.k_shell_deexcitation_skipped,
        o.quirks.ionisation_without_secondary,
        o.quirks.vibrational_noop,
        o.quirks.solvation_zero_rmean,
        r.solvation.n.into(),
        r.solvation.sum_displacement_nm.to_bits(),
        r.solvation.sum_energy_ev.to_bits(),
        r.radial.underflow_ev.to_bits(),
        r.radial.overflow_ev.to_bits(),
    ];
    out.extend(ProcessKind::ALL.iter().map(|&p| o.counts.get(p)));
    out.extend(o.excitation_levels);
    out.extend(o.ionisation_shells);
    out.extend(o.vibrational_levels);
    out.extend(v(o.start_position));
    out.extend(v(o.start_direction));
    out.extend(v(o.end_position));
    for sp in &r.spectra {
        out.push(u64::from(sp.underflow));
        out.push(u64::from(sp.overflow));
        out.push(sp.energy_sum_ev.to_bits());
        out.extend(sp.counts.iter().map(|&c| u64::from(c)));
    }
    out.extend(r.radial.energy_ev.iter().map(|x| x.to_bits()));
    for e in &r.physchem {
        out.push(e.kind as u64);
        out.push(u64::from(e.level_or_shell));
        out.push(e.parent_history);
        out.push(e.deposited_energy.ev().to_bits());
        out.extend(v(e.position));
    }
    out
}

fn all_bits(rs: &[HistoryResult]) -> Vec<Vec<u64>> {
    rs.iter().map(bits).collect()
}

#[test]
fn per_history_results_are_bitwise_identical_for_1_2_and_8_threads() {
    let Some((table, _)) = setup() else { return };
    for (energy, n) in [(100.0, 300u64), (1.0e3, 300), (1.0e4, 60), (1.0e5, 6)] {
        let mut config = RunConfig::new(energy, n);
        config.master_seed = 2026;
        config.record_physchem = true;
        config.threads = Some(1);
        let reference = run_detailed(&table, &config).unwrap();
        assert_eq!(reference.len() as u64, n);
        assert!(reference.iter().enumerate().all(|(i, r)| r.id == i as u64));
        assert!(reference.iter().all(|r| !r.physchem.is_empty()));
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
    let mut config = RunConfig::new(5.0e3, 300);
    config.master_seed = 9;
    let detailed = run_detailed(&table, &config).unwrap();
    let s = run_summary(&table, &config).unwrap();

    assert_eq!(s.n_histories, 300);
    let steps: u64 = detailed.iter().map(|r| r.outcome.steps).sum();
    assert_eq!(
        s.processes.iter().map(|p| p.interactions).sum::<u64>(),
        steps
    );
    assert_eq!(s.processes.len(), ProcessKind::COUNT);
    let sp = &s.secondary_spectra.all;
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
    // the generations partition the secondaries; generation 1 are those of the primary
    let by_gen = &s.secondary_spectra.by_generation;
    assert_eq!(by_gen.len(), 4);
    assert_eq!(
        by_gen.iter().map(|g| g.n_secondaries).sum::<u64>(),
        sp.n_secondaries
    );
    assert!(by_gen[0].n_secondaries > 0 && by_gen[1].n_secondaries > 0);
    assert_eq!(sp.counts.len() + 1, s.secondary_spectra.edges_ev.len());
    assert_eq!(
        s.first_step.first_process_counts.iter().sum::<u64>()
            + s.first_step.histories_without_steps,
        300
    );
    // everything is deposited: E_in = deposited (five categories), nothing else
    let e = &s.energy;
    let categories = e.deposited_excitation_ev.mean
        + e.deposited_vibrational_ev.mean
        + e.deposited_binding_ev.mean
        + e.deposited_attachment_ev.mean
        + e.deposited_solvation_ev.mean;
    assert!((categories - e.deposited_total_ev.mean).abs() < 1e-9);
    assert!((e.deposited_total_ev.mean - e.e_in_ev).abs() < 1e-9 * e.e_in_ev);
    assert_eq!(e.untracked_secondaries_ev.mean, 0.0);
    assert_eq!(e.remaining_untracked_ev.mean, 0.0);
    assert!(e.max_abs_ledger_residual_relative < 1e-12);
    // the mean over the secondary spectrum equals the secondary energy per secondary
    let sum_sec: f64 = detailed
        .iter()
        .flat_map(|r| r.spectra.iter())
        .map(|sp| sp.energy_sum_ev)
        .sum();
    let mean_sec = sp.mean_energy_ev.unwrap();
    assert!((mean_sec * sp.n_secondaries as f64 - sum_sec).abs() < 1e-6 * sum_sec);
    // mean path per interaction x interactions = total path of all tracks
    let total_len: f64 = detailed
        .iter()
        .map(|r| r.outcome.total_track_length_nm)
        .sum();
    let from_processes = s
        .processes
        .iter()
        .find_map(|p| {
            p.mean_path_per_interaction_nm
                .map(|m| m * p.interactions as f64)
        })
        .unwrap();
    assert!((from_processes - total_len).abs() < 1e-9 * total_len);
    assert_eq!(s.terminations.completed, 300);
    assert_eq!(
        s.terminations.interaction_limit + s.terminations.tracking_cut,
        0
    );
    // solvations: every secondary and the primary end in a solvation or an attachment
    let attachments = s.processes[ProcessKind::Attachment.index()].interactions;
    assert_eq!(
        s.solvation.events + attachments,
        detailed.iter().map(|r| r.outcome.n_tracks).sum::<u64>()
    );
    // radial deposit profile integrates to E_in per history
    let r = &s.radial_deposit;
    let radial_total: f64 =
        r.mean_energy_ev.iter().sum::<f64>() + r.mean_underflow_ev + r.mean_overflow_ev;
    assert!(
        (radial_total - e.e_in_ev).abs() < 1e-9 * e.e_in_ev,
        "{radial_total}"
    );
    assert_eq!(r.edges_nm.len(), r.mean_energy_ev.len() + 1);
    // W-like quantity: E_in / N_ion, with N_ion from the shell counts
    let n_ion: u64 = s.ionisation_shells.iter().sum();
    let w = s.w_value.w_ev_ratio_of_totals.unwrap();
    assert!((w.ratio - 300.0 * 5.0e3 / n_ion as f64).abs() < 1e-9 * w.ratio);
    assert!(w.std_err > 0.0);
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

    assert_eq!(j["schema"], "geant4-rs/dna-sim/run/v2");
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
            "G4DNABornIonisationModel1",
            "G4DNASancheExcitationModel",
            "G4DNAMeltonAttachmentModel",
            "G4DNAOneStepThermalizationModel<Meesungnoen2002>",
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
    assert_eq!(
        windows,
        [
            (7.4, 1.0e6),
            (9.0, 1.0e6),
            (11.0, 1.0e6),
            (2.0, 100.0),
            (4.0, 13.0),
            (0.0, 7.4)
        ]
    );
    // every cross-section model lists its data files; solvation uses none (polynomial)
    assert!(
        models[..5]
            .iter()
            .all(|m| m["data_files"].as_array().is_some_and(|f| !f.is_empty()))
    );
    assert!(models[5]["process"].is_null());
    assert_eq!(j["summary"]["n_histories"], 20);
    assert!(j["summary"]["w_value"]["w_ev_ratio_of_totals"]["ratio"].is_f64());
    assert!(j["summary"]["solvation"]["events"].is_u64());
    assert!(j["summary"]["quirks"]["shell2_redraws"].is_u64());

    // the file writer round-trips
    let path = std::env::temp_dir().join(format!("g4rs-run-{}.json", std::process::id()));
    out.write_json(&path).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
    std::fs::remove_file(path).unwrap();
}

/// The physicochemical events reach the CSV sink in history order, identically for any thread
/// count, and their numbers agree with the interaction counts.
#[test]
fn physchem_csv_is_thread_count_independent_and_consistent_with_the_counts() {
    let Some((table, _)) = setup() else { return };
    let mut texts = Vec::new();
    let mut summaries = Vec::new();
    for threads in [1, 2, 8] {
        let mut config = RunConfig::new(500.0, 200);
        config.master_seed = 31;
        config.threads = Some(threads);
        config.batch_size = 17;
        let mut sink = CsvPhysChemWriter::new(Vec::new(), "memory").unwrap();
        let s = run_summary_with_sink(&table, &config, Some(&mut sink)).unwrap();
        let rows = sink.rows();
        texts.push((String::from_utf8(sink.into_inner()).unwrap(), rows));
        summaries.push(s);
    }
    assert!(texts.windows(2).all(|w| w[0] == w[1]));
    assert!(summaries.windows(2).all(|w| w[0] == w[1]));
    let (text, rows) = &texts[0];
    let s = &summaries[0];
    let mut lines = text.lines();
    assert_eq!(
        lines.next().unwrap(),
        g4_physics::physchem::PHYSCHEM_CSV_HEADER
    );
    let mut by_kind = std::collections::BTreeMap::<String, u64>::new();
    let mut last_history = 0u64;
    for l in lines {
        let f: Vec<&str> = l.split(',').collect();
        assert_eq!(f.len(), 7, "{l}");
        let h: u64 = f[0].parse().unwrap();
        assert!(
            h >= last_history && h < 200,
            "history order: {h} after {last_history}"
        );
        last_history = h;
        let deposit: f64 = f[6].parse().unwrap();
        assert!(deposit >= 0.0 && deposit.is_finite());
        *by_kind.entry(f[1].to_owned()).or_default() += 1;
    }
    let count = |p: ProcessKind| s.processes[p.index()].interactions;
    assert_eq!(by_kind["excitation"], count(ProcessKind::Excitation));
    // ionisation events: all ionisations except the (unreachable) no-op below the binding energy
    assert_eq!(
        by_kind["ionisation"],
        count(ProcessKind::Ionisation) - s.quirks.ionisation_noop_below_binding
    );
    assert_eq!(by_kind["attachment"], count(ProcessKind::Attachment));
    assert_eq!(by_kind["solvation"], s.solvation.events);
    assert_eq!(by_kind.values().sum::<u64>(), *rows);
}
