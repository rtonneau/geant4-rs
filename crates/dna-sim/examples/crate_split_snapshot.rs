//! Small, exact same-toolchain oracle for crate-ownership changes; no physics changes.
use dna_chemistry::PhysChemEngine;
use dna_chemistry::{SPECIES_CSV_HEADER, Species};
use dna_core::manifest::Manifest;
use dna_core::rng::{RngStreams, Stage};
use dna_core::vec3::Vec3;
use dna_data::datadir::{DataDir, repo_data_dir};
use dna_physics::models::{ElasticRotation, ProcessKind};
use dna_physics::table::ProcessTable;
use dna_physics::transport::SecondaryRecorder;
use dna_sim::config::RunConfig;
use dna_sim::history::{
    HistoryResult, radial_edges_nm, run_history_with_observer, spectrum_edges_ev,
};
use dna_sim::run::{CsvPhysChemWriter, run_detailed, run_with_physchem, run_with_sink};
use serde_json::{Value, json};

fn bits(x: f64) -> String {
    format!("0x{:016x}", x.to_bits())
}

fn vector(v: Vec3) -> [String; 3] {
    [bits(v.x), bits(v.y), bits(v.z)]
}

/// `to_value` retains f64 values in memory: convert before any decimal JSON round trip.
fn exact(value: Value) -> Value {
    match value {
        Value::Number(ref n) if n.is_f64() => json!(bits(n.as_f64().unwrap())),
        Value::Array(a) => Value::Array(a.into_iter().map(exact).collect()),
        Value::Object(o) => Value::Object(o.into_iter().map(|(k, v)| (k, exact(v))).collect()),
        other => other,
    }
}

