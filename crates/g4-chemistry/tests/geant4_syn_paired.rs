//! RR-007 finite common-input mapping diagnostic. No distribution/RNG claim.
mod syn_support;
use g4_chemistry::SpeciesInstance;
use g4_chemistry::molecule::{Molecule, MoleculeId, MoleculePair};
use g4_chemistry::syn::{
    PlacementPrimitives, parent_positions_with_primitives, place_pair_with_primitives,
};
use g4_core::units::{Diffusivity, Time};
use g4_core::vec3::Vec3;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const SEED: u64 = 20261109071;
const TAPES: usize = 16;
const SOURCE_HASH: &str = "8ae87604437c1f7d57f410961c01d51adf9215e5a35f6702247ad6592a1f0559";
const HEADER: &str = "kind,channel,ratio,dt_ps,orientation,fixed,tape,zx,zy,zz,u_phi,u_theta,u_product,mask,status,a_nm,d1_nm2_ps,d2_nm2_ps,p1x,p1y,p1z,p2x,p2y,p2z,r1x,r1y,r1z,r2x,r2y,r2z,products,px,py,pz,killed";
#[derive(Clone, Copy)]
struct Case {
    kind: &'static str,
    id: u16,
    ratio: f64,
    dt: f64,
    orientation: usize,
    fixed: i32,
}
fn cases() -> Vec<Case> {
    let mut cases = vec![];
    for id in [1, 2, 11] {
        for ratio in [1.0, 3.0] {
            for dt in [0.0, 1.0, 1000.0] {
                for orientation in 0..3 {
                    cases.push(Case {
                        kind: "base",
                        id,
                        ratio,
                        dt,
                        orientation,
                        fixed: -1,
                    });
                }
            }
        }
    }
    for fixed in [0, 1] {
        for dt in [0.0, 1.0, 1000.0] {
            for ratio in [3.0, 0.0] {
                cases.push(Case {
                    kind: "immobile",
                    id: 2,
                    ratio,
                    dt,
                    orientation: 0,
                    fixed,
                });
            }
        }
    }
    for id in [1, 2, 11] {
        for dt in [0.0, 1.0, 1000.0] {
            cases.push(Case {
                kind: "coincident",
                id,
                ratio: 0.0,
                dt,
                orientation: 0,
                fixed: -1,
            });
        }
    }
    cases
}
fn primitive(tape: usize) -> PlacementPrimitives {
    let j = ((tape as u64 + SEED % 16) % 16) as i32;
    PlacementPrimitives {
        normal: Vec3::new(
            f64::from(j % 7 - 3) * 0.75,
            f64::from((3 * j + 1) % 7 - 3) * 0.5,
            f64::from((5 * j + 2) % 7 - 3) * 1.25,
        ),
        azimuth: (f64::from(j % 8) + 0.5) / 8.0,
        polar: (f64::from((3 * j + 1) % 8) + 0.5) / 8.0,
        product: (f64::from((5 * j + 2) % 8) + 0.5) / 8.0,
    }
}
fn close(actual: f64, expected: f64) {
    assert!(actual.is_finite() && expected.is_finite());
    assert!(
        (actual - expected).abs() <= 1e-12 + 1e-12 * expected.abs(),
        "actual={actual:.17e} expected={expected:.17e} delta={:.17e}",
        actual - expected
    );
}
fn number(value: &str) -> f64 {
    let n = value.parse::<f64>().expect("invalid numeric field");
    assert!(n.is_finite());
    n
}

