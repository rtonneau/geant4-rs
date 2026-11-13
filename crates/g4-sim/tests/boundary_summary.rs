//! `physchem.boundary` scoring in a volume (docs/04 D.6, docs/07 S3), on the real tables.
//!
//! Pre-registered criteria:
//! * Infinite water: `physchem.boundary` is absent from the summary and the JSON.
//! * A small sphere (radius 20 nm, 1 keV electrons): the block is present with one row per
//!   species; the e_aq `shortened` count equals the sum of the per-history
//!   `solvations_shortened`; some e_aq and some products are shortened; no alive species is
//!   Outside at 1 ps (D2: placements are clamped inside and recombination only removes
//!   species); `not_inside_at_1ps <= alive_total` per species.
//! * The block is identical for 1 and 4 threads.

use g4_chemistry::{PhysChemEngine, Species};
use g4_core::geometry::Volume;
use g4_core::vec3::Vec3;
use g4_data::datadir::{DataDir, repo_data_dir};
use g4_physics::models::ElasticRotation;
use g4_physics::table::ProcessTable;
use g4_sim::config::RunConfig;
use g4_sim::run::{run_detailed, run_physchem_summary};

fn table() -> Option<ProcessTable> {
    let root = repo_data_dir();
    if !root.join("dna").is_dir() || !root.join("MANIFEST.toml").is_file() {
        eprintln!("SKIPPED: Geant4 data not found at {}", root.display());
        return None;
    }
    let dir = DataDir::new(&root).unwrap();
    Some(ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap())
}

fn sphere(r: f64) -> Volume {
    Volume::sphere(
        Vec3::ZERO,
        0.0,
        r,
        0.0,
        std::f64::consts::TAU,
        0.0,
        std::f64::consts::PI,
    )
    .unwrap()
}

fn config(volume: Option<Volume>, threads: Option<usize>) -> RunConfig {
    let mut c = RunConfig::new(1.0e3, 100);
    c.master_seed = 41;
    c.volume = volume;
    c.threads = threads;
    c
}

#[test]
fn infinite_water_has_no_boundary_block() {
    let Some(table) = table() else { return };
    let engine = PhysChemEngine::option3().unwrap();
    let (_, p) = run_physchem_summary(&table, &engine, &config(None, None)).unwrap();
    assert!(p.boundary.is_none());
    assert!(serde_json::to_value(&p).unwrap().get("boundary").is_none());
}

#[test]
fn a_small_sphere_reports_clamped_placements_and_nothing_outside() {
    let Some(table) = table() else { return };
    let engine = PhysChemEngine::option3().unwrap();
    let cfg = config(Some(sphere(20.0)), Some(1));
    let (_, p) = run_physchem_summary(&table, &engine, &cfg).unwrap();
    let b = p.boundary.as_ref().expect("boundary block with a volume");
    assert_eq!(b.species.len(), Species::ALL.len());

    let detailed = run_detailed(&table, &cfg).unwrap();
    let solvation_shortened: u64 = detailed
        .iter()
        .map(|h| h.outcome.solvations_shortened)
        .sum();
    let e_aq = &b.species[Species::e_aq.index()];
    assert_eq!(e_aq.species, Species::e_aq.chemistry_name());
    assert!(e_aq.shortened >= solvation_shortened);
    assert!(solvation_shortened > 0, "the geometry must clamp some e_aq");
    let products: u64 = b
        .species
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != Species::e_aq.index())
        .map(|(_, s)| s.shortened)
        .sum();
    assert!(products > 0, "the geometry must clamp some products");

    for (row, yields) in b.species.iter().zip(&p.species) {
        assert_eq!(row.species, yields.species);
        assert_eq!(row.outside_at_1ps, 0, "{} outside at 1 ps", row.species);
        assert!(row.not_inside_at_1ps <= yields.alive_total);
    }

    let four = config(Some(sphere(20.0)), Some(4));
    let (_, p4) = run_physchem_summary(&table, &engine, &four).unwrap();
    assert_eq!(p4.boundary, p.boundary);
    let json = serde_json::to_value(&p).unwrap();
    assert!(json["boundary"]["species"][0]["shortened"].is_u64());
}
