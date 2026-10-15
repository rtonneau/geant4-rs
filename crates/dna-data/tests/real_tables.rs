//! V1/V2 infrastructure tests on the committed `data/dna` files (G4EMLOW 8.8).
//!
//! Expected numbers are literals copied from the data files (or computed by hand from literals
//! with the Geant4 formula, see the comments); none is produced by the code under test.

use approx::assert_relative_eq;
use dna_core::error::DnaError;
use dna_core::manifest::Manifest;
use dna_core::units::{Area, Energy};
use dna_data::DataError;
use dna_data::blocks::{AngularCdfTable, BlockTable, IonisationCdfTable};
use dna_data::catalog::{DnaFile, Format, Table};
use dna_data::datadir::{DataDir, repo_data_dir};
use dna_data::xs::{CrossSectionTable, Interpolation};
use std::collections::HashSet;
use std::sync::Arc;

/// Skip (with a visible message) when the committed data subset is not present, e.g. in a
/// source-only checkout. Tests that do run still `unwrap` every load, so a present but corrupt
/// data set fails loudly.
macro_rules! require_data {
    () => {
        let root = repo_data_dir();
        if !root.join("dna").is_dir() || !root.join("MANIFEST.toml").is_file() {
            eprintln!(
                "SKIPPED: Geant4 data not found at {} (expected dna/ and MANIFEST.toml); \
                 run scripts/import_data.py or use the full repository checkout",
                root.display()
            );
            return;
        }
    };
}

fn data() -> DataDir {
    DataDir::new(repo_data_dir()).unwrap()
}

fn manifest() -> Manifest {
    Manifest::load(&repo_data_dir().join("MANIFEST.toml")).unwrap()
}

fn xs(file: DnaFile) -> CrossSectionTable {
    match data().load(file).unwrap() {
        Table::CrossSection(t) => t,
        _ => panic!("{file:?} is not a cross-section table"),
    }
}

fn ev(e: f64) -> Energy {
    Energy::from_ev(e)
}

#[test]
fn catalog_lists_exactly_the_manifest_files() {
    require_data!();
    let catalog: HashSet<&str> = DnaFile::ALL.iter().map(|f| f.relative_path()).collect();
    let manifest = manifest();
    let listed: HashSet<&str> = manifest.files().iter().map(|f| f.path.as_str()).collect();
    assert_eq!(DnaFile::ALL.len(), 20);
    assert_eq!(catalog.len(), 20);
    assert_eq!(catalog, listed);
}

#[test]
fn all_20_files_load_after_checksum_verification() {
    require_data!();
    let (dir, manifest) = (data(), manifest());
    dir.verify(&manifest).unwrap();
    for &file in DnaFile::ALL {
        let table = dir.load_verified(file, &manifest).unwrap();
        match (&table, file.format()) {
            (Table::CrossSection(t), Format::CrossSection(spec)) => {
                assert_eq!(t.n_components(), file.n_components(), "{file:?}");
                assert_eq!(t.interpolation(), spec.interpolation);
            }
            (Table::AngularCdf(_), Format::AngularCdf) => {}
            (Table::IonisationCdf(_), Format::IonisationCdf) => {}
            (Table::IonisationDcs(_), Format::IonisationDcs) => {}
            _ => panic!("{file:?}: table kind does not match its catalog format"),
        }
        assert!(
            !file.consumers().is_empty() && !file.columns().is_empty(),
            "{file:?}"
        );
    }
}

