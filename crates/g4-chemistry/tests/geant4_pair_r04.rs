//! RR-004: source-matched, isolated Geant4 option3 IRT pairs.
//! The 144 endpoints have their own Bonferroni family; V9 and RR-003 retain
//! their original tests. Missing reference evidence is REQUESTED, not PASS.

mod common;

use std::path::PathBuf;

use common::{isolated_pair, option3_parameters};
use g4_chemistry::irt::{ReactionTime, sample_independent_reaction_time};
use g4_chemistry::molecule::MoleculePair;
use g4_chemistry::reaction_parameters::TwoBodyReactionParameters;
use g4_core::rng::{RngStreams, Stage};
use g4_core::units::RateConstant;
use sha2::{Digest, Sha256};

const IDS: [u16; 8] = [1, 2, 8, 9, 11, 17, 42, 51];
const DISTANCES: [f64; 3] = [1.0, 1.5, 3.0];
const TAU: [f64; 5] = [0.01, 0.1, 1.0, 10.0, 100.0];
const ROWS: usize = 144;
const N: u64 = 100_000;
const G4_SEED: u64 = 20_261_027_404;
const RUST_SEED: u64 = 20_261_027_403;
const SEED_BASE: u64 = 2_147_483_562;
// Phi^-1(1 - 0.001/(2*144)); two-sided Bonferroni family.
const Z_CRIT: f64 = 4.495_384_196_564_731;
const EFFECT_LIMIT: f64 = 0.01;
const COMMAND: &str =
    "g4rs_ref pair_matrix_irt ../../reference/pair_matrix_irt_r04_n100000.csv 20261027404 100000";
const HEADER: &str = "reaction_id,r0_over_a,a_nm,D_nm2_per_ps,onsager_nm,kobs_M_inv_s_inv,endpoint,tau,observation_time_ps,total_pairs,event_count";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Endpoint {
    Cdf(usize),
    NoReaction,
}

#[derive(Clone, Copy, Debug)]
struct Row {
    id: u16,
    distance: f64,
    endpoint: Endpoint,
    event_count: u64,
}

fn close(actual: f64, expected: f64) -> bool {
    actual.is_finite() && (actual - expected).abs() <= 1e-12 * expected.abs().max(1e-12)
}

fn float(value: &str, field: &str) -> Result<f64, String> {
    value
        .parse::<f64>()
        .map_err(|_| format!("invalid {field}: {value}"))
}