fn parameter_close(actual: f64, expected: f64) {
    assert!(actual.is_finite());
    if expected == 0.0 {
        assert_eq!(actual, 0.0);
    } else {
        assert!((actual - expected).abs() <= 1e-12 * expected.abs());
    }
}
fn provenance(root: &std::path::Path, csv: &str) {
    let v: toml::Value = toml::from_str(
        &std::fs::read_to_string(root.join("provenance_syn_paired_1141.toml"))
            .expect("RR-007 missing provenance"),
    )
    .unwrap();
    for (k, value) in [
        ("geant4_version", "11.4.1"),
        ("physics_constructor", "G4EmDNAPhysics_option2"),
        ("chemistry_constructor", "G4EmDNAChemistry_option3"),
        ("timestep_model", "IRT_syn"),
        ("mode", "paired"),
        ("schema", "syn_paired_v1"),
        ("claim", "finite-input mapping only"),
        ("source_sha256", SOURCE_HASH),
        ("g4emlow_version", "8.8"),
        (
            "command",
            "g4rs_syn_ref paired ../../reference/syn_paired_1141.csv 20261109071 16",
        ),
    ] {
        assert_eq!(v[k].as_str(), Some(value), "provenance {k}");
    }
    assert_eq!(v["seed"].as_integer(), Some(SEED as i64));
    assert_eq!(v["tapes_per_setting"].as_integer(), Some(16));
    assert_eq!(v["completion_rows"].as_integer(), Some(1200));
    assert_eq!(
        v["csv_sha256"].as_str(),
        Some(format!("{:x}", Sha256::digest(csv.as_bytes())).as_str())
    );
    for k in [
        "generator_commit",
        "generator_source_sha256",
        "compiler",
        "platform",
        "date",
        "actual_command",
        "compile_definitions",
        "g4emlow_md5",
    ] {
        assert!(!v[k].as_str().unwrap().is_empty());
    }
    assert!(v["generator_dirty"].as_bool().is_some());
}

#[test]
fn paired_manifest_and_rust_draw_masks_are_complete() {
    let network = syn_support::network();
    let mut rows = 0;
    let mut coordinates = 0;
    let mut errors = 0;
    for case in cases() {
        let reaction = network.get(case.id).unwrap();
        let p = *reaction.pair().unwrap();
        let points =
            syn_support::positions(p.effective_radius().nm(), case.ratio, case.orientation);
        for tape in 0..TAPES {
            let input = primitive(tape);
            let mut diffusion = reaction.reactant_diffusivities();
            if case.fixed >= 0 {
                diffusion[case.fixed as usize] = Diffusivity::from_nm2_per_ps(0.0);
            }
            let m = std::array::from_fn::<_, 2, _>(|i| {
                Molecule::new(
                    MoleculeId(i as u64),
                    SpeciesInstance {
                        species: p.reactants()[i],
                        position: points[i],
                        time: Time::from_ps(0.0),
                    },
                )
            });
            if case.kind == "base" {
                let (result, mask) = place_pair_with_primitives(
                    MoleculePair::new(&m[0], &m[1]).unwrap(),
                    reaction,
                    Time::from_ps(case.dt),
                    input,
                );
                let result = result.unwrap();
                let expected = if case.dt > 0.0 { 31 } else { 0 }
                    | if result.products.is_empty() { 0 } else { 32 };
                assert_eq!(mask, expected);
                coordinates += 6 + if result.products.is_empty() { 0 } else { 3 };
            } else {
                let (result, mask) = parent_positions_with_primitives(
                    case.id,
                    points,
                    diffusion,
                    p.effective_radius(),
                    Time::from_ps(case.dt),
                    input,
                );
                assert_eq!(mask, 0);
                if case.kind == "coincident" {
                    assert!(result.is_err());
                    errors += 1;
                } else {
                    assert_eq!(result.unwrap(), [points[case.fixed as usize]; 2]);
                    coordinates += 6;
                }
            }
            rows += 1;
        }
    }
    assert_eq!((rows, coordinates, errors), (1200, 8064, 144));
}