#[test]
fn row_counts_match_the_files() {
    require_data!();
    // Counts observed with `wc -l` (docs/10); the last Sanche line has no trailing newline.
    let rows = |f| xs(f).n_nodes();
    assert_eq!(rows(DnaFile::MeltonAttachmentE), 14);
    assert_eq!(rows(DnaFile::ChampionElasticE), 101);
    assert_eq!(rows(DnaFile::HtranElasticProton), 50);
    assert_eq!(rows(DnaFile::BornExcitationE), 83);
    assert_eq!(rows(DnaFile::BornIonisationE), 83);
    assert_eq!(rows(DnaFile::SancheVibrationalE), 38);
    assert_eq!(rows(DnaFile::RuddIonisationP), 645);
    assert_eq!(rows(DnaFile::RuddIonisationH), 600);
    let blocks = |f: DnaFile| match data().load(f).unwrap() {
        Table::AngularCdf(t) => t.energies_ev().len(),
        Table::IonisationCdf(t) => t.energies_ev().len(),
        Table::IonisationDcs(t) => t.blocks().energies_ev().len(),
        Table::CrossSection(_) => unreachable!(),
    };
    assert_eq!(blocks(DnaFile::ChampionElasticCdfE), 101);
    assert_eq!(blocks(DnaFile::HtranElasticCdfProton), 50);
    assert_eq!(blocks(DnaFile::BornIonisationCdfE), 83);
    assert_eq!(blocks(DnaFile::BornIonisationCdfP), 31);
    assert_eq!(blocks(DnaFile::RpwbaIonisationCdfP), 40);
    assert_eq!(blocks(DnaFile::BornIonisationDcsE), 83);
}

/// The clamp of `G4EMDataSet::FindValue` must be unreachable inside a model's own window; the
/// only exception is Melton (docs/02 B.7.1).
#[test]
fn tables_cover_their_consumers_windows_except_melton() {
    require_data!();
    for &file in DnaFile::ALL {
        let range = data().load(file).unwrap().energy_range();
        for c in file.consumers() {
            let Some((lo, hi)) = c.window_ev else {
                continue;
            };
            let covered = range.min().ev() <= lo && hi <= range.max().ev();
            if file == DnaFile::MeltonAttachmentE {
                assert!(
                    !covered,
                    "Melton table now covers the window: update docs/02 B.7.1"
                );
                assert_eq!((range.min().ev(), range.max().ev()), (0.0, 12.3));
            } else {
                assert!(
                    covered,
                    "{file:?}: table [{}, {}] eV does not cover {} window [{lo}, {hi}]",
                    range.min().ev(),
                    range.max().ev(),
                    c.model
                );
            }
        }
    }
}

#[test]
fn champion_elastic_nodes_scale_and_hand_checked_interpolation() {
    require_data!();
    let t = xs(DnaFile::ChampionElasticE);
    let cm2 = |e: f64| t.total(ev(e)).unwrap().cm2();
    // Literal rows 1, 2, 51 and 101 of sigma_elastic_e_champion.dat, scale 1e-16 cm^2.
    assert_relative_eq!(
        cm2(7.4),
        13.8523650336866e-16,
        epsilon = 0.0,
        max_relative = 1e-14
    );
    assert_relative_eq!(
        cm2(8.52195825627734),
        13.2333694817000e-16,
        epsilon = 0.0,
        max_relative = 1e-12
    );
    assert_relative_eq!(
        cm2(8602.32526704263),
        7.199444599554865e-2 * 1e-16,
        epsilon = 0.0,
        max_relative = 1e-12
    );
    assert_relative_eq!(
        cm2(1.0e7),
        6.207085133577258e-5 * 1e-16,
        epsilon = 0.0,
        max_relative = 1e-14
    );
    // Log-log at the log-midpoint of (7.4, 8.52195825627734) is the geometric mean of the values.
    let e_mid = (7.4f64 * 8.52195825627734).sqrt();
    let expected = (13.8523650336866f64 * 13.2333694817000).sqrt() * 1e-16;
    assert_relative_eq!(cm2(e_mid), expected, epsilon = 0.0, max_relative = 1e-12);
    // Same in the internal unit.
    assert_relative_eq!(
        t.total(ev(e_mid)).unwrap().nm2(),
        expected * 1e14,
        epsilon = 0.0,
        max_relative = 1e-12
    );
}

