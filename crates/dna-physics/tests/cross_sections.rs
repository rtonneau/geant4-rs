//! V2/V3-style checks of the macroscopic cross sections against values computed by hand.
//!
//! Reference values come from `scripts/hand_cross_sections.py`, an independent Python
//! re-implementation of the C++ algorithm (`G4DNACrossSectionDataSet::LoadData` `:83-240`,
//! `G4EMDataSet::FindValue` `:167-183`, `G4LogLogInterpolation::Calculate` `:95-127`, model scale
//! factors `G4DNAChampionElasticModel.cc:129`, `G4DNABornExcitationModel1.cc:112`,
//! `G4DNABornIonisationModel1.cc:116`, the water density of `G4DNAMolecularMaterial.cc:197-252`).
//! The Rust output was never used to produce them. Tolerance fixed beforehand: 1e-9 relative
//! (docs/07 V2: nodes 1e-9; the same log-log law applies off-node, so the same tolerance holds).

mod common;

use approx::assert_relative_eq;
use common::{data, ev, raw_rows};
use dna_physics::models::{
    BornIonisation, ChampionElastic, CrossSectionModel, ElasticRotation, ProcessKind,
};
use dna_physics::table::ProcessTable;
use dna_physics::water::WATER_MOLECULES_PER_NM3;

/// `(E eV, Sigma_elastic, Sigma_excitation, Sigma_ionisation, Sigma_vibrational)` in 1/nm, pasted
/// from `python3 scripts/hand_cross_sections.py`. (Sanche is open up to and including 100 eV.)
const HAND: [(f64, f64, f64, f64, f64); 3] = [
    (
        100.0,
        0.8141190997411056,
        0.0223282710194591,
        0.8023861553842268,
        0.2861434739649086,
    ),
    (
        4321.0,
        0.04782758037698331,
        0.0011415633581586882,
        0.07620042629855338,
        0.0,
    ),
    (
        250000.0,
        0.0008298780439745434,
        5.037236935827414e-05,
        0.0037333363471253682,
        0.0,
    ),
];

/// Born ionisation partial cross sections (nm^2) at the same energies (hand computed).
const HAND_IONISATION_PARTIALS: [(f64, [f64; 5]); 3] = [
    (
        100.0,
        [
            0.01245983547711637,
            0.0080300418785522,
            0.0024227738558181273,
            0.0010907810349985046,
            2.991325157044571e-305,
        ],
    ),
    (
        4321.0,
        [
            0.0009107431353375955,
            0.0007477468360782302,
            0.0003886589829234078,
            0.00022206650066805106,
            1.0325084840084565e-05,
        ],
    ),
    (
        250000.0,
        [
            4.177059062329583e-05,
            3.7386450833204784e-05,
            2.042624705591823e-05,
            1.135945554363009e-05,
            7.402473069472842e-07,
        ],
    ),
];

#[test]
fn macroscopic_cross_sections_equal_the_hand_computation() {
    let Some(dir) = data() else { return };
    let table = ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap();
    assert_relative_eq!(
        table.molecules_per_nm3(),
        33.427975930472165,
        max_relative = 1e-14
    );
    for (e, el, ex, io, vib) in HAND {
        let xs = table.macroscopic(ev(e)).unwrap();
        if vib == 0.0 {
            assert_eq!(xs.get(ProcessKind::VibrationalExcitation), 0.0);
        } else {
            assert_relative_eq!(
                xs.get(ProcessKind::VibrationalExcitation),
                vib,
                max_relative = 1e-9
            );
        }
        assert_eq!(xs.get(ProcessKind::Attachment), 0.0);
        assert_relative_eq!(xs.get(ProcessKind::Elastic), el, max_relative = 1e-9);
        assert_relative_eq!(xs.get(ProcessKind::Excitation), ex, max_relative = 1e-9);
        assert_relative_eq!(xs.get(ProcessKind::Ionisation), io, max_relative = 1e-9);
        assert_relative_eq!(xs.total(), el + ex + io + vib, max_relative = 1e-9);
    }
}

#[test]
fn ionisation_partial_cross_sections_equal_the_hand_computation() {
    let Some(dir) = data() else { return };
    let model = BornIonisation::from_data_dir(&dir).unwrap();
    for (e, partials) in HAND_IONISATION_PARTIALS {
        let got = model.partial_cross_sections(ev(e)).unwrap();
        for (g, h) in got.iter().zip(partials) {
            assert_relative_eq!(*g, h, max_relative = 1e-9, epsilon = 0.0);
        }
    }
}

/// At a tabulated energy the per-molecule cross section is the file value times the model scale
/// (literals read straight from the raw files, no interpolation involved).
#[test]
fn cross_sections_at_table_nodes_are_the_file_values_times_the_scale() {
    let Some(dir) = data() else { return };
    let champion = ChampionElastic::from_data_dir(&dir, ElasticRotation::Geant4).unwrap();
    let ion = BornIonisation::from_data_dir(&dir).unwrap();
    let cm2_to_nm2 = 1e14;
    let m2_to_nm2 = 1e18;
    let champion_rows = raw_rows("sigma_elastic_e_champion.dat");
    for r in champion_rows
        .iter()
        .filter(|r| (7.4..=1.0e6).contains(&r[0]))
    {
        assert_relative_eq!(
            champion.cross_section(ev(r[0])).unwrap().nm2(),
            r[1] * 1e-16 * cm2_to_nm2,
            max_relative = 1e-12
        );
    }
    for r in raw_rows("sigma_ionisation_e_born.dat")
        .iter()
        .filter(|r| (11.0..=1.0e6).contains(&r[0]))
    {
        let want: f64 = r[1..].iter().sum::<f64>() * (1e-22 / 3.343) * m2_to_nm2;
        assert_relative_eq!(
            ion.cross_section(ev(r[0])).unwrap().nm2(),
            want,
            max_relative = 1e-12
        );
    }
}

/// The density constant is the one derived in `water.rs` (see also its own hand check).
#[test]
fn water_density_enters_as_molecules_per_nm3() {
    assert_relative_eq!(
        WATER_MOLECULES_PER_NM3,
        33.427975930472165,
        max_relative = 1e-14
    );
}
