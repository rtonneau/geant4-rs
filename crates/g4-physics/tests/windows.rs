//! Model windows and boundaries, on the committed tables.
//!
//! Windows (docs/02 B.7.2): Champion [7.4 eV, 1 MeV] (`G4DNAChampionElasticModel.cc:46-47,253`),
//! Born excitation [9 eV, 1 MeV] (`G4DNABornExcitationModel1.cc:99-100,157`), Born1 ionisation
//! [11 eV, 1 MeV] (`G4DNABornIonisationModel1.cc:126-127,363`), Sanche vibrational excitation
//! [2 eV, 100 eV] (`G4DNASancheExcitationModel.cc:48-49,207`; Melton and solvation are in
//! `low_energy.rs`). Each model returns zero outside
//! its closed window and a positive value on both boundaries; sampling outside is
//! `UnsupportedEnergy`. Neighbours are the adjacent doubles (`nextafter`).

mod common;

use common::{data, ev};
use g4_core::error::DnaError;
use g4_core::manifest::Manifest;
use g4_core::rng::{RngStreams, Stage};
use g4_core::testing::ScriptedRng;
use g4_core::vec3::Vec3;
use g4_data::catalog::DnaFile;
use g4_data::datadir::{DataDir, VerifiedDataSet};
use g4_physics::PhysicsError;
use g4_physics::models::{
    BornExcitation, BornIonisation, ChampionElastic, CrossSectionModel, ElasticRotation,
    ElectronModel, ProcessKind, SancheVibrational,
};
use g4_physics::table::{ProcessTable, VerifiedProcessTable};
use g4_physics::transport::{NoObserver, Primary, TransportConfig, transport_electron};

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
fn check_window(model: &ElectronModel, low: f64, high: f64, rng: &mut g4_core::rng::HistoryRng) {
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
        (
            ElectronModel::Vibrational(SancheVibrational::from_data_dir(&dir).unwrap()),
            2.0,
            100.0,
        ),
    ] {
        check_window(&model, low, high, &mut rng);
    }
}

#[test]
fn the_table_assembles_the_option2_windows() {
    let Some(dir) = data() else { return };
    let manifest = Manifest::load(&dir.root().join("MANIFEST.toml")).unwrap();
    let snapshot = VerifiedDataSet::load(&dir, &manifest).unwrap();
    let verified =
        VerifiedProcessTable::option2_electron(&snapshot, ElasticRotation::Geant4).unwrap();
    assert_eq!(verified.identity(), &manifest);
    let table = verified.table();
    let range = table.primary_energy_range();
    assert_eq!((range.min().ev(), range.max().ev()), (2.0, 1.0e6));
    // (process, low, high); each process has one model, closed at both ends
    for (p, lo, hi) in [
        (ProcessKind::Elastic, 7.4, 1.0e6),
        (ProcessKind::Excitation, 9.0, 1.0e6),
        (ProcessKind::Ionisation, 11.0, 1.0e6),
        (ProcessKind::VibrationalExcitation, 2.0, 100.0),
        (ProcessKind::Attachment, 4.0, 13.0),
    ] {
        assert!(table.select(p, ev(lo)).is_some(), "{p:?} at {lo}");
        assert!(table.select(p, ev(next_down(lo))).is_none(), "{p:?}");
        assert!(table.select(p, ev(hi)).is_some(), "{p:?} at {hi}");
        assert!(table.select(p, ev(next_up(hi))).is_none(), "{p:?}");
    }
    // macroscopic cross sections follow the windows
    let below = table.macroscopic(ev(next_down(2.0))).unwrap();
    assert_eq!(below.total(), 0.0);
    let vib_only = table.macroscopic(ev(3.0)).unwrap();
    assert!(vib_only.get(ProcessKind::VibrationalExcitation) > 0.0);
    assert_eq!(
        vib_only.total(),
        vib_only.get(ProcessKind::VibrationalExcitation)
    );
    let between = table.macroscopic(ev(8.0)).unwrap();
    assert!(between.get(ProcessKind::Elastic) > 0.0);
    assert!(between.get(ProcessKind::VibrationalExcitation) > 0.0);
    assert!(between.get(ProcessKind::Attachment) > 0.0);
    assert_eq!(between.get(ProcessKind::Excitation), 0.0);
    assert_eq!(between.get(ProcessKind::Ionisation), 0.0);
    let excitation_starts = table.macroscopic(ev(9.0)).unwrap();
    assert!(excitation_starts.get(ProcessKind::Excitation) > 0.0);
    assert_eq!(excitation_starts.get(ProcessKind::Ionisation), 0.0);
    let ionisation_starts = table.macroscopic(ev(11.0)).unwrap();
    assert!(ionisation_starts.get(ProcessKind::Ionisation) > 0.0);
    // Sanche is closed at 100 eV, open one ulp below and absent one ulp above
    assert!(
        table
            .macroscopic(ev(100.0))
            .unwrap()
            .get(ProcessKind::VibrationalExcitation)
            > 0.0
    );
    assert_eq!(
        table
            .macroscopic(ev(next_up(100.0)))
            .unwrap()
            .get(ProcessKind::VibrationalExcitation),
        0.0
    );
    assert_eq!(table.macroscopic(ev(next_up(1.0e6))).unwrap().total(), 0.0);
    // solvation owns [0, 7.4 eV] (closed): no cross-section process serves 7.4 eV except elastic
    let solv = table.solvation().unwrap();
    assert!(solv.applies(ev(0.0)) && solv.applies(ev(7.4)) && !solv.applies(ev(next_up(7.4))));
}