#[test]
fn units_of_the_other_scales_are_applied() {
    require_data!();
    // sigma_excitation_e_born: row "9 1.645515e+00 ..." x (1e-22/3.343) m^2.
    let born = xs(DnaFile::BornExcitationE);
    assert_relative_eq!(
        born.component(0, ev(9.0)).unwrap().m2(),
        1.645515e0 * 1.0e-22 / 3.343,
        epsilon = 0.0,
        max_relative = 1e-12
    );
    // sigma_ionisation_p_rudd (CRLF file): first row "1.000000e+02 6.105244e-22 ..." in m^2.
    let rudd = xs(DnaFile::RuddIonisationP);
    assert_relative_eq!(
        rudd.component(0, ev(100.0)).unwrap().m2(),
        6.105244e-22,
        epsilon = 0.0,
        max_relative = 1e-14
    );
    // sigma_ionisation_p_RPWBA: first row "1.0e5 1.995826965e-16 ..." in cm^2.
    let rpwba = xs(DnaFile::RpwbaIonisationP);
    assert_relative_eq!(
        rpwba.component(0, ev(1.0e5)).unwrap().cm2(),
        1.995826965e-16,
        epsilon = 0.0,
        max_relative = 1e-14
    );
    // HTran: first row "1.00E+02 2.48746E+01" x 1e-16 cm^2.
    let htran = xs(DnaFile::HtranElasticProton);
    assert_relative_eq!(
        htran.total(ev(100.0)).unwrap().cm2(),
        24.8746e-16,
        epsilon = 0.0,
        max_relative = 1e-14
    );
    // Melton: row "5.8 0.25" x 1e-18 cm^2 (an interior node).
    let melton = xs(DnaFile::MeltonAttachmentE);
    assert_relative_eq!(
        melton.total(ev(5.8)).unwrap().cm2(),
        0.25e-18,
        epsilon = 0.0,
        max_relative = 1e-12
    );
}

#[test]
fn threshold_zeros_follow_the_geant4_floor() {
    require_data!();
    // sigma_ionisation_e_born: shell 0 is 0 at 8, 9, 10 eV and 3.108911 (x scale) at 11 eV.
    let t = xs(DnaFile::BornIonisationE);
    let scale = Area::from_m2(1.0e-22 / 3.343).nm2();
    assert_eq!(t.component(0, ev(8.0)).unwrap().nm2(), 0.0); // first node: stored value
    assert!(t.component(0, ev(10.0)).unwrap().nm2() < 1e-290 * scale.max(1.0)); // interior zero
    assert_relative_eq!(
        t.component(0, ev(11.0)).unwrap().nm2(),
        3.108911 * scale,
        epsilon = 0.0,
        max_relative = 1e-12
    );
    // Hand value: log10 v = -300 + (log10(3.108911) + 300) f, f = ln(10.5/10)/ln(11/10).
    let f = (10.5f64 / 10.0).ln() / (11.0f64 / 10.0).ln();
    let expected = 10f64.powf(-300.0 + (3.108911f64.log10() + 300.0) * f) * scale;
    assert_relative_eq!(
        t.component(0, ev(10.5)).unwrap().nm2(),
        expected,
        epsilon = 0.0,
        max_relative = 1e-9
    );
    assert!(t.component(0, ev(10.5)).unwrap().nm2() < 1e-100);
}

#[test]
fn sanche_is_linear_in_energy() {
    require_data!();
    // Rows: E=1.7 -> 0 .203 .12 .485 .017 .068 .002 .005 0 (sum 0.9);
    //       E=3.2 -> 0 .146 .08 .295 .017 .048 .004 .011 .006 (sum 0.607); unit 1e-16 cm^2.
    let t = xs(DnaFile::SancheVibrationalE);
    assert_eq!(t.interpolation(), Interpolation::LinLin);
    let e = (1.7 + 3.2) / 2.0;
    assert_relative_eq!(
        t.component(1, ev(e)).unwrap().cm2(),
        0.1745e-16,
        epsilon = 0.0,
        max_relative = 1e-12
    );
    assert_relative_eq!(
        t.total(ev(e)).unwrap().cm2(),
        0.7535e-16,
        epsilon = 0.0,
        max_relative = 1e-12
    );
    // Nodes, and the 100 eV end of the table (1e-12 protection).
    assert_relative_eq!(
        t.total(ev(1.7)).unwrap().cm2(),
        0.9e-16,
        epsilon = 0.0,
        max_relative = 1e-12
    );
    assert!(t.total(ev(100.0)).unwrap().cm2() > 0.0);
    assert!(t.total(ev(100.0 + 1e-9)).is_err());
}

