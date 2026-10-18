//! The Rust data and constants against a machine extraction of the Geant4 C++ text
//! (`scripts/extract_physchem_from_cpp.py` -> `tests/fixtures/cpp_extract.json`) and the independent Python
//! evaluation (`scripts/hand_physchem.py` -> `tests/fixtures/hand_values.json`).
//!
//! Acceptance (fixed before the first run): exact equality for probabilities, energies, diffusion coefficients,
//! charges, radii, orbit indices, product lists and displacement cases (the same decimal literals are parsed on both
//! sides); relative 1e-12 for the Onsager radius, the dielectric constant and the penetration law.

mod common;

use common::{CPP_BOOKKEEPING_KEYS, CPP_SPECIES_KEYS, fixture};
use dna_chemistry::channels::{ChannelTable, StateKind};
use dna_chemistry::displacer::{self, autoionisation_r_mean_nm};
use dna_chemistry::params::PhysChemParams;
use dna_chemistry::reactions::{
    KineticIntent, ObservedRate, ReactionParticipant, ReactionTable, SourceClass,
};
use dna_chemistry::species::{SpeciesTable, atoms};
use dna_core::species::Species;

fn rel_eq(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol * b.abs().max(f64::MIN_POSITIVE)
}

#[test]
fn species_table_equals_the_extraction() {
    let x = fixture("cpp_extract.json");
    let table = SpeciesTable::option3().unwrap();
    let fx = x["species"].as_object().unwrap();
    assert_eq!(fx.len(), 15, "the extraction lists 15 species");
    assert_eq!(Species::ALL.len(), CPP_SPECIES_KEYS.len());
    for (species, &tag) in Species::ALL.iter().zip(&CPP_SPECIES_KEYS) {
        assert_eq!(species.chemistry_name(), tag, "canonical key/order");
        assert_eq!(Species::from_chemistry_name(tag), Some(*species));
        let v = fx
            .get(tag)
            .unwrap_or_else(|| panic!("C++ key {tag} missing"));
        let p = table.get(*species);
        // Geant4 m^2/s -> nm^2/ps: x 1e6
        let d = v["diffusion_m2_s"].as_f64().unwrap() * 1.0e6;
        assert!(
            rel_eq(p.diffusivity.nm2_per_ps(), d, 1e-14),
            "{tag}: D {} vs {d}",
            p.diffusivity.nm2_per_ps()
        );
        assert_eq!(
            i64::from(p.charge),
            v["charge"].as_i64().unwrap(),
            "{tag}: charge"
        );
        assert_eq!(
            p.radius.nm(),
            v["vdw_radius_nm"].as_f64().unwrap(),
            "{tag}: radius"
        );
    }
    for tag in CPP_BOOKKEEPING_KEYS {
        assert!(!fx.contains_key(tag), "{tag} is bookkeeping, not Species");
        assert_eq!(Species::from_chemistry_name(tag), None);
    }
    for alias in ["OH", "HO2"] {
        assert!(
            !fx.contains_key(alias),
            "{alias} is not a C++ configuration key"
        );
        assert_eq!(Species::from_chemistry_name(alias), None);
    }
}