fn parse_csv(text: &str) -> Result<Vec<Row>, String> {
    let lines: Vec<_> = text.lines().collect();
    let marker = format!("# complete: true; seed: {G4_SEED}; pairs: {N}; rows: {ROWS}");
    if lines.last().copied() != Some(marker.as_str()) {
        return Err("missing or mismatched final completion marker".into());
    }
    if lines.first().copied()
        != Some("# generator: g4rs_ref; mode: pair_matrix_irt; schema: r04_pair_v1")
        || !lines.get(1).is_some_and(|s| {
            s.starts_with("# geant4_version_number: 1141 (")
                && s.ends_with("; chemistry: G4EmDNAChemistry_option3; timestep_model: IRT")
        })
        || lines.get(2).copied()
            != Some(
                format!(
                    "# seed: {G4_SEED}; rng: RanecuEngine; engine_seeds: {},{}; pairs: {N}",
                    G4_SEED % SEED_BASE + 1,
                    G4_SEED / SEED_BASE + 1
                )
                .as_str(),
            )
        || lines.get(3).copied()
            != Some(
                "# direct G4DNAIRT pair kernel; negative -1 ps sentinel means eventual nonreaction; no scheduler cutoff",
            )
        || lines.get(4).copied() != Some(HEADER)
    {
        return Err("CSV generator, settings, or header mismatch".into());
    }
    let mut rows = Vec::with_capacity(ROWS);
    let mut channel_ids = Vec::new();
    for line in &lines[5..lines.len() - 1] {
        if let Some(comment) = line.strip_prefix("# channel: ") {
            let parts: Vec<_> = comment.split("; ").collect();
            if parts.len() != 4 {
                return Err("incomplete channel comment".into());
            }
            let id = parts[0]
                .parse::<u16>()
                .map_err(|_| "invalid channel comment")?;
            if !IDS.contains(&id) || rows.len() != channel_ids.len() * 18 {
                return Err("unexpected channel comment position".into());
            }
            let reactants = parts[1]
                .strip_prefix("reactants: ")
                .ok_or("missing reactants")?;
            if reactants.split(',').count() != 2
                || reactants.split(',').any(str::is_empty)
                || !close(
                    float(
                        parts[2]
                            .strip_prefix("effective_radius_nm: ")
                            .ok_or("missing effective radius")?,
                        "effective radius",
                    )?,
                    option3_parameters(id).effective_radius().nm(),
                )
                || !close(
                    float(
                        parts[3]
                            .strip_prefix("contact_probability: ")
                            .ok_or("missing contact probability")?,
                        "contact probability",
                    )?,
                    option3_parameters(id).contact_probability(),
                )
            {
                return Err("wrong channel metadata".into());
            }
            channel_ids.push(id);
            continue;
        }
        if line.starts_with('#') || line.is_empty() {
            return Err("unexpected comment or empty data row".into());
        }
        let fields: Vec<_> = line.split(',').collect();
        if fields.len() != 11 {
            return Err(format!("row {} has {} fields", rows.len(), fields.len()));
        }
        let index = rows.len();
        if index >= ROWS {
            return Err("extra data row".into());
        }
        let case = index / 18;
        let distance = index / 6 % 3;
        let endpoint = index % 6;
        let id = fields[0]
            .parse::<u16>()
            .map_err(|_| "invalid reaction ID")?;
        if id != IDS[case] || channel_ids.last() != Some(&id) || channel_ids.len() != case + 1 {
            return Err(format!(
                "row {index} has missing, duplicate or unordered channel"
            ));
        }
        let p = option3_parameters(id);
        let expected = [
            DISTANCES[distance],
            p.reaction_radius().nm(),
            p.relative_diffusivity().nm2_per_ps(),
            p.onsager_radius().nm(),
            p.observed_rate().nm3_per_ps() / RateConstant::from_per_molar_per_s(1.0).nm3_per_ps(),
        ];
        for (field, target) in fields[1..6].iter().zip(expected) {
            if !close(float(field, "option3 setting")?, target) {
                return Err(format!("row {index} has wrong source setting"));
            }
        }
        let row_endpoint = if endpoint == 5 {
            if fields[6..9] != ["no_reaction", "inf", "inf"] {
                return Err(format!("row {index} has wrong no-reaction endpoint"));
            }
            Endpoint::NoReaction
        } else {
            let time = TAU[endpoint] * p.reaction_radius().nm().powi(2)
                / p.relative_diffusivity().nm2_per_ps();
            if fields[6] != "cdf"
                || !close(float(fields[7], "tau")?, TAU[endpoint])
                || !close(float(fields[8], "observation time")?, time)
            {
                return Err(format!("row {index} has wrong finite endpoint"));
            }
            Endpoint::Cdf(endpoint)
        };
        let total = fields[9].parse::<u64>().map_err(|_| "invalid pair count")?;
        let event_count = fields[10]
            .parse::<u64>()
            .map_err(|_| "invalid event count")?;
        if total != N || event_count > N {
            return Err(format!("row {index} has incomplete or impossible count"));
        }
        rows.push(Row {
            id,
            distance: DISTANCES[distance],
            endpoint: row_endpoint,
            event_count,
        });
    }
    if rows.len() != ROWS || channel_ids != IDS {
        return Err("missing or duplicate rows/channels".into());
    }
    for block in rows.chunks_exact(6) {
        for pair in block[..5].windows(2) {
            if pair[0].event_count > pair[1].event_count {
                return Err("nonmonotonic CDF counts".into());
            }
        }
        if block[4].event_count + block[5].event_count > N {
            return Err("CDF and eventual nonreaction exceed total pairs".into());
        }
    }
    Ok(rows)
}

