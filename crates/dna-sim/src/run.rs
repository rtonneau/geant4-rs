//! Deterministic parallel execution of histories.
//!
//! History `i` is transported on the stream `(master_seed, i, Physics)`; nothing about it depends
//! on the thread that runs it. Histories of a batch are computed in parallel into a `Vec` in
//! history order (`collect` into an indexed parallel iterator preserves order) and then folded
//! sequentially into the scorer, batch after batch, so scores are bitwise identical for any
//! thread count (docs/05 E.1-E.3, docs/reproducibility.md).

use crate::config::RunConfig;
use crate::error::SimError;
use crate::history::{HistoryResult, radial_edges_nm, run_history, spectrum_edges_ev};
use crate::scoring::{RunSummary, Scorer};
use dna_core::provenance::Provenance;
use dna_core::rng::RngStreams;
use dna_physics::physchem::{PHYSCHEM_CSV_HEADER, PhysChemEvent};
use dna_physics::table::{ModelInfo, ProcessTable};
use rayon::prelude::*;
use serde::Serialize;
use std::io::Write;

/// Receiver of the physicochemical events of a run, called once per history **in history
/// order** (so the output is identical for any thread count).
pub trait PhysChemSink: Send {
    fn write_history(&mut self, history_id: u64, events: &[PhysChemEvent]) -> Result<(), SimError>;
}

/// Writes the events as CSV (`PHYSCHEM_CSV_HEADER`, one row per event).
pub struct CsvPhysChemWriter<W: Write> {
    out: W,
    path: String,
    rows: u64,
}

impl<W: Write> CsvPhysChemWriter<W> {
    /// `path` only appears in error messages. Writes the header immediately.
    pub fn new(mut out: W, path: &str) -> Result<Self, SimError> {
        writeln!(out, "{PHYSCHEM_CSV_HEADER}").map_err(|e| SimError::Io {
            path: path.to_owned(),
            message: e.to_string(),
        })?;
        Ok(Self {
            out,
            path: path.to_owned(),
            rows: 0,
        })
    }

    /// Number of event rows written so far.
    pub fn rows(&self) -> u64 {
        self.rows
    }

    pub fn into_inner(self) -> W {
        self.out
    }
}

impl<W: Write + Send> PhysChemSink for CsvPhysChemWriter<W> {
    fn write_history(&mut self, _id: u64, events: &[PhysChemEvent]) -> Result<(), SimError> {
        for e in events {
            writeln!(self.out, "{}", e.csv_row()).map_err(|err| SimError::Io {
                path: self.path.clone(),
                message: err.to_string(),
            })?;
        }
        self.rows += events.len() as u64;
        Ok(())
    }
}

/// Bin edges shared by every history of a run.
struct Edges {
    spectrum: Vec<f64>,
    radial: Vec<f64>,
}

impl Edges {
    fn new() -> Self {
        Self {
            spectrum: spectrum_edges_ev(),
            radial: radial_edges_nm(),
        }
    }
}

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
    edges: &Edges,
    start: u64,
    end: u64,
) -> Result<Vec<HistoryResult>, SimError> {
    // `usize` ranges are indexed parallel iterators, so `collect` keeps history order.
    let n = (end - start) as usize;
    (0..n)
        .into_par_iter()
        .with_min_len(4)
        .map(|k| {
            run_history(
                table,
                config,
                streams,
                start + k as u64,
                &edges.spectrum,
                &edges.radial,
            )
        })
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
    let edges = Edges::new();
    with_pool(config.threads, || {
        run_batch(table, config, &streams, &edges, first, end)
    })?
}

/// Run and score; memory is bounded by `config.batch_size` results.
pub fn run_summary(table: &ProcessTable, config: &RunConfig) -> Result<RunSummary, SimError> {
    run_summary_with_sink(table, config, None)
}

/// Run and score; if `sink` is given, the physicochemical events of every history are passed to
/// it in history order (event recording is switched on for the run).
pub fn run_summary_with_sink(
    table: &ProcessTable,
    config: &RunConfig,
    mut sink: Option<&mut dyn PhysChemSink>,
) -> Result<RunSummary, SimError> {
    config.validate()?;
    let mut config = config.clone();
    config.record_physchem = sink.is_some();
    let config = &config;
    let (first, end) = ids(config)?;
    let streams = RngStreams::new(config.master_seed);
    let edges = Edges::new();
    let mut scorer = Scorer::new(config.primary_energy_ev);
    with_pool(config.threads, || -> Result<(), SimError> {
        let mut start = first;
        while start < end {
            let stop = start.saturating_add(config.batch_size as u64).min(end);
            for r in run_batch(table, config, &streams, &edges, start, stop)? {
                scorer.add(&r);
                if let Some(sink) = sink.as_deref_mut() {
                    sink.write_history(r.id, &r.physchem)?;
                }
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
    run_with_sink(table, config, manifest, None)
}

/// As [`run`], with the physicochemical events streamed to `sink` (see
/// [`run_summary_with_sink`]).
pub fn run_with_sink(
    table: &ProcessTable,
    config: &RunConfig,
    manifest: &dna_core::manifest::Manifest,
    sink: Option<&mut dyn PhysChemSink>,
) -> Result<RunOutput, SimError> {
    let summary = run_summary_with_sink(table, config, sink)?;
    let rotation = table
        .chain(dna_physics::models::ProcessKind::Elastic)
        .map(|c| c.models())
        .unwrap_or(&[])
        .iter()
        .find_map(|m| match m {
            dna_physics::models::ElectronModel::Elastic(c) => Some(format!("{:?}", c.rotation())),
            _ => None,
        })
        .unwrap_or_else(|| "n/a".to_owned());
    Ok(RunOutput {
        schema: "geant4-rs/dna-sim/run/v2",
        provenance: Provenance::new(manifest, config.master_seed),
        models: table.describe(),
        elastic_rotation: rotation,
        config: config.clone(),
        summary,
    })
}
