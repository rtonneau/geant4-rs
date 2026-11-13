//! RR-008 ordinary installed-library trace. Not a stochastic timing gate.
mod syn_support;
use g4_core::vec3::Vec3;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
const HEADER: &str = "channel,orientation,trial,outcome,a_nm,d1_nm2_ps,d2_nm2_ps,p1x,p1y,p1z,p2x,p2y,p2z,event_time_ps,event_step_ps,r1x,r1y,r1z,r2x,r2y,r2z,products,px,py,pz,parents_removed";
const SEEDS: [u64; 2] = [20261109091, 20261109092];
const TRIALS: usize = 32;
fn number(s: &str) -> f64 {
    let x: f64 = s.parse().expect("invalid number");
    assert!(x.is_finite());
    x
}
fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 + 1e-12 * expected.abs(),
        "actual={actual} expected={expected}"
    );
}
fn vector(row: &[&str], start: usize) -> Vec3 {
    Vec3::new(
        number(row[start]),
        number(row[start + 1]),
        number(row[start + 2]),
    )
}
fn same(actual: Vec3, expected: Vec3) {
    for (a, b) in syn_support::flat(actual)
        .into_iter()
        .zip(syn_support::flat(expected))
    {
        close(a, b);
    }
}
fn verify(csv: &str, seed: u64) -> usize {
    assert!(csv.len() < 1000000);
    let lines: Vec<_> = csv.lines().collect();
    assert_eq!(lines.len(), 292);
    assert_eq!(
        lines[0],
        "# schema: syn_scheduler_v1; Geant4: 1141; claim: ordinary-library first-reaction trace"
    );
    assert_eq!(
        lines[1],
        format!("# seed: {seed}; trials: 32; birth_ps: 1; horizon_ps: 1000000")
    );
    assert_eq!(lines[2], HEADER);
    assert_eq!(
        lines[291],
        format!("# complete: true; seed: {seed}; trials: 32; rows: 288")
    );
    let network = syn_support::network();
    let mut index = 3;
    let mut observed = 0;
    for id in [1, 2, 11] {
        for orientation in 0..3 {
            let reaction = network.get(id).unwrap();
            let a = reaction.pair().unwrap().effective_radius().nm();
            let points = syn_support::positions(a, 3.0, orientation);
            let d = reaction.reactant_diffusivities().map(|d| d.nm2_per_ps());
            let mut coverage = 0;
            for trial in 0..TRIALS {
                let row: Vec<_> = lines[index].split(',').collect();
                index += 1;
                assert_eq!(row.len(), 26);
                assert_eq!(row[0], id.to_string());
                assert_eq!(row[1], orientation.to_string());
                assert_eq!(row[2], trial.to_string());
                for (column, expected) in [(4, a), (5, d[0]), (6, d[1])] {
                    assert!((number(row[column]) - expected).abs() <= 1e-12 * expected.abs());
                }
                same(vector(&row, 7), points[0]);
                same(vector(&row, 10), points[1]);
                match row[3] {
                    "censored" => assert!(row[13..].iter().all(|s| *s == "NA")),
                    "reaction" => {
                        let time = number(row[13]);
                        let step = number(row[14]);
                        assert!(time > 1.0 && time <= 1000000.0 && step > 0.0 && step <= time);
                        same(vector(&row, 15), points[0]);
                        same(vector(&row, 18), points[1]);
                        assert_eq!(row[21], reaction.source().products.len().to_string());
                        assert_eq!(row[25], "1");
                        if reaction.source().products.is_empty() {
                            assert_eq!(&row[22..25], &["NA"; 3]);
                        } else {
                            let site = vector(&row, 22);
                            let delta = points[1] - points[0];
                            let along = (site - points[0]).dot(delta) / delta.dot(delta);
                            let q = d[1].sqrt() / (d[0].sqrt() + d[1].sqrt());
                            assert!(
                                along >= (1.0 - q) / 2.0 - 1e-12
                                    && along <= (2.0 - q) / 2.0 + 1e-12
                            );
                            same(site, points[0] + delta * along);
                        }
                        coverage += 1;
                        observed += 1;
                    }
                    _ => panic!("unknown outcome"),
                }
            }
            assert!(
                coverage > 0,
                "INCONCLUSIVE: no positive-time reaction for channel {id} orientation {orientation}"
            );
        }
    }
    assert_eq!(index, 291);
    observed
}
fn provenance(root: &std::path::Path, csv: &str, replicate: usize, seed: u64) {
    let v: toml::Value = toml::from_str(
        &std::fs::read_to_string(
            root.join(format!("provenance_syn_scheduler_1141_r{replicate}.toml")),
        )
        .expect("RR-008 missing provenance"),
    )
    .unwrap();
    for (key, expected) in [
        ("geant4_version", "11.4.1"),
        ("physics_constructor", "G4EmDNAPhysics_option2"),
        ("chemistry_constructor", "G4EmDNAChemistry_option3"),
        ("timestep_model", "IRT_syn"),
        ("mode", "trace"),
        ("schema", "syn_scheduler_v1"),
        ("claim", "ordinary-library first-reaction trace"),
        ("g4emlow_version", "8.8"),
    ] {
        assert_eq!(v[key].as_str(), Some(expected));
    }
    assert_eq!(v["seed"].as_integer(), Some(seed as i64));
    assert_eq!(v["trials_per_setting"].as_integer(), Some(32));
    assert_eq!(v["completion_rows"].as_integer(), Some(288));
    assert_eq!(v["command"].as_str(),Some(format!("g4rs_syn_scheduler_ref trace ../../reference/syn_scheduler_1141_r{replicate}.csv {seed} 32").as_str()));
    assert_eq!(
        v["csv_sha256"].as_str(),
        Some(format!("{:x}", Sha256::digest(csv.as_bytes())).as_str())
    );
    for key in [
        "generator_commit",
        "generator_source_sha256",
        "compiler",
        "platform",
        "date",
        "actual_command",
        "compile_definitions",
        "g4emlow_md5",
        "source_hashes",
    ] {
        assert!(!v[key].as_str().unwrap().is_empty());
    }
    assert!(v["generator_dirty"].as_bool().is_some());
}
#[test]
#[ignore = "RR-008 requires both complete ordinary-library trace fixtures/provenance"]
fn syn_ordinary_scheduler_preserves_parent_sites() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../validation/reference");
    for (rep, seed) in SEEDS.into_iter().enumerate() {
        let csv =
            std::fs::read_to_string(root.join(format!("syn_scheduler_1141_r{}.csv", rep + 1)))
                .expect("RR-008 missing CSV")
                .replace("\r\n", "\n");
        provenance(&root, &csv, rep + 1, seed);
        let events = verify(&csv, seed);
        println!(
            "replicate={} rows=288 positive-time parent-site/product-support/removal traces={events} PASS",
            rep + 1
        );
    }
}
#[cfg(test)]
fn fixture() -> String {
    let mut s = format!(
        "# schema: syn_scheduler_v1; Geant4: 1141; claim: ordinary-library first-reaction trace\n# seed: {}; trials: 32; birth_ps: 1; horizon_ps: 1000000\n{HEADER}\n",
        SEEDS[0]
    );
    let network = syn_support::network();
    for id in [1, 2, 11] {
        for orientation in 0..3 {
            for trial in 0..TRIALS {
                let r = network.get(id).unwrap();
                let a = r.pair().unwrap().effective_radius().nm();
                let points = syn_support::positions(a, 3.0, orientation);
                let d = r.reactant_diffusivities().map(|d| d.nm2_per_ps());
                let mut row = vec![
                    id.to_string(),
                    orientation.to_string(),
                    trial.to_string(),
                    "reaction".into(),
                    a.to_string(),
                    d[0].to_string(),
                    d[1].to_string(),
                ];
                for p in points {
                    row.extend(syn_support::flat(p).map(|x| x.to_string()));
                }
                row.extend(["2".into(), "1".into()]);
                for p in points {
                    row.extend(syn_support::flat(p).map(|x| x.to_string()));
                }
                row.push(r.source().products.len().to_string());
                if r.source().products.is_empty() {
                    row.extend(["NA".into(), "NA".into(), "NA".into()]);
                } else {
                    let q = d[1].sqrt() / (d[0].sqrt() + d[1].sqrt());
                    let site = points[0] + (points[1] - points[0]) * ((1.5 - q) / 2.0);
                    row.extend(syn_support::flat(site).map(|x| x.to_string()));
                }
                row.push("1".into());
                s.push_str(&row.join(","));
                s.push('\n');
            }
        }
    }
    s.push_str(&format!(
        "# complete: true; seed: {}; trials: 32; rows: 288\n",
        SEEDS[0]
    ));
    s
}
#[test]
fn trace_parser_checks_complete_ordered_positive_time_cases() {
    assert_eq!(verify(&fixture(), SEEDS[0]), 288);
}

