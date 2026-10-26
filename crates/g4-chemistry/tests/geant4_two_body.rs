//! M7 Geant4 11.4.1 H+H two-body gate.
//!
//! The comparison family is the six points tau=[0.25,0.5,1,2,4,16] for
//! option3 reaction id 1, r0=0.2 nm, N=100,000. Rust IRT uses master seed
//! 20_261_023_006; Geant4 SBS uses seed 202_610_231. At every point the
//! acceptance rule is |F_IRT-F_G4| <= 3*sqrt(2*F_V9*(1-F_V9)/N), using the
//! independently fixed V9 probability. This is a pointwise three-sigma rule
//! across the fixed six-point family, not a simultaneous-coverage claim.
//!
//! The Geant4 run occurred before this file was committed, so this header
//! freezes the unchanged V9 rule without claiming retrospective preregistration
//! or tuning to the observed result.

use std::path::PathBuf;

mod common;

use common::{isolated_pair, option3_parameters};
use g4_chemistry::irt::{ReactionTime, sample_independent_reaction_time};
use g4_chemistry::molecule::MoleculePair;
use g4_core::rng::{RngStreams, Stage};

const N: u64 = 100_000;
const REACTION_ID: u16 = 1;
const IRT_SEED: u64 = 20_261_023_006;
const G4_SEED: i64 = 202_610_231;
const TAU: [f64; 6] = [0.25, 0.5, 1.0, 2.0, 4.0, 16.0];
const TIME_SCALE_PS: f64 = 0.197_050_889_166_177_16;
const V9_CDF: [f64; 6] = [
    0.002_220_827_343_701_086,
    0.021_601_957_061_711_33,
    0.074_680_241_949_397_87,
    0.150_648_092_540_691_45,
    0.227_650_131_308_291_4,
    0.343_575_287_428_839_3,
];
const COMMAND: &str = "g4rs_ref two_body_h_h ../../reference/two_body_h_h.csv 202610231 100000";

#[derive(Debug, PartialEq)]
struct Row {
    tau: f64,
    observation_time_ps: f64,
    total_pairs: u64,
    reacted_count: u64,
    surviving_count: u64,
}

fn parse_and_validate_csv(text: &str) -> Result<Vec<Row>, String> {
    let lines: Vec<_> = text.lines().collect();
    let marker = format!("# complete: true; seed: {G4_SEED}; pairs: {N}; rows: 6");
    if lines.last().copied() != Some(marker.as_str()) {
        return Err("missing or mismatched completion marker".into());
    }
    let mut data = lines.iter().copied().filter(|line| !line.starts_with('#'));
    if data.next() != Some("tau,observation_time_ps,total_pairs,reacted_count,surviving_count") {
        return Err("unexpected CSV header".into());
    }
    let mut rows = Vec::new();
    for (line_number, line) in data.enumerate() {
        let fields: Vec<_> = line.split(',').collect();
        if fields.len() != 5 {
            return Err(format!(
                "data row {} has {} fields",
                line_number + 1,
                fields.len()
            ));
        }
        let parse_f64 = |value: &str| value.parse::<f64>().map_err(|error| error.to_string());
        let parse_u64 = |value: &str| value.parse::<u64>().map_err(|error| error.to_string());
        rows.push(Row {
            tau: parse_f64(fields[0])?,
            observation_time_ps: parse_f64(fields[1])?,
            total_pairs: parse_u64(fields[2])?,
            reacted_count: parse_u64(fields[3])?,
            surviving_count: parse_u64(fields[4])?,
        });
    }
    if rows.len() != TAU.len() {
        return Err(format!("expected 6 data rows, found {}", rows.len()));
    }
    for (index, (row, expected_tau)) in rows.iter().zip(TAU).enumerate() {
        if row.tau != expected_tau {
            return Err(format!(
                "row {index} has tau {}, expected {expected_tau}",
                row.tau
            ));
        }
        let expected_time = expected_tau * TIME_SCALE_PS;
        if (row.observation_time_ps - expected_time).abs() > 1e-15 {
            return Err(format!("row {index} has wrong observation time"));
        }
        if row.total_pairs != N || row.reacted_count + row.surviving_count != N {
            return Err(format!("row {index} has incomplete pair coverage"));
        }
        if index > 0 && row.reacted_count < rows[index - 1].reacted_count {
            return Err(format!("row {index} has non-monotonic reaction count"));
        }
    }
    Ok(rows)
}