#[test]
fn primaries_outside_2_ev_to_1_mev_are_rejected_with_unsupported_energy() {
    let Some(dir) = data() else { return };
    let table = ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap();
    let mut rng = RngStreams::new(5).stream(0, Stage::Physics).unwrap();
    for e in [
        next_up(1.0e6),
        1.0e6 * 1.001,
        2.0e6,
        1.0e7,
        next_down(2.0),
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
                history_id: 0,
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
            history_id: 0,
        },
        &mut rng,
        &mut NoObserver,
    )
    .unwrap_err();
    assert_eq!(err, PhysicsError::InvalidDirection);
}

fn synthetic_xs(count: usize) -> g4_data::xs::CrossSectionTable {
    use g4_core::units::Area;
    use g4_data::xs::{CrossSectionTable, Interpolation, XsSpec};
    let values = " 1".repeat(count);
    CrossSectionTable::from_text(
        "shape",
        &format!("1{values}\n1000000{values}\n"),
        XsSpec {
            scale: Area::from_nm2(1.0),
            interpolation: Interpolation::LinLin,
        },
    )
    .unwrap()
}

fn synthetic_cdf() -> g4_data::blocks::IonisationCdfTable {
    g4_data::blocks::IonisationCdfTable::from_text(
        "shape",
        "1 1 0 0 0 0 0\n1 2 1 1 1 1 1\n1000000 1 0 0 0 0 0\n1000000 2 1 1 1 1 1\n",
    )
    .unwrap()
}

#[test]
fn direct_models_reject_missing_and_extra_channels() {
    use g4_data::DataError;
    for count in [4, 6] {
        assert!(matches!(
            BornExcitation::new(synthetic_xs(count)),
            Err(PhysicsError::Data(DataError::InvalidTable { .. }))
        ));
        assert!(matches!(
            BornIonisation::new(synthetic_xs(count), synthetic_cdf()),
            Err(PhysicsError::Data(DataError::InvalidTable { .. }))
        ));
    }
    for count in [8, 10] {
        assert!(matches!(
            SancheVibrational::new(synthetic_xs(count)),
            Err(PhysicsError::Data(DataError::InvalidTable { .. }))
        ));
    }
    assert!(BornExcitation::new(synthetic_xs(5)).is_ok());
    assert!(BornIonisation::new(synthetic_xs(5), synthetic_cdf()).is_ok());
    assert!(SancheVibrational::new(synthetic_xs(9)).is_ok());
}

proptest::proptest! {
    #[test]
    fn models_require_exact_channel_counts(count in 1_usize..20) {
        proptest::prop_assert_eq!(BornExcitation::new(synthetic_xs(count)).is_ok(), count == 5);
        proptest::prop_assert_eq!(BornIonisation::new(synthetic_xs(count), synthetic_cdf()).is_ok(), count == 5);
        proptest::prop_assert_eq!(SancheVibrational::new(synthetic_xs(count)).is_ok(), count == 9);
    }
}

