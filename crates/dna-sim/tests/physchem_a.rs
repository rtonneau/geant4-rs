//! M6 part A on real cascades: the physical stage (M3, the Geant4 11.4.1 option2 electron models on the committed
//! data) feeds the physicochemical stage (`dna-chemistry`).
//!
//! Acceptance criteria, fixed before the first run:
//! * the energy ledger of every history stays conserved (|residual| < 1e-9 eV relative, as in M3) and the deposits
//!   carried by the physicochemical events add up to the ledger per kind (excitation, binding, attachment,
//!   solvation) to a relative 1e-9: converting events to species never touches the ledger;
//! * for every history, the species obey exact conservation: with N the number of water-molecule events
//!   (excitation + ionisation + attachment) that do not relax to the ground state (a relaxation leaves no species)
//!   and W the number of water molecules of the medium absorbed by the
//!   selected channels (hand table `channel_balance` of `hand_values.json`),
//!   H = 2 (N + W), O = N + W and charge = n_ionisation - n_attachment - n_solvation;
//! * the number of e_aq is `n_solvation + (number of auto-ionisation channels) - recombinations`;
//! * every species is born at 1 ps;
//! * the species lists are bitwise identical for 1, 2 and 8 threads.
//!
//! No yield is asserted: the comparison with Geant4 is part B (docs/next.md).

use dna_chemistry::PhysChemEngine;
use dna_chemistry::channels::DisplacementKind;
use dna_chemistry::species::atoms;
use dna_core::manifest::Manifest;
use dna_core::physchem::PhysChemKind;
use dna_core::rng::{RngStreams, Stage};
use dna_core::species::Species;
use dna_data::datadir::{DataDir, repo_data_dir};
use dna_physics::models::ElasticRotation;
use dna_physics::table::ProcessTable;
use dna_sim::config::RunConfig;
use dna_sim::history::{HistoryResult, radial_edges_nm, run_history, spectrum_edges_ev};
use rayon::prelude::*;
use std::collections::HashMap;

fn setup() -> Option<ProcessTable> {
    let root = repo_data_dir();
    if !root.join("dna").is_dir() || !root.join("MANIFEST.toml").is_file() {
        eprintln!("SKIPPED: Geant4 data not found at {}", root.display());
        return None;
    }
    let _ = Manifest::load(&root.join("MANIFEST.toml")).unwrap();
    let dir = DataDir::new(&root).unwrap();
    Some(ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap())
}

fn histories(table: &ProcessTable, energy_ev: f64, n: u64, seed: u64) -> Vec<HistoryResult> {
    let mut config = RunConfig::new(energy_ev, n);
    config.master_seed = seed;
    config.record_physchem = true;
    let streams = RngStreams::new(seed);
    let (se, re) = (spectrum_edges_ev(), radial_edges_nm());
    (0..n)
        .map(|id| run_history(table, &config, &streams, id, &se, &re).unwrap())
        .collect()
}

fn extra_waters() -> HashMap<String, i64> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../dna-chemistry/tests/fixtures/hand_values.json"
    );
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    v["channel_balance"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, e)| (k.clone(), e["extra_waters"].as_i64().unwrap()))
        .collect()
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs().max(1e-300)
}