#[test]
fn channel_table_equals_the_extraction() {
    let x = fixture("cpp_extract.json");
    let species = SpeciesTable::option3().unwrap();
    let table = ChannelTable::option3(&species).unwrap();
    let states = x["states"].as_object().unwrap();
    assert_eq!(states.len(), table.states().len());
    let exc = x["excitation_energies_ev"].as_array().unwrap();
    let mut n_channels = 0;
    for (name, v) in states {
        let st = table
            .states()
            .iter()
            .find(|s| &s.name == name)
            .unwrap_or_else(|| panic!("state {name} missing in the Rust table"));
        // electronic configuration: orbit emptied (None for attachment/recombined)
        let vacated: Vec<u64> = v["orbits_vacated"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o.as_u64().unwrap())
            .collect();
        match st.orbit_vacated {
            Some(o) => assert_eq!(vacated, vec![u64::from(o)], "{name}: orbit"),
            None => assert!(vacated.is_empty(), "{name}: orbit"),
        }
        let filled: Vec<u64> = v["orbits_filled"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o.as_u64().unwrap())
            .collect();
        match st.kind {
            StateKind::Excitation => {
                assert_eq!(filled, vec![5], "{name}: an excitation fills orbit 5")
            }
            _ => assert!(
                filled.is_empty() || st.kind == StateKind::Attachment,
                "{name}"
            ),
        }
        let chans = v["channels"].as_array().unwrap();
        assert_eq!(chans.len(), st.channels.len(), "{name}: number of channels");
        for (fc, rc) in chans.iter().zip(&st.channels) {
            n_channels += 1;
            assert_eq!(
                fc["name"].as_str().unwrap(),
                rc.name,
                "{name}: channel name/order"
            );
            assert_eq!(
                fc["probability"].as_f64().unwrap(),
                rc.probability,
                "{}: p",
                rc.name
            );
            assert_eq!(
                fc["energy_ev"].as_f64().unwrap(),
                rc.decay_energy.ev(),
                "{}: energy",
                rc.name
            );
            if let Some(level) = fc.get("energy_from_excitation_level") {
                assert_eq!(
                    exc[level.as_u64().unwrap() as usize].as_f64().unwrap(),
                    rc.decay_energy.ev()
                );
            }
            let prods: Vec<&str> = fc["products"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p.as_str().unwrap())
                .collect();
            let ours: Vec<&str> = rc.products.iter().map(|s| s.chemistry_name()).collect();
            assert_eq!(prods, ours, "{}: products", rc.name);
            match (fc["displacement"].as_str(), rc.displacement.cpp_case()) {
                (Some("NoDisplacement"), None) | (None, None) => {}
                (Some(a), Some(b)) => assert_eq!(a, b, "{}: displacement", rc.name),
                other => panic!("{}: displacement mismatch {other:?}", rc.name),
            }
        }
    }
    assert_eq!(n_channels, 23);
}

#[test]
fn physics_to_orbit_mapping_is_four_minus_index() {
    let x = fixture("cpp_extract.json");
    let species = SpeciesTable::option3().unwrap();
    let table = ChannelTable::option3(&species).unwrap();
    for s in table.states() {
        if matches!(s.kind, StateKind::Excitation | StateKind::Ionisation) {
            let fx = &x["states"][&s.name];
            let vacated = fx["orbits_vacated"][0].as_u64().unwrap();
            assert_eq!(vacated, 4 - u64::from(s.index), "{}", s.name);
        }
    }
}