#[test]
fn out_of_range_energies_are_typed_errors_on_real_tables() {
    require_data!();
    let melton = xs(DnaFile::MeltonAttachmentE);
    // 12.3 eV is the last node (value 0); 12.5 eV is inside the model window but off the table.
    assert_eq!(melton.total(ev(12.3)).unwrap().nm2(), 0.0);
    assert_eq!(
        melton.total(ev(12.5)).unwrap_err(),
        DataError::Core(DnaError::UnsupportedEnergy {
            energy_ev: 12.5,
            min_ev: 0.0,
            max_ev: 12.3
        })
    );
    let born = xs(DnaFile::BornExcitationE);
    assert!(born.component(0, ev(7.999)).is_err());
    assert!(born.component(0, ev(8.0)).is_ok());
    assert!(born.component(0, ev(1.0e6)).is_ok());
    assert!(born.component(0, ev(1.0e6 * (1.0 + 1e-9))).is_err());
}

#[test]
fn champion_angle_hand_checked() {
    require_data!();
    let t = match data().load(DnaFile::ChampionElasticCdfE).unwrap() {
        Table::AngularCdf(t) => t,
        _ => unreachable!(),
    };
    // Block 7.4 eV has rows (P, theta): (0.02236762407, 10), (0.02702708771, 11);
    // block 8.521958256 eV has (0.02211842486, 10), (0.02672535366, 11).
    // p = 0.025 -> theta1 = 10 + (0.025-0.02236762407)/(0.02702708771-0.02236762407)
    //            -> theta2 = 10 + (0.025-0.02211842486)/(0.02672535366-0.02211842486).
    let theta1 = 10.564952563939313;
    let theta2 = 10.62548723131992;
    assert_relative_eq!(
        t.angle_deg(ev(7.4), 0.025).unwrap(),
        theta1,
        epsilon = 0.0,
        max_relative = 1e-12
    );
    assert_relative_eq!(
        t.angle_deg(ev(8.521958256), 0.025).unwrap(),
        theta2,
        epsilon = 0.0,
        max_relative = 1e-9
    );
    let e_mid = (7.4 + 8.521958256) / 2.0;
    assert_relative_eq!(
        t.angle_deg(ev(e_mid), 0.025).unwrap(),
        10.595219897629615,
        epsilon = 0.0,
        max_relative = 1e-12
    );
    // First and last probability nodes of block 7.4 eV: (0, 0) and (1, 180).
    assert_eq!(t.angle_deg(ev(7.4), 0.0).unwrap(), 0.0);
    assert_relative_eq!(
        t.angle_deg(ev(7.4), 1.0).unwrap(),
        180.0,
        epsilon = 0.0,
        max_relative = 1e-12
    );
    assert!(t.angle_deg(ev(7.399), 0.5).is_err());
    assert!(t.angle_deg(ev(1.0e7 * 1.0001), 0.5).is_err());
}

#[test]
fn htran_first_node_is_not_zero_degrees() {
    require_data!();
    // First row of sigmadiff_cumulated_elastic_proton_HTran.dat: "1.0E+02 0.0 5.8455811895E-01".
    let t = match data().load(DnaFile::HtranElasticCdfProton).unwrap() {
        Table::AngularCdf(t) => t,
        _ => unreachable!(),
    };
    assert_eq!(t.angle_deg(ev(100.0), 0.0).unwrap(), 0.584_558_118_95);
}

fn born_cdf(file: DnaFile) -> IonisationCdfTable {
    match data().load(file).unwrap() {
        Table::IonisationCdf(t) => t,
        _ => unreachable!(),
    }
}