fn validate_provenance(text: &str, csv: &[u8]) -> Result<(), String> {
    let value: toml::Value = toml::from_str(text).map_err(|e| e.to_string())?;
    let string = |key: &str| value.get(key).and_then(toml::Value::as_str);
    let integer = |key: &str| value.get(key).and_then(toml::Value::as_integer);
    for (key, expected) in [
        ("geant4_version", "11.4.1"),
        ("g4emlow_version", "8.8"),
        ("physics_constructor", "G4EmDNAPhysics_option2"),
        ("chemistry_constructor", "G4EmDNAChemistry_option3"),
        ("timestep_model", "IRT"),
        ("mode", "pair_matrix_irt"),
        ("schema", "r04_pair_v1"),
        ("rng_engine", "RanecuEngine"),
        ("command", COMMAND),
        ("working_directory", "validation/geant4/g4rs_ref"),
    ] {
        if string(key) != Some(expected) {
            return Err(format!("wrong provenance {key}"));
        }
    }
    for (key, expected) in [
        ("geant4_version_number", 1141),
        ("seed", G4_SEED as i64),
        ("pairs_per_case_distance", N as i64),
        ("completion_rows", ROWS as i64),
    ] {
        if integer(key) != Some(expected) {
            return Err(format!("wrong provenance {key}"));
        }
    }
    if value.get("generator_dirty").and_then(toml::Value::as_bool) != Some(false) {
        return Err("generator was dirty or cleanliness unrecorded".into());
    }
    let array = |key: &str| {
        value
            .get(key)
            .and_then(toml::Value::as_array)
            .ok_or_else(|| format!("missing provenance {key}"))
    };
    if array("engine_seeds")?
        .iter()
        .map(toml::Value::as_integer)
        .collect::<Vec<_>>()
        != [
            Some((G4_SEED % SEED_BASE + 1) as i64),
            Some((G4_SEED / SEED_BASE + 1) as i64),
        ]
        || array("reaction_ids")?
            .iter()
            .map(toml::Value::as_integer)
            .collect::<Vec<_>>()
            != IDS.map(|id| Some(i64::from(id)))
    {
        return Err("wrong provenance engine seeds or channels".into());
    }
    for (key, expected) in [("r0_over_a", &DISTANCES[..]), ("tau", &TAU[..])] {
        let found = array(key)?;
        if found.len() != expected.len()
            || !found
                .iter()
                .zip(expected)
                .all(|(v, &e)| v.as_float().or_else(|| v.as_integer().map(|n| n as f64)) == Some(e))
        {
            return Err(format!("wrong provenance {key}"));
        }
    }
    for key in ["compiler", "platform", "date"] {
        if string(key).is_none_or(str::is_empty) {
            return Err(format!("missing provenance {key}"));
        }
    }
    let commit = string("generator_commit").ok_or("missing generator commit")?;
    if commit.len() != 40 || !commit.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid generator commit".into());
    }
    let hash = format!("{:x}", Sha256::digest(csv));
    if string("csv_sha256") != Some(hash.as_str()) {
        return Err("CSV SHA-256 does not match provenance".into());
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    Pass,
    Fail,
    Inconclusive,
}

fn compare(rust_count: u64, g4_count: u64, n: u64) -> (Verdict, f64, f64, f64) {
    let difference = (rust_count as f64 - g4_count as f64) / n as f64;
    // Half-count stabilization avoids zero estimated variance for unobserved
    // rare events. Samples and seeds are independent, never paired.
    let p_r = (rust_count as f64 + 0.5) / (n as f64 + 1.0);
    let p_g = (g4_count as f64 + 0.5) / (n as f64 + 1.0);
    let se = ((p_r * (1.0 - p_r) + p_g * (1.0 - p_g)) / n as f64).sqrt();
    let z = difference / se;
    let margin = Z_CRIT * se;
    let verdict = if difference.abs() - margin >= EFFECT_LIMIT {
        Verdict::Fail
    } else if difference.abs() + margin >= EFFECT_LIMIT {
        Verdict::Inconclusive
    } else if z.abs() > Z_CRIT || difference.abs() >= EFFECT_LIMIT {
        Verdict::Fail
    } else {
        Verdict::Pass
    };
    (verdict, z, difference.abs(), margin)
}

fn rust_counts(p: TwoBodyReactionParameters, case: usize, distance: usize) -> [u64; 6] {
    let mut counts = [0; 6];
    let radius = p.reaction_radius().nm();
    let molecules = isolated_pair(p, DISTANCES[distance] * radius, 0);
    let pair = MoleculePair::new(&molecules[0], &molecules[1]).unwrap();
    let streams = RngStreams::new(RUST_SEED);
    for i in 0..N {
        let history = ((case * 3 + distance) as u64) * N + i;
        let mut rng = streams.stream(history, Stage::Chemistry).unwrap();
        let tau = match sample_independent_reaction_time(pair, &p, &mut rng).unwrap() {
            ReactionTime::Immediate => 0.0,
            ReactionTime::After(t) => {
                t.ps() * p.relative_diffusivity().nm2_per_ps() / radius.powi(2)
            }
            ReactionTime::NoReaction => {
                counts[5] += 1;
                continue;
            }
        };
        for (j, observation) in TAU.iter().enumerate() {
            counts[j] += u64::from(tau <= *observation);
        }
    }
    counts
}

