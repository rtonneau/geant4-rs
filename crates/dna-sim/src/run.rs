//! Deterministic parallel execution of histories.
//!
//! History `i` is transported on the stream `(master_seed, i, Physics)`; nothing about it depends
//! on the thread that runs it. Histories of a batch are computed in parallel into a `Vec` in
//! history order (`collect` into an indexed parallel iterator preserves order) and then folded
//! sequentially into the scorer, batch after batch, so scores are bitwise identical for any
//! thread count (docs/05 E.1-E.3, docs/reproducibility.md).

use crate::config::RunConfig;
use crate::error::SimError;
use crate::history::{HistoryResult, run_history, spectrum_edges_ev};
use crate::scoring::{RunSummary, Scorer};
use dna_core::provenance::Provenance;
use dna_core::rng::RngStreams;
use dna_physics::table::{ModelInfo, ProcessTable};
use rayon::prelude::*;
use serde::Serialize;

fn with_pool<T: Send>(threads: Option<usize>, f: impl FnOnce() -> T + Send) -> Result<T, SimError> {
    match threads {
        None => Ok(f()),
        Some(n) => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(n)
                .build()
                .map_err(|e| SimError::ThreadPool(e.to_string()))?;
            Ok(pool.install(f))
        }
    }
}

fn run_batch(
    table: &ProcessTable,
    config: &RunConfig,
    streams: &RngStreams,
    edges: &[f64],
    start: u64,
    end: u64,
) -> Result<Vec<HistoryResult>, SimError> {
    // `usize` ranges are indexed parallel iterators, so `collect` keeps history order.
    let n = (end - start) as usize;
    (0..n)
        .into_par_iter()
        .with_min_len(4)
        .map(|k| run_history(table, config, streams, start + k as u64, edges))
        .collect()
}

fn ids(config: &RunConfig) -> Result<(u64, u64), SimError> {
    let end = config
        .first_history_id
        .checked_add(config.n_histories)
        .ok_or_else(|| SimError::InvalidConfig("history id range overflows u64".into()))?;
    Ok((config.first_history_id, end))
}

/// All per-history results, in history order (memory O(n); for tests and small runs).
pub fn run_detailed(
    table: &ProcessTable,
    config: &RunConfig,
) -> Result<Vec<HistoryResult>, SimError> {
    config.validate()?;
    let (first, end) = ids(config)?;
    let streams = RngStreams::new(config.master_seed);
    let edges = spectrum_edges_ev();
    with_pool(config.threads, || {
        run_batch(table, config, &streams, &edges, first, end)
    })?
}

/// Run and score; memory is bounded by `config.batch_size` results.
pub fn run_summary(table: &ProcessTable, config: &RunConfig) -> Result<RunSummary, SimError> {
    config.validate()?;
    let (first, end) = ids(config)?;
    let streams = RngStreams::new(config.master_seed);
    let edges = spectrum_edges_ev();
    let mut scorer = Scorer::new(config.primary_energy_ev);
    with_pool(config.threads, || -> Result<(), SimError> {
        let mut start = first;
        while start < end {
            let stop = start.saturating_add(config.batch_size as u64).min(end);
            for r in run_batch(table, config, &streams, &edges, start, stop)? {
                scorer.add(&r);
            }
            start = stop;
        }
        Ok(())
    })??;
    Ok(scorer.finish())
}

/// Machine-readable result of a run with its provenance.
#[derive(Debug, Clone, Serialize)]
pub struct RunOutput {
    /// Version of this JSON layout.
    pub schema: &'static str,
    pub provenance: Provenance,
    /// Models used, with Geant4 source paths and data files.
    pub models: Vec<ModelInfo>,
    /// Elastic-scattering rotation convention (see `dna_physics::models::ElasticRotation`).
    pub elastic_rotation: String,
    pub config: RunConfig,
    pub summary: RunSummary,
}

impl RunOutput {
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

/// Run, score and attach provenance. `manifest` is the checksum manifest of the data in use.
pub fn run(
    table: &ProcessTable,
    config: &RunConfig,
    manifest: &dna_core::manifest::Manifest,
) -> Result<RunOutput, SimError> {
    let summary = run_summary(table, config)?;
    let rotation = table
        .chain(dna_physics::models::ProcessKind::Elastic)
        .models()
        .iter()
        .find_map(|m| match m {
            dna_physics::models::ElectronModel::Elastic(c) => Some(format!("{:?}", c.rotation())),
            _ => None,
        })
        .unwrap_or_else(|| "n/a".to_owned());
    Ok(RunOutput {
        schema: "geant4-rs/dna-sim/run/v1",
        provenance: Provenance::new(manifest, config.master_seed),
        models: table.describe(),
        elastic_rotation: rotation,
        config: config.clone(),
        summary,
    })
}