#[test]
fn born_transferred_energy_hand_checked() {
    require_data!();
    let t = born_cdf(DnaFile::BornIonisationCdfE);
    // Shell 0, p = 0.5. Rows (W, P): E = 900: (22.60, 0.4868954260), (23.12, 0.5078790618);
    //                                E = 1000: (22.60, 0.4875120886), (23.12, 0.5085658883).
    // Per block W = 10^(log 22.6 + (log 23.12 - log 22.6) (0.5 - P1)/(P2 - P1)); then log-lin in E.
    let w900: f64 = 22.923359044568638;
    let w1000: f64 = 22.907005987671976;
    assert_relative_eq!(
        t.transferred_energy_ev(0, ev(900.0), 0.5).unwrap(),
        w900,
        epsilon = 0.0,
        max_relative = 1e-12
    );
    assert_relative_eq!(
        t.transferred_energy_ev(0, ev(950.0), 0.5).unwrap(),
        22.915181057357774,
        epsilon = 0.0,
        max_relative = 1e-12
    );
    let mid = 10f64.powf(w900.log10() + (w1000.log10() - w900.log10()) * 0.5);
    assert_relative_eq!(
        t.transferred_energy_ev(0, ev(950.0), 0.5).unwrap(),
        mid,
        epsilon = 0.0,
        max_relative = 1e-12
    );
}

#[test]
fn born_k_shell_closed_below_threshold_scales_from_zero() {
    require_data!();
    // Shell 4: all P = 0 at 539.7 eV (closed), open at 545 eV with rows (W, P):
    // (540.9963, 0.4722683139), (541.0960, 0.5207149604). p = 0.5, E = 542.35 (about half-way).
    let t = born_cdf(DnaFile::BornIonisationCdfE);
    let w545 = 541.0533677307855;
    let frac = (542.35 - 539.7) / (545.0 - 539.7);
    let expected = w545 * frac;
    assert_relative_eq!(
        t.transferred_energy_ev(4, ev(542.35), 0.5).unwrap(),
        expected,
        epsilon = 0.0,
        max_relative = 1e-9
    );
    // Well below threshold the shell is closed in both blocks: Geant4 reads past the vector.
    assert!(matches!(
        t.transferred_energy_ev(4, ev(450.0 + 10.0), 0.5),
        Err(DataError::UndefinedLookup(_))
    ));
}

#[test]
fn every_cumulative_table_is_monotone() {
    require_data!();
    for &file in DnaFile::ALL {
        match data().load(file).unwrap() {
            Table::AngularCdf(_) | Table::IonisationCdf(_) => {}
            _ => continue,
        }
        // Re-read the raw blocks: each cumulative column must be non-decreasing in its block and
        // the key column strictly increasing (the constructors enforce it; assert it from outside).
        let text = std::fs::read_to_string(data().path(file)).unwrap();
        let n_val = file.n_components();
        let bt = BlockTable::from_text(file.relative_path(), &text, n_val).unwrap();
        for (b, block) in bt.blocks().iter().enumerate() {
            assert!(
                block.keys().windows(2).all(|w| w[1] > w[0]),
                "{file:?} block {b}"
            );
            for j in 0..n_val {
                let col = block.column(j);
                assert!(
                    col.windows(2).all(|w| w[1] >= w[0]),
                    "{file:?} block {b} col {j}"
                );
            }
        }
    }
}

#[test]
fn real_angular_cdfs_give_monotone_angles() {
    require_data!();
    for (file, e0, e1) in [
        // (table, lowest, highest incident energy in eV)
        (DnaFile::ChampionElasticCdfE, 7.4f64, 1.0e6f64),
        (DnaFile::HtranElasticCdfProton, 100.0, 1.0e6),
    ] {
        let Table::AngularCdf(t) = data().load(file).unwrap() else {
            unreachable!()
        };
        for i in 0..=50 {
            let e = e0 * (e1 / e0).powf(i as f64 / 50.0);
            let mut prev = -1.0;
            for k in 0..=1000 {
                let a = t.angle_deg(ev(e), k as f64 / 1000.0).unwrap();
                assert!(
                    a >= prev - 1e-9,
                    "{file:?} E={e} p={}: {prev} -> {a}",
                    k as f64 / 1000.0
                );
                assert!((0.0..=180.0 + 1e-9).contains(&a));
                prev = a;
            }
        }
    }
}