#[test]
#[ignore = "RR-004 REQUESTED: complete user-run CSV and provenance have not been committed"]
fn r04_option3_irt_matches_geant4() {
    let reference = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../validation/reference");
    let path = reference.join("pair_matrix_irt_r04_n100000.csv");
    let csv = std::fs::read(&path).expect("RR-004 REQUESTED: missing complete Geant4 CSV");
    let provenance =
        std::fs::read_to_string(reference.join("provenance_r04_pair_matrix_n100000.toml"))
            .expect("RR-004 REQUESTED: missing provenance");
    validate_provenance(&provenance, &csv).expect("RR-004 invalid provenance");
    let rows = parse_csv(std::str::from_utf8(&csv).expect("RR-004 CSV is not UTF-8"))
        .expect("RR-004 incomplete or invalid Geant4 CSV");
    let mut failures = Vec::new();
    let mut inconclusive = Vec::new();
    for (case, &id) in IDS.iter().enumerate() {
        let p = option3_parameters(id);
        for distance in 0..3 {
            let counts = rust_counts(p, case, distance);
            for endpoint in 0..6 {
                let row = rows[(case * 3 + distance) * 6 + endpoint];
                let (verdict, z, effect, margin) = compare(counts[endpoint], row.event_count, N);
                eprintln!(
                    "GEANT4 id={} r/a={} endpoint={:?} Rust={}/{} G4={}/{} z={z:.6} zcrit={Z_CRIT:.6} abs_diff={effect:.6} adjusted_margin={margin:.6} effect_limit={EFFECT_LIMIT:.6} {verdict:?}",
                    row.id, row.distance, row.endpoint, counts[endpoint], N, row.event_count, N
                );
                let label = format!(
                    "id={} r/a={} endpoint={:?}",
                    row.id, row.distance, row.endpoint
                );
                match verdict {
                    Verdict::Pass => {}
                    Verdict::Fail => failures.push(label),
                    Verdict::Inconclusive => inconclusive.push(label),
                }
            }
        }
    }
    let verdict = if !failures.is_empty() {
        "FAIL"
    } else if !inconclusive.is_empty() {
        "INCONCLUSIVE"
    } else {
        "PASS"
    };
    eprintln!(
        "R04 ANALYTIC=see pair_analytic BROWNIAN=see pair_brownian GEANT4={verdict} G1={}",
        if verdict == "PASS" {
            "OPEN until analytic and Brownian PASS"
        } else {
            "OPEN"
        }
    );
    assert!(
        failures.is_empty() && inconclusive.is_empty(),
        "RR-004 GEANT4={verdict}; failures={failures:?}; insufficient precision={inconclusive:?}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_csv() -> String {
        let mut s = format!(
            "# generator: g4rs_ref; mode: pair_matrix_irt; schema: r04_pair_v1\n# geant4_version_number: 1141 (Geant4 11.4.1); chemistry: G4EmDNAChemistry_option3; timestep_model: IRT\n# seed: {G4_SEED}; rng: RanecuEngine; engine_seeds: {},{}; pairs: {N}\n# direct G4DNAIRT pair kernel; negative -1 ps sentinel means eventual nonreaction; no scheduler cutoff\n{HEADER}\n",
            G4_SEED % SEED_BASE + 1,
            G4_SEED / SEED_BASE + 1
        );
        for id in IDS {
            let p = option3_parameters(id);
            s.push_str(&format!("# channel: {id}; reactants: A,B; effective_radius_nm: {:.17}; contact_probability: {:.17}\n", p.effective_radius().nm(), p.contact_probability()));
            for ratio in DISTANCES {
                for (endpoint, maybe_tau) in TAU.into_iter().map(Some).chain([None]).enumerate() {
                    let (label, tau, time) = if let Some(tau) = maybe_tau {
                        (
                            "cdf",
                            tau.to_string(),
                            (tau * p.reaction_radius().nm().powi(2)
                                / p.relative_diffusivity().nm2_per_ps())
                            .to_string(),
                        )
                    } else {
                        ("no_reaction", "inf".to_string(), "inf".to_string())
                    };
                    s.push_str(&format!(
                        "{id},{ratio},{:.17},{:.17},{:.17},{:.17},{label},{tau},{time},{N},{}\n",
                        p.reaction_radius().nm(),
                        p.relative_diffusivity().nm2_per_ps(),
                        p.onsager_radius().nm(),
                        p.observed_rate().nm3_per_ps()
                            / RateConstant::from_per_molar_per_s(1.0).nm3_per_ps(),
                        if endpoint == 5 { 0 } else { endpoint as u64 }
                    ));
                }
            }
        }
        s.push_str(&format!(
            "# complete: true; seed: {G4_SEED}; pairs: {N}; rows: {ROWS}\n"
        ));
        s
    }

    fn valid_provenance(csv: &[u8]) -> String {
        format!(
            r#"geant4_version = "11.4.1"
geant4_version_number = 1141
g4emlow_version = "8.8"
physics_constructor = "G4EmDNAPhysics_option2"
chemistry_constructor = "G4EmDNAChemistry_option3"
timestep_model = "IRT"
mode = "pair_matrix_irt"
schema = "r04_pair_v1"
seed = {G4_SEED}
rng_engine = "RanecuEngine"
engine_seeds = [{}, {}]
pairs_per_case_distance = {N}
reaction_ids = [1, 2, 8, 9, 11, 17, 42, 51]
r0_over_a = [1.0, 1.5, 3.0]
tau = [0.01, 0.1, 1.0, 10.0, 100.0]
command = "{COMMAND}"
working_directory = "validation/geant4/g4rs_ref"
date = "2026-10-27"
generator_commit = "0123456789abcdef0123456789abcdef01234567"
generator_dirty = false
compiler = "MSVC 19.50"
platform = "Windows"
csv_sha256 = "{:x}"
completion_rows = 144
"#,
            G4_SEED % SEED_BASE + 1,
            G4_SEED / SEED_BASE + 1,
            Sha256::digest(csv)
        )
    }

    #[test]
    fn accepts_complete_matrix_and_provenance() {
        let csv = valid_csv();
        assert_eq!(parse_csv(&csv).unwrap().len(), ROWS);
        validate_provenance(&valid_provenance(csv.as_bytes()), csv.as_bytes()).unwrap();
    }

    #[test]
    fn rejects_missing_duplicate_unordered_or_wrong_settings() {
        let csv = valid_csv();
        let line = csv.lines().find(|s| s.starts_with("1,1,0.")).unwrap();
        assert!(parse_csv(&csv.replacen(&format!("{line}\n"), "", 1)).is_err());
        assert!(
            parse_csv(&csv.replacen(&format!("{line}\n"), &format!("{line}\n{line}\n"), 1))
                .is_err()
        );
        assert!(parse_csv(&csv.replacen("1,1,", "1,2,", 1)).is_err());
        assert!(parse_csv(&csv.replacen("1,1,", "2,1,", 1)).is_err());
        assert!(parse_csv(&csv.replacen(",100000,0\n", ",99999,0\n", 1)).is_err());
        assert!(parse_csv(&csv.replacen("# complete: true", "# complete: false", 1)).is_err());
        assert!(parse_csv(&csv.replacen("rows: 144", "rows: 144 ", 1)).is_err());
    }

    #[test]
    fn rejects_invalid_counts_and_provenance() {
        let csv = valid_csv();
        assert!(parse_csv(&csv.replacen(",100000,0\n", ",100000,100001\n", 1)).is_err());
        let provenance = valid_provenance(csv.as_bytes());
        for bad in [
            provenance.replace("generator_dirty = false", "generator_dirty = true"),
            provenance.replace("seed = 20261027404", "seed = 1"),
            provenance.replace(
                "pairs_per_case_distance = 100000",
                "pairs_per_case_distance = 200000",
            ),
            provenance.replace("G4EmDNAChemistry_option3", "G4EmDNAChemistry_option2"),
            provenance.replace("csv_sha256 = \"", "csv_sha256 = \"0"),
        ] {
            assert!(validate_provenance(&bad, csv.as_bytes()).is_err());
        }
    }

    #[test]
    fn bias_and_underpowered_equivalence_are_rejected() {
        assert_eq!(compare(100_000, 100_000, 200_000).0, Verdict::Pass);
        assert_eq!(compare(50_000, 50_000, N).0, Verdict::Inconclusive);
        assert_eq!(compare(55_000, 50_000, N).0, Verdict::Fail);
        assert_eq!(compare(5, 0, 100).0, Verdict::Inconclusive);
    }
}