#[test]
fn species_conservation_and_untouched_ledger_on_real_cascades() {
    let Some(table) = setup() else { return };
    let engine = PhysChemEngine::option3().unwrap();
    let waters = extra_waters();
    let streams = RngStreams::new(20261015);
    let mut total_events = 0u64;
    let mut n_recombined = 0u64;
    for (energy, n) in [(300.0, 40), (1_000.0, 40), (10_000.0, 8)] {
        for h in histories(&table, energy, n, 20261015) {
            let l = &h.outcome.ledger;
            assert!(
                l.residual_relative().abs() < 1e-9,
                "history {}: ledger residual {}",
                h.id,
                l.residual()
            );
            // deposits of the events add up to the ledger, per kind
            let by_kind = |k: PhysChemKind| -> f64 {
                h.physchem
                    .iter()
                    .filter(|e| e.kind == k)
                    .map(|e| e.deposited_energy.ev())
                    .sum()
            };
            for (k, ledger) in [
                (PhysChemKind::Excitation, l.deposited_excitation),
                (PhysChemKind::Ionisation, l.deposited_binding),
                (PhysChemKind::Attachment, l.deposited_attachment),
                (PhysChemKind::Solvation, l.deposited_solvation),
            ] {
                let s = by_kind(k);
                assert!(
                    ledger == 0.0 && s == 0.0 || rel(s, ledger) < 1e-9,
                    "history {}: {k:?} events {s} vs ledger {ledger}",
                    h.id
                );
            }
            let events_before = h.physchem.clone();
            let mut rng = streams.stream(h.id, Stage::PhysChem).unwrap();
            let r = engine.convert_history(&h.physchem, &mut rng).unwrap();
            assert_eq!(h.physchem, events_before, "the event list is not modified");

            // exact conservation
            let n_of = |k: PhysChemKind| h.physchem.iter().filter(|e| e.kind == k).count() as i64;
            let (n_exc, n_ion, n_att, n_solv) = (
                n_of(PhysChemKind::Excitation),
                n_of(PhysChemKind::Ionisation),
                n_of(PhysChemKind::Attachment),
                n_of(PhysChemKind::Solvation),
            );
            // molecules that relax back to the ground state leave no species behind
            let mut n_relax = 0i64;
            let mut w = 0i64;
            let mut n_auto = 0i64;
            for (si, st) in engine.channel_table().states().iter().enumerate() {
                for (ci, ch) in st.channels.iter().enumerate() {
                    let c = r.stats.channel_counts[si][ci] as i64;
                    w += c * waters[&format!("{}/{}", st.name, ch.name)];
                    if ch.products.is_empty() {
                        n_relax += c;
                    }
                    if ch.displacement == DisplacementKind::AutoIonisation {
                        n_auto += c;
                    }
                }
            }
            let n_mol = n_exc + n_ion + n_att - n_relax;
            assert_eq!(
                n_exc + n_ion + n_att,
                r.stats.events[0] as i64 + r.stats.events[1] as i64 + r.stats.events[2] as i64
            );
            let (mut hh, mut oo, mut q) = (0i64, 0i64, 0i64);
            for s in &r.species {
                let (a, b) = atoms(s.species);
                hh += i64::from(a);
                oo += i64::from(b);
                q += i64::from(engine.species_table().get(s.species).charge);
                assert_eq!(s.time.ps(), 1.0);
            }
            assert_eq!(hh, 2 * (n_mol + w), "history {}: hydrogen atoms", h.id);
            assert_eq!(oo, n_mol + w, "history {}: oxygen atoms", h.id);
            assert_eq!(q, n_ion - n_att - n_solv, "history {}: charge", h.id);
            let n_eaq = r
                .species
                .iter()
                .filter(|s| s.species == Species::EAq)
                .count() as i64;
            assert_eq!(
                n_eaq,
                n_solv + n_auto - r.stats.recombinations as i64,
                "history {}: e_aq balance",
                h.id
            );
            assert_eq!(
                r.stats.events.iter().sum::<u64>() as usize,
                h.physchem.len()
            );
            total_events += h.physchem.len() as u64;
            n_recombined += r.stats.recombinations;
        }
    }
    eprintln!("converted {total_events} events, {n_recombined} recombinations");
    assert!(total_events > 1000);
}

#[test]
fn species_lists_of_real_cascades_are_bitwise_identical_for_any_thread_count() {
    let Some(table) = setup() else { return };
    let engine = PhysChemEngine::option3().unwrap();
    let streams = RngStreams::new(20261016);
    let hs = histories(&table, 3_000.0, 24, 20261016);
    let bits = |threads: usize| -> Vec<Vec<u64>> {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                hs.par_iter()
                    .map(|h| {
                        let mut rng = streams.stream(h.id, Stage::PhysChem).unwrap();
                        let r = engine.convert_history(&h.physchem, &mut rng).unwrap();
                        r.species
                            .iter()
                            .flat_map(|s| {
                                [
                                    s.species.index() as u64,
                                    s.position.x.to_bits(),
                                    s.position.y.to_bits(),
                                    s.position.z.to_bits(),
                                ]
                            })
                            .collect()
                    })
                    .collect()
            })
    };
    let one = bits(1);
    assert_eq!(one, bits(2));
    assert_eq!(one, bits(8));
    assert!(one.iter().map(Vec::len).sum::<usize>() > 1000);
}

#[test]
fn informational_initial_species_per_100_ev_are_printed_not_asserted() {
    let Some(table) = setup() else { return };
    let engine = PhysChemEngine::option3().unwrap();
    let streams = RngStreams::new(20261017);
    let hs = histories(&table, 10_000.0, 20, 20261017);
    let mut counts = [0u64; 15];
    let mut e_dep = 0.0;
    let mut rec = (0u64, 0u64, 0u64);
    for h in &hs {
        let mut rng = streams.stream(h.id, Stage::PhysChem).unwrap();
        let r = engine.convert_history(&h.physchem, &mut rng).unwrap();
        for (c, n) in counts.iter_mut().zip(r.stats.species_counts) {
            *c += n;
        }
        e_dep += h.outcome.ledger.deposited();
        rec.0 += r.stats.recombinations;
        rec.1 += r.stats.holes_with_candidate;
        rec.2 += r.stats.events[1];
    }
    eprintln!(
        "10 keV electrons, {} histories, E_dep = {e_dep:.1} eV (species per 100 eV at 1 ps, NOT validated, no Geant4 comparison yet):",
        hs.len()
    );
    for s in Species::ALL {
        if counts[s.index()] > 0 {
            eprintln!(
                "  {:>5}: {:.3}",
                s.tag(),
                counts[s.index()] as f64 / (e_dep / 100.0)
            );
        }
    }
    eprintln!(
        "  ionised molecules {}, with an e_aq within 10 r_Onsager {}, recombined {}",
        rec.2, rec.1, rec.0
    );
    assert!(e_dep > 0.0);
}
