//! Coupled physical and physicochemical orchestration.

use super::*;
use crate::history::run_history_with_observer;
use crate::physchem_scoring::{PhysChemScorer, PhysChemSummary};
use crate::spatial_diagnostic::{RustSpatialObserver, SpatialDiagnosticSink};
use g4_chemistry::{PhysChemEngine, option3_data_sha256};
use g4_core::rng::Stage;

/// Run the physical and physicochemical stages and score both in fixed history order.
///
/// This is the M6 stage boundary: physics produces events, `engine` consumes them on the
/// independent `(seed, history, PhysChem)` stream, and the returned species are only scored.
/// All returned species are at global time 1 ps, and radius is measured from the primary's
/// start at zero. No diffusion or two-body chemical reactions are performed; electron-hole
/// recombination is part of the conversion. Geant4's existing `physchem` snapshot additionally
/// includes zero-time option3 reactions (docs/08), so equal time alone does not align the stages.
pub fn run_physchem_summary(
    table: &ProcessTable,
    engine: &PhysChemEngine,
    config: &RunConfig,
) -> Result<(RunSummary, PhysChemSummary), SimError> {
    run_physchem_summary_with_diagnostics(table, engine, config, None)
}

/// Optional per-history spatial records, emitted in history order. The normal
/// summaries use exactly the same scored history and conversion as without a sink.
/// Diagnostic output is bounded by the batch size and does not alter run JSON.
pub fn run_physchem_summary_with_diagnostics(
    table: &ProcessTable,
    engine: &PhysChemEngine,
    config: &RunConfig,
    mut sink: Option<&mut dyn SpatialDiagnosticSink>,
) -> Result<(RunSummary, PhysChemSummary), SimError> {
    config.validate()?;
    if sink.is_some()
        && (config.direction[0] != 0.0
            || config.direction[1] != 0.0
            || !config.direction[2].is_finite()
            || config.direction[2] <= 0.0)
    {
        return Err(SimError::InvalidConfig(
            "spatial diagnostics require the primary direction along +z".into(),
        ));
    }
    let mut config = config.clone();
    config.record_physchem = true;
    let config = &config;
    let (first, end) = ids(config)?;
    let streams = RngStreams::new(config.master_seed);
    let edges = Edges::new();
    let mut physics = Scorer::new(config.primary_energy_ev);
    let mut physchem = PhysChemScorer::default();
    with_pool(config.threads, || -> Result<(), SimError> {
        let mut start = first;
        while start < end {
            let stop = start.saturating_add(config.batch_size as u64).min(end);
            let batch: Vec<_> = if sink.is_some() {
                (0..(stop - start) as usize)
                    .into_par_iter()
                    .with_min_len(4)
                    .map(|k| {
                        let id = start + k as u64;
                        let mut observer = RustSpatialObserver::new(config.master_seed, id);
                        let result = run_history_with_observer(
                            table,
                            config,
                            &streams,
                            id,
                            &edges.spectrum,
                            &edges.radial,
                            &mut observer,
                        )?;
                        Ok((result, Some(observer)))
                    })
                    .collect::<Result<_, SimError>>()?
            } else {
                run_batch(table, config, &streams, &edges, start, stop)?
                    .into_iter()
                    .map(|result| (result, None))
                    .collect()
            };
            for (result, mut observer) in batch {
                let deposited = result.outcome.ledger.deposited();
                let mut rng = streams.stream(result.id, Stage::PhysChem)?;
                let chemistry = match observer.as_mut() {
                    Some(observer) => engine.convert_history_with_observer(
                        &result.physchem,
                        &mut rng,
                        observer,
                    )?,
                    None => engine.convert_history(&result.physchem, &mut rng)?,
                };
                physics.add(&result);
                physchem.add(&chemistry, deposited);
                if let (Some(sink), Some(observer)) = (sink.as_deref_mut(), observer) {
                    sink.write_history(&observer.finish()?)?;
                }
            }
            start = stop;
        }
        Ok(())
    })??;
    Ok((physics.finish(), physchem.finish()))
}

/// Machine-readable two-stage M6 result.
#[derive(Debug, Clone, Serialize)]
pub struct PhysChemRunOutput {
    pub schema: &'static str,
    pub provenance: Provenance,
    /// SHA-256 over the embedded option3 species, channel and parameter TOML files.
    pub chemistry_data_sha256: String,
    pub models: Vec<ModelInfo>,
    pub elastic_rotation: String,
    pub config: RunConfig,
    pub summary: RunSummary,
    pub physchem: PhysChemSummary,
}

impl PhysChemRunOutput {
    pub fn to_json_pretty(&self) -> Result<String, SimError> {
        serde_json::to_string_pretty(self).map_err(|e| SimError::Json(e.to_string()))
    }

    pub fn write_json(&self, path: &std::path::Path) -> Result<(), SimError> {
        std::fs::write(path, self.to_json_pretty()?).map_err(|e| SimError::Io {
            path: path.display().to_string(),
            message: e.to_string(),
        })
    }
}

/// Run and score the physical and physicochemical stages, with chemistry-data provenance.
pub fn run_with_physchem(
    table: &ProcessTable,
    engine: &PhysChemEngine,
    config: &RunConfig,
    manifest: &g4_core::manifest::Manifest,
) -> Result<PhysChemRunOutput, SimError> {
    let (summary, physchem) = run_physchem_summary(table, engine, config)?;
    let rotation = table
        .chain(g4_physics::models::ProcessKind::Elastic)
        .map(|c| c.models())
        .unwrap_or(&[])
        .iter()
        .find_map(|m| match m {
            g4_physics::models::ElectronModel::Elastic(c) => Some(format!("{:?}", c.rotation())),
            _ => None,
        })
        .unwrap_or_else(|| "n/a".to_owned());
    Ok(PhysChemRunOutput {
        schema: "geant4-rs/dna-sim/run/v3",
        provenance: Provenance::new(manifest, config.master_seed),
        chemistry_data_sha256: option3_data_sha256(),
        models: table.describe(),
        elastic_rotation: rotation,
        config: config.clone(),
        summary,
        physchem,
    })
}