fn validate_provenance(text: &str) -> Result<(), String> {
    let value: toml::Value = toml::from_str(text).map_err(|error| error.to_string())?;
    let expected_strings = [
        ("geant4_version", "11.4.1"),
        ("g4emlow_version", "8.8"),
        ("chemistry_constructor", "G4EmDNAChemistry_option3"),
        ("time_step_model", "SBS"),
        ("command_line", COMMAND),
    ];
    for (key, expected) in expected_strings {
        if value.get(key).and_then(toml::Value::as_str) != Some(expected) {
            return Err(format!(
                "provenance {key} does not match the frozen contract"
            ));
        }
    }
    if value.get("reaction_id").and_then(toml::Value::as_integer) != Some(1)
        || value.get("pairs").and_then(toml::Value::as_integer) != Some(N as i64)
        || value.get("seed").and_then(toml::Value::as_integer) != Some(G4_SEED)
        || value
            .get("initial_separation_nm")
            .and_then(toml::Value::as_float)
            != Some(0.2)
    {
        return Err("provenance numeric settings do not match the frozen contract".into());
    }
    let tau = value
        .get("tau")
        .and_then(toml::Value::as_array)
        .ok_or("provenance tau must be an array")?;
    if tau.len() != TAU.len()
        || !tau
            .iter()
            .zip(TAU)
            .all(|(actual, expected)| actual.as_float() == Some(expected))
    {
        return Err("provenance tau family does not match".into());
    }
    if value
        .get("generator_dirty")
        .and_then(toml::Value::as_bool)
        .is_none()
    {
        return Err("provenance generator_dirty must be boolean".into());
    }
    for key in ["generator_commit", "compiler", "platform", "generated_at"] {
        if value
            .get(key)
            .and_then(toml::Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Err(format!("provenance {key} must be nonempty"));
        }
    }
    Ok(())
}

fn valid_csv() -> String {
    let mut text = String::from(
        "# generator: test\n\
         # complete only with marker\n\
         tau,observation_time_ps,total_pairs,reacted_count,surviving_count\n",
    );
    for (index, tau) in TAU.into_iter().enumerate() {
        let reacted = index as u64;
        text.push_str(&format!(
            "{tau},{},{N},{reacted},{}\n",
            tau * 0.197_050_889_166_177_16,
            N - reacted
        ));
    }
    text.push_str("# complete: true; seed: 202610231; pairs: 100000; rows: 6\n");
    text
}

fn valid_provenance() -> String {
    format!(
        r#"geant4_version = "11.4.1"
g4emlow_version = "8.8"
chemistry_constructor = "G4EmDNAChemistry_option3"
time_step_model = "SBS"
reaction_id = 1
initial_separation_nm = 0.2
pairs = {N}
seed = {G4_SEED}
tau = [0.25, 0.5, 1.0, 2.0, 4.0, 16.0]
command_line = "{COMMAND}"
generator_commit = "0123456789abcdef"
generator_dirty = false
compiler = "MSVC 19.50"
platform = "Windows"
generated_at = "2026-10-23"
"#
    )
}

#[test]
fn accepts_exact_six_row_csv_and_matching_provenance() {
    let rows = parse_and_validate_csv(&valid_csv()).expect("valid CSV");
    assert_eq!(rows.len(), 6);
    assert_eq!(rows[5].surviving_count, N - 5);
    validate_provenance(&valid_provenance()).expect("valid provenance");
}