#[test]
fn crlf_and_lf_versions_of_real_files_give_identical_tables() {
    require_data!();
    let (dir, _) = (data(), ());
    for file in [
        DnaFile::RuddIonisationP,
        DnaFile::MeltonAttachmentE,
        DnaFile::SancheVibrationalE,
    ] {
        let Format::CrossSection(spec) = file.format() else {
            unreachable!()
        };
        let original = std::fs::read_to_string(dir.path(file)).unwrap();
        let lf = original.replace("\r\n", "\n");
        let crlf = lf.replace('\n', "\r\n");
        let a = CrossSectionTable::from_text("lf", &lf, spec).unwrap();
        let b = CrossSectionTable::from_text("crlf", &crlf, spec).unwrap();
        let c = CrossSectionTable::from_text("orig", &original, spec).unwrap();
        for (x, y) in [(&a, &b), (&a, &c)] {
            assert_eq!(x.node_energies_ev(), y.node_energies_ev());
            for comp in 0..x.n_components() {
                assert_eq!(
                    x.node_values_nm2(comp).unwrap(),
                    y.node_values_nm2(comp).unwrap()
                );
            }
        }
    }
    // The committed Rudd proton file really is CRLF, the others LF.
    assert!(
        std::fs::read_to_string(dir.path(DnaFile::RuddIonisationP))
            .unwrap()
            .contains("\r\n")
    );
    assert!(
        !std::fs::read_to_string(dir.path(DnaFile::RuddIonisationH))
            .unwrap()
            .contains('\r')
    );
}

#[test]
fn missing_and_tampered_files_are_detected() {
    require_data!();
    let scratch = std::env::temp_dir().join(format!("dna-data-tamper-{}", std::process::id()));
    std::fs::create_dir_all(scratch.join("dna")).unwrap();
    let dir = DataDir::new(&scratch).unwrap();
    let m = manifest();
    // missing
    assert!(matches!(
        dir.load_verified(DnaFile::MeltonAttachmentE, &m),
        Err(DataError::Core(DnaError::MissingDataFile { .. }))
    ));
    // tampered copy: verified loading refuses it, plain loading would silently accept it
    let src = std::fs::read(data().path(DnaFile::MeltonAttachmentE)).unwrap();
    let mut bad = src.clone();
    bad[0] = b'9';
    std::fs::write(dir.path(DnaFile::MeltonAttachmentE), &bad).unwrap();
    assert!(matches!(
        dir.load_verified(DnaFile::MeltonAttachmentE, &m),
        Err(DataError::Core(DnaError::ChecksumMismatch { .. }))
    ));
    // untampered copy passes
    std::fs::write(dir.path(DnaFile::MeltonAttachmentE), &src).unwrap();
    dir.load_verified(DnaFile::MeltonAttachmentE, &m).unwrap();
    std::fs::remove_dir_all(&scratch).unwrap();
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn tables_are_shareable_read_only_across_threads() {
    require_data!();
    assert_send_sync::<CrossSectionTable>();
    assert_send_sync::<AngularCdfTable>();
    assert_send_sync::<IonisationCdfTable>();
    assert_send_sync::<Table>();

    let table = Arc::new(xs(DnaFile::ChampionElasticE));
    let energies: Vec<f64> = (0..200)
        .map(|i| 7.4 * (1.0e6f64 / 7.4).powf(i as f64 / 199.0))
        .collect();
    let serial: Vec<f64> = energies
        .iter()
        .map(|&e| table.total(ev(e)).unwrap().nm2())
        .collect();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let (t, es) = (Arc::clone(&table), energies.clone());
            std::thread::spawn(move || {
                es.iter()
                    .map(|&e| t.total(ev(e)).unwrap().nm2())
                    .collect::<Vec<f64>>()
            })
        })
        .collect();
    for h in handles {
        assert_eq!(h.join().unwrap(), serial);
    }
}