#[test]
fn displacer_constants_equal_the_extraction() {
    let x = fixture("cpp_extract.json");
    let d = &x["displacer"];
    let rms = |case: &str| -> Vec<f64> {
        d["products"][case]["rms_nm"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect()
    };
    let strs = |case: &str, key: &str| -> Vec<String> {
        d["products"][case][key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect()
    };
    // A1B1: R = 2.4 nm, OH -1/18, H +17/18
    assert_eq!(rms("A1B1_DissociationDecay"), vec![displacer::A1B1_RMS_NM]);
    assert_eq!(
        strs("A1B1_DissociationDecay", "fractions"),
        ["-1.0/18.0", "+17.0/18.0"]
    );
    // B1A1 and dissociative attachment: R = 0.8 nm (H2 -16/18, O recoil +2/18), OH pair 1.1 nm, factors +-0.5
    for case in ["B1A1_DissociationDecay", "DissociativeAttachment"] {
        assert_eq!(
            rms(case),
            vec![displacer::B1A1_AXIS_RMS_NM, displacer::OH_PAIR_RMS_NM],
            "{case}"
        );
        assert_eq!(
            strs(case, "fractions"),
            ["-16.0/18.0", "+2.0/18.0"],
            "{case}"
        );
        assert_eq!(strs(case, "half_factors"), ["0.5", "-0.5"], "{case}");
    }
    // B1A1 2: each H 1.6 nm, factors +-0.5
    assert_eq!(
        rms("B1A1_DissociationDecay2"),
        vec![displacer::B1A1_2_H_RMS_NM]
    );
    assert_eq!(
        strs("B1A1_DissociationDecay2", "half_factors"),
        ["0.5", "-0.5"]
    );
    // Ionisation and auto-ionisation: 50/50 branch, displaced product 0.8 nm
    for case in ["Ionisation_DissociationDecay", "AutoIonisation"] {
        assert_eq!(
            rms(case),
            vec![
                0.0,
                displacer::IONISATION_RMS_NM,
                displacer::IONISATION_RMS_NM,
                0.0
            ],
            "{case}"
        );
        assert_eq!(strs(case, "branch_probability"), ["0.5"], "{case}");
    }
    // mother displacement: 2.0 nm for the two ionising cases, 0 for the others used here
    let mother = |case: &str| d["mother_rms_nm"][case].as_f64().unwrap();
    assert_eq!(
        mother("Ionisation_DissociationDecay"),
        displacer::MOTHER_RMS_NM
    );
    assert_eq!(mother("AutoIonisation"), displacer::MOTHER_RMS_NM);
    for case in [
        "A1B1_DissociationDecay",
        "B1A1_DissociationDecay",
        "B1A1_DissociationDecay2",
        "DissociativeAttachment",
    ] {
        assert_eq!(mother(case), 0.0, "{case}");
    }
    assert_eq!(
        d["radial_distribution_of_products_sigma_per_component"]
            .as_str()
            .unwrap(),
        "Rrms / sqrt(3)"
    );
    // auto-ionisation electron: energy and Meesungnoen2002 penetration
    let params = PhysChemParams::option3().unwrap();
    assert_eq!(
        d["autoionisation_electron_kinetic_energy_ev"]
            .as_f64()
            .unwrap(),
        params.autoionisation_electron_energy.ev()
    );
}

#[test]
fn penetration_law_equals_the_extraction_and_the_hand_value() {
    let x = fixture("cpp_extract.json");
    let h = fixture("hand_values.json");
    let g: Vec<f64> = x["penetration"]["meesungnoen2002_gcoeff"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    assert_eq!(g, dna_core::penetration::MEESUNGNOEN_COEFFICIENTS.to_vec());
    assert_eq!(
        x["penetration"]["sigma_1d_per_rmean"].as_f64().unwrap(),
        dna_core::penetration::SIGMA_1D_PER_RMEAN
    );
    let ke = h["penetration"]["ke_ev"].as_f64().unwrap();
    assert!(rel_eq(
        autoionisation_r_mean_nm(ke),
        h["penetration"]["r_mean_nm"].as_f64().unwrap(),
        1e-12
    ));
}

#[test]
fn recombination_parameters_and_onsager_radius_equal_the_hand_values() {
    let x = fixture("cpp_extract.json");
    let h = fixture("hand_values.json");
    let rc = &x["recombination"];
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/chemistry/physchem_option3.toml"
    ))
    .unwrap();
    let toml: toml::Value = toml::from_str(&text).unwrap();
    let r = &toml["recombination"];
    let arr = |v: &toml::Value| -> Vec<f64> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_float().unwrap())
            .collect()
    };
    let fx = |k: &str| -> Vec<f64> {
        rc[k]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_f64().unwrap())
            .collect()
    };
    assert_eq!(arr(&r["a"]), fx("A"));
    assert_eq!(arr(&r["b"]), fx("B"));
    assert_eq!(arr(&r["s"]), fx("S"));
    assert_eq!(r["y0"].as_float().unwrap(), fx("Y")[0]);
    assert_eq!(
        r["search_radius_in_onsager_radii"].as_float().unwrap(),
        rc["search_radius_in_onsager_radii"].as_f64().unwrap()
    );
    assert_eq!(
        r["temperature_k"].as_float().unwrap(),
        rc["NTP_Temperature_K"].as_f64().unwrap()
    );
    assert_eq!(
        r["density_g_cm3"].as_float().unwrap(),
        rc["G4_WATER_density_g_cm3"].as_f64().unwrap()
    );
    assert_eq!(
        r["k_boltzmann_mev_per_k"].as_float().unwrap().to_string(),
        "0.00000000008617333"
    );
    assert_eq!(rc["clhep"]["k_Boltzmann"].as_str().unwrap(), "8.617333e-11");
    assert_eq!(r["e_si"].as_float().unwrap(), 1.602176634e-19);
    // Onsager radius, dielectric constant and probabilities from the independent Python evaluation
    let rec = PhysChemParams::option3().unwrap().recombination;
    let o = &h["onsager"];
    assert!(rel_eq(
        rec.dielectric_constant,
        o["dielectric_constant"].as_f64().unwrap(),
        1e-12
    ));
    assert!(rel_eq(
        rec.onsager_radius.nm(),
        o["onsager_radius_nm"].as_f64().unwrap(),
        1e-12
    ));
    for (d, p) in o["probabilities_at_distance_nm"].as_object().unwrap() {
        let d: f64 = d.parse().unwrap();
        assert!(
            rel_eq(rec.probability(d), p.as_f64().unwrap(), 1e-12),
            "p({d})"
        );
    }
    // the rest-process ordering that makes the recombination win the tie
    assert_eq!(
        rc["rest_process_ordering"]["dissociation"].as_i64(),
        Some(1)
    );
    assert_eq!(
        rc["rest_process_ordering"]["recombination"].as_i64(),
        Some(2)
    );
}