#[test]
fn censoring_is_explicit_and_missing_event_coverage_is_inconclusive() {
    let original = fixture();
    let mut lines: Vec<String> = original.lines().map(str::to_string).collect();
    for trial in 0..TRIALS {
        let mut row: Vec<String> = lines[3 + trial].split(',').map(str::to_string).collect();
        row[3] = "censored".into();
        row[13..].fill("NA".into());
        lines[3 + trial] = row.join(",");
        if trial == 0 {
            assert_eq!(verify(&lines.join("\n"), SEEDS[0]), 287);
        }
    }
    assert!(std::panic::catch_unwind(|| verify(&lines.join("\n"), SEEDS[0])).is_err());
}
#[test]
fn trace_parser_rejects_parent_motion_missing_row_and_wrong_removal() {
    let original = fixture();
    for column in [15, 25] {
        let mut lines: Vec<String> = original.lines().map(str::to_string).collect();
        let mut row: Vec<String> = lines[3].split(',').map(str::to_string).collect();
        row[column] = if column == 15 { "1" } else { "0" }.into();
        lines[3] = row.join(",");
        assert!(std::panic::catch_unwind(|| verify(&lines.join("\n"), SEEDS[0])).is_err());
    }
    let mut lines: Vec<_> = original.lines().collect();
    lines.remove(3);
    assert!(std::panic::catch_unwind(|| verify(&lines.join("\n"), SEEDS[0])).is_err());
}