#[test]
fn verified_option2_preserves_cross_sections_final_states_and_rng_consumption() {
    let Some(dir) = data() else { return };
    let manifest = Manifest::load(&dir.root().join("MANIFEST.toml")).unwrap();
    let snapshot = VerifiedDataSet::load(&dir, &manifest).unwrap();
    for rotation in [ElasticRotation::Geant4, ElasticRotation::Exact] {
        let verified = VerifiedProcessTable::option2_electron(&snapshot, rotation).unwrap();
        let original = ProcessTable::option2_electron_unverified(&dir, rotation).unwrap();
        assert_eq!(verified.table().describe(), original.describe());
        for energy in [
            0.0, 2.0, 4.0, 7.4, 8.0, 9.0, 11.0, 13.0, 50.0, 100.0, 1e3, 1e6,
        ] {
            assert_eq!(
                verified.table().macroscopic(ev(energy)).unwrap(),
                original.macroscopic(ev(energy)).unwrap(),
                "{rotation:?} at {energy} eV"
            );
            for process in ProcessKind::ALL {
                let Some(before) = original.select(process, ev(energy)) else {
                    continue;
                };
                let after = verified.table().select(process, ev(energy)).unwrap();
                let mut rng_before = ScriptedRng::new(&[0.17, 0.43, 0.61, 0.83, 0.29]);
                let mut rng_after = rng_before.clone();
                let direction = Vec3::new(1.0, 2.0, 3.0).normalised().unwrap();
                assert_eq!(
                    before
                        .sample(ev(energy), direction, &mut rng_before)
                        .unwrap(),
                    after.sample(ev(energy), direction, &mut rng_after).unwrap(),
                    "{rotation:?} {process:?} at {energy} eV"
                );
                assert_eq!(rng_before.consumed(), rng_after.consumed());
            }
        }
    }
}

#[test]
fn verified_option2_uses_frozen_bytes_and_keeps_identity_after_sources_change() {
    let Some(dir) = data() else { return };
    let root = std::env::temp_dir().join(format!("g4-physics-frozen-{}", std::process::id()));
    std::fs::create_dir_all(root.join("dna")).unwrap();
    let manifest = Manifest::load(&dir.root().join("MANIFEST.toml")).unwrap();
    for entry in manifest.files() {
        std::fs::copy(dir.root().join(&entry.path), root.join(&entry.path)).unwrap();
    }
    std::fs::copy(dir.root().join("MANIFEST.toml"), root.join("MANIFEST.toml")).unwrap();
    let relocated_dir = DataDir::new(&root).unwrap();
    let relocated_manifest = Manifest::load(&root.join("MANIFEST.toml")).unwrap();
    let snapshot = VerifiedDataSet::load(&relocated_dir, &relocated_manifest).unwrap();
    let before =
        VerifiedProcessTable::option2_electron(&snapshot, ElasticRotation::Geant4).unwrap();

    std::fs::write(
        relocated_dir.path(DnaFile::ChampionElasticE),
        b"changed after capture",
    )
    .unwrap();
    std::fs::remove_file(root.join("MANIFEST.toml")).unwrap();
    assert!(VerifiedDataSet::load(&relocated_dir, &manifest).is_err());
    let after = VerifiedProcessTable::option2_electron(&snapshot, ElasticRotation::Geant4).unwrap();
    // Each returned model owns its tables; it does not depend on the snapshot's lifetime either.
    drop(snapshot);
    assert_eq!(before.identity(), &manifest);
    assert_eq!(after.identity(), &manifest);
    for energy in [7.4, 100.0, 1e3, 1e6] {
        assert_eq!(
            before.table().macroscopic(ev(energy)),
            after.table().macroscopic(ev(energy))
        );
        let before_model = before
            .table()
            .select(ProcessKind::Elastic, ev(energy))
            .unwrap();
        let after_model = after
            .table()
            .select(ProcessKind::Elastic, ev(energy))
            .unwrap();
        let mut rng_before = ScriptedRng::new(&[0.1, 0.9]);
        let mut rng_after = rng_before.clone();
        let direction = Vec3::new(0.0, 0.0, 1.0);
        assert_eq!(
            before_model.sample(ev(energy), direction, &mut rng_before),
            after_model.sample(ev(energy), direction, &mut rng_after)
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn verified_option2_rejects_an_unlisted_model_input_even_when_it_exists_on_disk() {
    let Some(dir) = data() else { return };
    let text = std::fs::read_to_string(dir.root().join("MANIFEST.toml")).unwrap();
    // Omit the first file, Champion's total cross section, but keep its file on disk.
    let first = text.find("[[file]]").unwrap();
    let second =
        first + "[[file]]".len() + text[first + "[[file]]".len()..].find("[[file]]").unwrap();
    let manifest =
        Manifest::from_toml_str(&format!("{}{}", &text[..first], &text[second..])).unwrap();
    let snapshot = VerifiedDataSet::load(&dir, &manifest).unwrap();
    assert!(matches!(
        VerifiedProcessTable::option2_electron(&snapshot, ElasticRotation::Geant4),
        Err(PhysicsError::Data(g4_data::DataError::Core(DnaError::InvalidManifest(message))))
            if message.contains(DnaFile::ChampionElasticE.relative_path())
    ));
}