#[test]
#[ignore = "RR-007 requires complete source-mapping fixture and provenance"]
fn syn_paired_mapping_matches_geant4() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../validation/reference");
    let csv = std::fs::read_to_string(root.join("syn_paired_1141.csv"))
        .expect("RR-007 requested")
        .replace("\r\n", "\n");
    assert!(csv.len() < 1_000_000, "fixture budget exceeded");
    provenance(&root, &csv);
    let lines: Vec<_> = csv.lines().collect();
    assert_eq!(lines.len(), 1204);
    assert_eq!(
        lines[0],
        "# schema: syn_paired_v1; Geant4: 1141; claim: finite-input mapping only"
    );
    assert_eq!(
        lines[1],
        format!("# seed: {SEED}; tapes: 16; source_sha256: {SOURCE_HASH}")
    );
    assert_eq!(lines[2], HEADER);
    assert_eq!(
        lines[1203],
        format!("# complete: true; seed: {SEED}; tapes: 16; rows: 1200")
    );
    let network = syn_support::network();
    let mut row = 3;
    let mut coordinates = 0;
    let mut errors = 0;
    for case in cases() {
        for tape in 0..TAPES {
            let columns: Vec<_> = lines[row].split(',').collect();
            row += 1;
            assert_eq!(columns.len(), 35, "row {row}");
            assert_eq!(columns[0], case.kind);
            assert_eq!(number(columns[1]), f64::from(case.id));
            assert_eq!(number(columns[2]), case.ratio);
            assert_eq!(number(columns[3]), case.dt);
            assert_eq!(number(columns[4]), case.orientation as f64);
            assert_eq!(number(columns[5]), f64::from(case.fixed));
            assert_eq!(number(columns[6]), tape as f64);
            let input = primitive(tape);
            for (column, value) in columns[7..13].iter().zip([
                input.normal.x,
                input.normal.y,
                input.normal.z,
                input.azimuth,
                input.polar,
                input.product,
            ]) {
                assert_eq!(number(column), value);
            }
            let reaction = network.get(case.id).unwrap();
            let p = *reaction.pair().unwrap();
            let a = p.effective_radius().nm();
            let points = syn_support::positions(a, case.ratio, case.orientation);
            let mut diffusion = reaction.reactant_diffusivities();
            if case.fixed >= 0 {
                diffusion[case.fixed as usize] = Diffusivity::from_nm2_per_ps(0.0);
            }
            parameter_close(number(columns[15]), a);
            for (i, d) in diffusion.iter().enumerate() {
                parameter_close(number(columns[16 + i]), d.nm2_per_ps());
            }
            for (column, value) in columns[18..24]
                .iter()
                .zip(points.into_iter().flat_map(syn_support::flat))
            {
                close(number(column), value);
            }
            let m = std::array::from_fn::<_, 2, _>(|i| {
                Molecule::new(
                    MoleculeId(i as u64),
                    SpeciesInstance {
                        species: p.reactants()[i],
                        position: points[i],
                        time: Time::from_ps(0.0),
                    },
                )
            });
            let (parents, products, mask) = if case.kind == "base" {
                let (result, mask) = place_pair_with_primitives(
                    MoleculePair::new(&m[0], &m[1]).unwrap(),
                    reaction,
                    Time::from_ps(case.dt),
                    input,
                );
                let result = result.unwrap();
                assert_eq!(number(columns[30]), result.products.len() as f64);
                assert_eq!(columns[34], "1");
                close(
                    (result.parent_positions[0] - result.parent_positions[1]).norm(),
                    if case.dt == 0.0 { case.ratio * a } else { a },
                );
                (Ok(result.parent_positions), Some(result.products), mask)
            } else {
                let (result, mask) = parent_positions_with_primitives(
                    case.id,
                    points,
                    diffusion,
                    p.effective_radius(),
                    Time::from_ps(case.dt),
                    input,
                );
                assert_eq!(columns[30], "NA");
                assert_eq!(columns[34], "NA");
                (result, None, mask)
            };
            assert_eq!(number(columns[13]), f64::from(mask));
            if case.kind == "coincident" {
                assert_eq!(columns[14], "coincident_error");
                assert!(matches!(
                    parents,
                    Err(g4_chemistry::ChemistryError::IrtDomain {
                        operation: "coincident or unrepresentable mobile separation",
                        ..
                    })
                ));
                assert!(columns[24..30].iter().all(|v| *v == "NA"));
                errors += 1;
            } else {
                assert_eq!(columns[14], "ok");
                let parents = parents.unwrap();
                for (column, value) in columns[24..30]
                    .iter()
                    .zip(parents.into_iter().flat_map(syn_support::flat))
                {
                    close(number(column), value);
                    coordinates += 1;
                }
            }
            if let Some(products) = products.filter(|p| !p.is_empty()) {
                for (column, value) in columns[31..34]
                    .iter()
                    .zip(syn_support::flat(products[0].position))
                {
                    close(number(column), value);
                    coordinates += 1;
                }
            } else {
                assert!(columns[31..34].iter().all(|v| *v == "NA"));
            }
        }
    }
    assert_eq!((row, coordinates, errors), (1203, 8064, 144));
    println!("RR-007 PASS: 1200 rows,8064 coordinates,144 expected errors; mapping only");
}