#[test]
fn rejects_missing_or_duplicate_tau_rows() {
    let missing = valid_csv().replace("4,0.7882035566647086,100000,4,99996\n", "");
    assert!(parse_and_validate_csv(&missing).is_err());

    let duplicate = valid_csv().replace(
        "# complete: true",
        "4,0.7882035566647086,100000,4,99996\n# complete: true",
    );
    assert!(parse_and_validate_csv(&duplicate).is_err());
}

#[test]
fn rejects_incomplete_counts_or_completion_marker() {
    let bad_counts = valid_csv().replace(",100000,3,99997", ",100000,3,99996");
    assert!(parse_and_validate_csv(&bad_counts).is_err());
    let incomplete = valid_csv().replace("# complete: true", "# complete: false");
    assert!(parse_and_validate_csv(&incomplete).is_err());
}

#[test]
fn rejects_mismatched_provenance_contract() {
    for invalid in [
        valid_provenance().replace("seed = 202610231", "seed = 1"),
        valid_provenance().replace("time_step_model = \"SBS\"", "time_step_model = \"IRT\""),
        valid_provenance().replace(COMMAND, "g4rs_ref two_body_h_h wrong.csv 1 2"),
    ] {
        assert!(validate_provenance(&invalid).is_err());
    }
}

#[test]
#[ignore = "RR-003: awaiting committed two_body_h_h.csv and provenance_m7_two_body_h_h.toml"]
fn m7_irt_survival_matches_geant4_sbs() {
    let reference = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../validation/reference");
    let csv = std::fs::read_to_string(reference.join("two_body_h_h.csv"))
        .expect("RR-003 awaiting validation/reference/two_body_h_h.csv");
    let provenance = std::fs::read_to_string(reference.join("provenance_m7_two_body_h_h.toml"))
        .expect("RR-003 awaiting validation/reference/provenance_m7_two_body_h_h.toml");
    let rows = parse_and_validate_csv(&csv).expect("invalid Geant4 H+H CSV");
    validate_provenance(&provenance).expect("invalid Geant4 H+H provenance");

    let parameters = option3_parameters(REACTION_ID);
    let thresholds = TAU.map(|tau| tau * TIME_SCALE_PS);
    let streams = RngStreams::new(IRT_SEED);
    let mut irt_reacted = [0_u64; 6];
    for index in 0..N {
        let history_id = u64::from(REACTION_ID) * N + index;
        let [a, b] = isolated_pair(parameters, 0.2, history_id);
        let mut rng = streams.stream(history_id, Stage::Chemistry).unwrap();
        match sample_independent_reaction_time(
            MoleculePair::new(&a, &b).unwrap(),
            &parameters,
            &mut rng,
        )
        .unwrap_or_else(|error| panic!("IRT pair {index}: {error}"))
        {
            ReactionTime::After(time) => {
                for point in 0..6 {
                    irt_reacted[point] += u64::from(time.ps() <= thresholds[point]);
                }
            }
            ReactionTime::NoReaction => {}
            ReactionTime::Immediate => panic!("IRT pair {index}: initially separated"),
        }
    }

    let mut failures = Vec::new();
    for point in 0..6 {
        let irt_cdf = irt_reacted[point] as f64 / N as f64;
        let geant4_cdf = rows[point].reacted_count as f64 / N as f64;
        let residual = (irt_cdf - geant4_cdf).abs();
        let bound = 3.0 * (2.0 * V9_CDF[point] * (1.0 - V9_CDF[point]) / N as f64).sqrt();
        eprintln!(
            "H+H tau={}: IRT={irt_cdf:.6} Geant4-SBS={geant4_cdf:.6} residual={residual:.6} bound={bound:.6}",
            TAU[point]
        );
        if residual > bound {
            failures.push(format!(
                "tau={}: |IRT-Geant4|={residual} exceeds combined three-sigma bound={bound}",
                TAU[point]
            ));
        }
    }
    assert!(failures.is_empty(), "H+H failures: {}", failures.join("; "));
}