fn physical(r: &HistoryResult) -> Value {
    let o = &r.outcome;
    let l = &o.ledger;
    exact(json!({
        "id": r.id,
        "ledger": {
            "e_in": l.e_in, "deposited_excitation": l.deposited_excitation,
            "deposited_vibrational": l.deposited_vibrational, "deposited_binding": l.deposited_binding,
            "deposited_attachment": l.deposited_attachment, "deposited_solvation": l.deposited_solvation,
            "escaped": l.escaped, "untracked_secondaries": l.untracked_secondaries,
            "remaining_untracked": l.remaining_untracked
        },
        "termination": format!("{:?}", o.termination), "steps": o.steps,
        "process_counts": ProcessKind::ALL.iter().map(|&p| o.counts.get(p)).collect::<Vec<_>>(),
        "solvations": o.solvations, "excitation_levels": o.excitation_levels,
        "ionisation_shells": o.ionisation_shells, "vibrational_levels": o.vibrational_levels,
        "n_secondaries": o.n_secondaries, "n_tracks": o.n_tracks,
        "track_length_nm": o.track_length_nm, "total_track_length_nm": o.total_track_length_nm,
        "start_position": vector(o.start_position), "start_direction": vector(o.start_direction),
        "end_position": vector(o.end_position),
        "first_step": o.first_step.map(|s| json!({"length_nm": bits(s.length_nm), "process": s.process.index()})),
        "quirks": {
            "shell2_redraws": o.quirks.shell2_redraws, "excitation_noop": o.quirks.excitation_noop,
            "ionisation_noop_below_binding": o.quirks.ionisation_noop_below_binding,
            "transfer_clamped_to_zero": o.quirks.transfer_clamped_to_zero,
            "k_shell_deexcitation_skipped": o.quirks.k_shell_deexcitation_skipped,
            "ionisation_without_secondary": o.quirks.ionisation_without_secondary,
            "vibrational_noop": o.quirks.vibrational_noop, "solvation_zero_rmean": o.quirks.solvation_zero_rmean
        },
        "spectra": r.spectra.iter().map(|s| json!({"counts": s.counts,
            "underflow": s.underflow, "overflow": s.overflow, "energy_sum_ev": bits(s.energy_sum_ev)})).collect::<Vec<_>>(),
        "radial": {"energy_ev": r.radial.energy_ev, "underflow_ev": r.radial.underflow_ev, "overflow_ev": r.radial.overflow_ev},
        "solvation": {"n": r.solvation.n, "sum_displacement_nm": r.solvation.sum_displacement_nm,
            "sum_energy_ev": r.solvation.sum_energy_ev},
        "physchem_events": r.physchem.iter().map(|e| json!({"kind": e.kind.name(),
            "level_or_shell": e.level_or_shell, "position": vector(e.position),
            "deposited_energy_ev": bits(e.deposited_energy.ev()), "parent_history": e.parent_history})).collect::<Vec<_>>()
    }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let coupled = match args.as_slice() {
        [arg] if arg == "--physics" => false,
        [arg] if arg == "--physchem" => true,
        _ => return Err("usage: crate_split_snapshot --physics|--physchem".into()),
    };
    let root = repo_data_dir();
    let manifest = Manifest::load(&root.join("MANIFEST.toml"))?;
    let dir = DataDir::new(root)?;
    dir.verify(&manifest)?;
    let table = ProcessTable::option2_electron(&dir, ElasticRotation::Geant4)?;
    let engine = coupled.then(PhysChemEngine::option3).transpose()?;
    let streams = RngStreams::new(20261026);
    let mut runs = Vec::new();
    for energy in [100.0, 1_000.0, 10_000.0] {
        let mut config = RunConfig::new(energy, 8);
        config.master_seed = 20261026;
        config.first_history_id = 3;
        config.threads = Some(1);
        config.batch_size = 7;
        config.record_physchem = true;
        let histories = run_detailed(&table, &config)?;
        let mut records = Vec::new();
        for h in histories {
            let mut record = physical(&h);
            let mut secondaries = SecondaryRecorder::default();
            let observed = run_history_with_observer(
                &table,
                &config,
                &streams,
                h.id,
                &spectrum_edges_ev(),
                &radial_edges_nm(),
                &mut secondaries,
            )?;
            assert_eq!(h, observed, "secondary observer must not change a history");
            record["secondaries"] = json!(
                secondaries
                    .secondaries
                    .iter()
                    .map(|s| json!({
                        "energy_ev": bits(s.energy.ev()), "position": vector(s.position),
                        "direction": vector(s.direction), "generation": s.generation
                    }))
                    .collect::<Vec<_>>()
            );
            if let Some(engine) = &engine {
                let mut rng = streams.stream(h.id, Stage::PhysChem)?;
                let c = engine.convert_history(&h.physchem, &mut rng)?;
                record["species"] = json!(
                    c.species
                        .iter()
                        .map(|s| json!({
                            "index": s.species.index(), "name": s.species.chemistry_name(),
                            "position": vector(s.position), "time_ps": bits(s.time.ps())
                        }))
                        .collect::<Vec<_>>()
                );
                record["species_csv_rows"] =
                    json!(c.species.iter().map(|s| s.csv_row()).collect::<Vec<_>>());
                let s = c.stats;
                record["statistics"] = exact(json!({
                    "events": s.events, "channel_counts": s.channel_counts,
                    "holes_with_candidate": s.holes_with_candidate, "recombinations": s.recombinations,
                    "decay_energy_ev": s.decay_energy_ev, "species_counts": s.species_counts,
                    "species_produced_by_event": s.species_produced_by_event,
                    "displacement_count": s.displacement_count, "displacement_sum_sq_nm2": s.displacement_sum_sq_nm2,
                    "pair_count": s.pair_count, "pair_sum_nm": s.pair_sum_nm, "pair_sum_sq_nm2": s.pair_sum_sq_nm2
                }));
            }
            records.push(record);
        }
        let mut csv = CsvPhysChemWriter::new(Vec::new(), "snapshot memory")?;
        let output = run_with_sink(&table, &config, &manifest, Some(&mut csv))?;
        let mut result = json!({"config": exact(serde_json::to_value(&config)?),
            "histories": records, "summary_json": exact(serde_json::to_value(output)?),
            "physchem_csv": String::from_utf8(csv.into_inner())?});
        if let Some(engine) = &engine {
            result["coupled_summary_json"] = exact(serde_json::to_value(run_with_physchem(
                &table, engine, &config, &manifest,
            )?)?);
            result["species_csv_header"] = json!(SPECIES_CSV_HEADER);
            result["species_order"] = json!(
                Species::ALL
                    .iter()
                    .map(|s| s.chemistry_name())
                    .collect::<Vec<_>>()
            );
        }
        runs.push(result);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "geant4-rs/crate-split-snapshot/v1",
            "profile": if coupled { "physchem" } else { "physics" },
            "spectrum_edges_ev": spectrum_edges_ev().into_iter().map(bits).collect::<Vec<_>>(),
            "radial_edges_nm": radial_edges_nm().into_iter().map(bits).collect::<Vec<_>>(),
            "runs": runs
        }))?
    );
    Ok(())
}
