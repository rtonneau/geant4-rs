//! Model windows and boundaries, on the committed tables.
//!
//! Windows (docs/02 B.7.2): Champion [7.4 eV, 1 MeV] (`G4DNAChampionElasticModel.cc:46-47,253`),
//! Born excitation [9 eV, 1 MeV] (`G4DNABornExcitationModel1.cc:99-100,157`), Born1 ionisation
//! [11 eV, 1 MeV] (`G4DNABornIonisationModel1.cc:126-127,363`). Each model returns zero outside
//! its closed window and a positive value on both boundaries; sampling outside is
//! `UnsupportedEnergy`. Neighbours are the adjacent doubles (`nextafter`).

mod common;

use common::{data, ev};
use dna_core::error::DnaError;
use dna_core::rng::{RngStreams, Stage};
use dna_core::vec3::Vec3;
use dna_physics::PhysicsError;
use dna_physics::models::{
    BornExcitation, BornIonisation, ChampionElastic, CrossSectionModel, ElasticRotation,
    ElectronModel, ProcessKind,
};
use dna_physics::table::ProcessTable;
use dna_physics::transport::{NoObserver, Primary, TransportConfig, transport_electron};

fn next_up(x: f64) -> f64 {
    f64::from_bits(x.to_bits() + 1)
}
fn next_down(x: f64) -> f64 {
    f64::from_bits(x.to_bits() - 1)
}

fn is_unsupported(e: &PhysicsError) -> bool {
    matches!(e, PhysicsError::Core(DnaError::UnsupportedEnergy { .. }))
}

/// Cross section is zero strictly outside, positive on the boundaries and next to them inside;
/// sampling mirrors it.
fn check_window(model: &ElectronModel, low: f64, high: f64, rng: &mut dna_core::rng::HistoryRng) {
    let dir = Vec3::new(0.0, 0.0, 1.0);
    assert_eq!(model.energy_range().min().ev(), low);
    assert_eq!(model.energy_range().max().ev(), high);
    for e in [next_down(low), next_up(high), 1.0, 0.0, 2.0e6] {
        assert_eq!(
            model.cross_section(ev(e)).unwrap().nm2(),
            0.0,
            "{} at {e}",
            model.name()
        );
        let err = model.sample(ev(e), dir, rng).unwrap_err();
        assert!(is_unsupported(&err), "{} at {e}: {err:?}", model.name());
    }
    for e in [low, next_up(low), next_down(high), high] {
        assert!(
            model.cross_section(ev(e)).unwrap().nm2() > 0.0,
            "{} at {e}",
            model.name()
        );
        let fs = model.sample(ev(e), dir, rng).unwrap();
        assert!(fs.energy_after.ev() <= e);
        assert!((fs.direction_after.norm() - 1.0).abs() < 1e-12);
    }
}

#[test]
fn each_model_is_zero_outside_and_positive_on_its_closed_window() {
    let Some(dir) = data() else { return };
    let mut rng = RngStreams::new(5).stream(0, Stage::Physics).unwrap();
    for (model, low, high) in [
        (
            ElectronModel::Elastic(
                ChampionElastic::from_data_dir(&dir, ElasticRotation::Geant4).unwrap(),
            ),
            7.4,
            1.0e6,
        ),
        (
            ElectronModel::Excitation(BornExcitation::from_data_dir(&dir).unwrap()),
            9.0,
            1.0e6,
        ),
        (
            ElectronModel::Ionisation(BornIonisation::from_data_dir(&dir).unwrap()),
            11.0,
            1.0e6,
        ),
    ] {
        check_window(&model, low, high, &mut rng);
    }
}

#[test]
fn the_table_assembles_the_option2_windows() {
    let Some(dir) = data() else { return };
    let table = ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap();
    let range = table.primary_energy_range();
    assert_eq!((range.min().ev(), range.max().ev()), (7.4, 1.0e6));
    for (p, lo) in [
        (ProcessKind::Elastic, 7.4),
        (ProcessKind::Excitation, 9.0),
        (ProcessKind::Ionisation, 11.0),
    ] {
        assert!(table.select(p, ev(lo)).is_some());
        assert!(table.select(p, ev(next_down(lo))).is_none());
        assert!(table.select(p, ev(1.0e6)).is_some());
        assert!(table.select(p, ev(next_up(1.0e6))).is_none());
    }
    // macroscopic cross sections follow the windows
    let below = table.macroscopic(ev(next_down(7.4))).unwrap();
    assert_eq!(below.total(), 0.0);
    let only_elastic = table.macroscopic(ev(8.0)).unwrap();
    assert!(only_elastic.get(ProcessKind::Elastic) > 0.0);
    assert_eq!(only_elastic.inelastic(), 0.0);
    let excitation_starts = table.macroscopic(ev(9.0)).unwrap();
    assert!(excitation_starts.get(ProcessKind::Excitation) > 0.0);
    assert_eq!(excitation_starts.get(ProcessKind::Ionisation), 0.0);
    let ionisation_starts = table.macroscopic(ev(11.0)).unwrap();
    assert!(ionisation_starts.get(ProcessKind::Ionisation) > 0.0);
    assert_eq!(table.macroscopic(ev(next_up(1.0e6))).unwrap().total(), 0.0);
}

#[test]
fn primaries_outside_7_4_ev_to_1_mev_are_rejected_with_unsupported_energy() {
    let Some(dir) = data() else { return };
    let table = ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap();
    let mut rng = RngStreams::new(5).stream(0, Stage::Physics).unwrap();
    for e in [
        next_up(1.0e6),
        1.0e6 * 1.001,
        2.0e6,
        1.0e7,
        next_down(7.4),
        1.0,
        0.0,
    ] {
        let err = transport_electron(
            &table,
            &TransportConfig::default(),
            Primary {
                energy: ev(e),
                position: Vec3::ZERO,
                direction: Vec3::new(0.0, 0.0, 1.0),
            },
            &mut rng,
            &mut NoObserver,
        )
        .unwrap_err();
        assert!(is_unsupported(&err), "{e}: {err:?}");
    }
}

#[test]
fn a_zero_direction_is_rejected() {
    let Some(dir) = data() else { return };
    let table = ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap();
    let mut rng = RngStreams::new(5).stream(0, Stage::Physics).unwrap();
    let err = transport_electron(
        &table,
        &TransportConfig::default(),
        Primary {
            energy: ev(1000.0),
            position: Vec3::ZERO,
            direction: Vec3::ZERO,
        },
        &mut rng,
        &mut NoObserver,
    )
    .unwrap_err();
    assert_eq!(err, PhysicsError::InvalidDirection);
}