#[test]
fn every_channel_balances_as_in_the_hand_table() {
    let h = fixture("hand_values.json");
    let species = SpeciesTable::option3().unwrap();
    let table = ChannelTable::option3(&species).unwrap();
    let mut checked = 0;
    for st in table.states() {
        for ch in &st.channels {
            let key = format!("{}/{}", st.name, ch.name);
            let hv = &h["channel_balance"][&key];
            assert!(hv.is_object(), "no hand value for {key}");
            if ch.products.is_empty() {
                assert_eq!(hv["relax"].as_bool(), Some(true), "{key}");
                continue;
            }
            let o: i32 = ch.products.iter().map(|&s| atoms(s).1).sum();
            assert_eq!(
                i64::from(o - 1),
                hv["extra_waters"].as_i64().unwrap(),
                "{key}: extra waters"
            );
            checked += 1;
        }
    }
    // relaxations: A^1B_1, B^1A_1, the three Rydberg layers, H2Ovib = 6
    assert_eq!(checked, 23 - 6, "23 channels, 6 of them relaxations");
}

#[test]
fn reaction_table_equals_the_cpp_extraction() {
    let x = fixture("cpp_reactions.json");
    let table = ReactionTable::option3().unwrap();
    let extracted = x["reactions"].as_array().unwrap();
    assert_eq!(extracted.len(), 72);
    assert_eq!(table.iter().count(), extracted.len());

    let class_name = |class: SourceClass| match class {
        SourceClass::I => "I",
        SourceClass::II => "II",
        SourceClass::III => "III",
        SourceClass::IV => "IV",
        SourceClass::VI => "VI",
        SourceClass::Scavenging => "scavenging",
    };
    let intent_name = |intent: KineticIntent| match intent {
        KineticIntent::TotallyDiffusionControlled => "totally_diffusion_controlled",
        KineticIntent::PartiallyDiffusionControlled => "partially_diffusion_controlled",
        KineticIntent::FirstOrder => "first_order",
        KineticIntent::PseudoFirstOrder => "pseudo_first_order",
    };

    for (reaction, expected) in table.iter().zip(extracted) {
        assert_eq!(u64::from(reaction.id), expected["id"].as_u64().unwrap());
        assert_eq!(
            u64::from(reaction.source_line),
            expected["source_line"].as_u64().unwrap(),
            "reaction {} source line",
            reaction.id
        );
        assert_eq!(class_name(reaction.source_class), expected["source_class"]);
        assert_eq!(
            intent_name(reaction.kinetic_intent),
            expected["kinetic_intent"]
        );
        assert_eq!(reaction.set_reaction_type, expected["set_reaction_type"]);
        let names = |participants: &[ReactionParticipant]| {
            participants
                .iter()
                .map(|participant| participant.chemistry_name())
                .collect::<Vec<_>>()
        };
        assert_eq!(names(&reaction.reactants), expected["reactants"]);
        assert_eq!(names(&reaction.products), expected["products"]);
        match reaction.rate {
            ObservedRate::Bimolecular { m3_per_mol_s } => {
                assert_eq!(expected["rate_order"], "bimolecular");
                assert_eq!(expected["rate_unit"], "m3_per_mol_s");
                assert_eq!(m3_per_mol_s, expected["rate_value"].as_f64().unwrap());
            }
            ObservedRate::FirstOrder { per_s } => {
                assert_eq!(expected["rate_order"], "first_order");
                assert_eq!(expected["rate_unit"], "per_s");
                assert_eq!(per_s, expected["rate_value"].as_f64().unwrap());
            }
        }
    }
}

#[test]
fn reaction_conservation_equals_the_independent_hand_evaluation() {
    let hand = fixture("hand_reactions.json");
    let table = ReactionTable::option3().unwrap();
    let balances = hand["balances"].as_array().unwrap();
    assert_eq!(balances.len(), 72);
    for (reaction, expected) in table.iter().zip(balances) {
        assert_eq!(u64::from(reaction.id), expected["id"].as_u64().unwrap());
        assert_eq!(
            i64::from(reaction.implicit_water),
            expected["implicit_water_products"].as_i64().unwrap()
        );
        assert_eq!(
            reaction.balance_exception,
            !expected["balanced"].as_bool().unwrap()
        );
    }
    assert_eq!(hand["anomaly_ids"], serde_json::json!([53]));
    assert_eq!(
        hand["rate_comment_discrepancy_ids"],
        serde_json::json!([59, 63, 65, 68, 70])
    );
}
