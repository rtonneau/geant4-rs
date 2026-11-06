//! R05-C1: protocol committed as c7188b35 before any comparison sampling.
//! See docs/validation/irt-history-convergence.md. This is a search-sensitivity
//! gate, not an independent physical oracle or a full-track Geant4 comparison.

use g4_chemistry::history::{
    CandidateSearch, HistoryDiagnostics, HistoryInput, HistoryOptions, PairHistory,
    PairHistoryOptions, run_history,
};
use g4_chemistry::reaction_network::ReactionNetwork;
use g4_chemistry::reactions::{DEFAULT_REACTIONS_TOML, ReactionTable};
use g4_chemistry::species::{DEFAULT_SPECIES_TOML, SpeciesTable};
use g4_chemistry::{Species, SpeciesInstance};
use g4_core::rng::{RngStreams, Stage};
use g4_core::testing::ScriptedRng;
use g4_core::units::Time;
use g4_core::vec3::Vec3;
use rayon::prelude::*;
use sha2::{Digest, Sha256};

const SEED: u64 = 20_261_106_007;
const BATCHES: usize = 64;
const PER_BATCH: usize = 1_024;
const HISTORIES: usize = BATCHES * PER_BATCH;
const TIMES: [f64; 6] = [10.0, 100.0, 1e3, 1e4, 1e5, 1e6];
const POLICIES: [&str; 4] = ["all-pairs", "1", "2", "4"];
const COMPARISONS: [(usize, usize); 6] = [(1, 0), (2, 0), (3, 0), (1, 2), (1, 3), (2, 3)];
const CRITICAL: f64 = 2.66;
const BAND: f64 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Verdict {
    Pass,
    Inconclusive,
    Fail,
    Invalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scale {
    Absolute,
    Relative,
}

#[derive(Debug)]
struct Assessment {
    verdict: Verdict,
    scale: Scale,
    left: f64,
    reference: f64,
    interval: Option<[f64; 2]>,
    reason: &'static str,
}

fn mean(values: &[f64]) -> f64 {
    // Center before summing: constant 0.1 must remain exactly 0.1.
    values[0] + values.iter().map(|x| x - values[0]).sum::<f64>() / values.len() as f64
}

fn interval_verdict([low, high]: [f64; 2]) -> Verdict {
    if !low.is_finite() || !high.is_finite() || low > high {
        Verdict::Invalid
    } else if low >= -BAND && high <= BAND {
        Verdict::Pass
    } else if low > BAND || high < -BAND {
        Verdict::Fail
    } else {
        Verdict::Inconclusive
    }
}

fn compare(left: &[f64], reference: &[f64]) -> Assessment {
    let mut result = Assessment {
        verdict: Verdict::Invalid,
        scale: Scale::Absolute,
        left: f64::NAN,
        reference: f64::NAN,
        interval: None,
        reason: "invalid batches",
    };
    if left.len() != BATCHES
        || reference.len() != BATCHES
        || left
            .iter()
            .chain(reference)
            .any(|x| !x.is_finite() || !(0.0..=100_000.0).contains(x))
    {
        return result;
    }
    result.left = mean(left);
    result.reference = mean(reference);
    result.scale = if result.reference < 0.1 {
        Scale::Absolute
    } else {
        Scale::Relative
    };
    let differences: Vec<_> = left.iter().zip(reference).map(|(x, y)| x - y).collect();
    if differences.iter().all(|&d| d == differences[0]) {
        (result.verdict, result.reason) = if differences[0] == 0.0 {
            (Verdict::Inconclusive, "no observed paired differences")
        } else {
            (Verdict::Fail, "constant nonzero paired difference")
        };
        return result;
    }
    let d = mean(&differences);
    let y = result.reference;
    let centered: Vec<_> = differences
        .iter()
        .zip(reference)
        .map(|(db, yb)| (db - d, yb - y))
        .collect();
    let divisor = (BATCHES - 1) as f64;
    let variance_d = centered.iter().map(|(a, _)| a * a).sum::<f64>() / divisor;
    let variance_y = centered.iter().map(|(_, b)| b * b).sum::<f64>() / divisor;
    let covariance = centered.iter().map(|(a, b)| a * b).sum::<f64>() / divisor;
    if ![variance_d, variance_y, covariance]
        .into_iter()
        .all(f64::is_finite)
        || variance_d < 0.0
        || variance_y < 0.0
    {
        result.reason = "invalid covariance";
        return result;
    }
    let q = CRITICAL * CRITICAL / BATCHES as f64;
    let (center, radius) = match result.scale {
        Scale::Absolute => (d, CRITICAL * (variance_d / BATCHES as f64).sqrt()),
        Scale::Relative => {
            let a = y * y - q * variance_y;
            if a <= 0.0 {
                result.verdict = Verdict::Inconclusive;
                result.reason = "unbounded Fieller interval";
                return result;
            }
            // Algebraically identical to the preregistered quadratic roots.
            // Regress D on Y to avoid subtracting nearly equal discriminant
            // terms when a paired ratio is almost constant. The residual sum
            // of squares is nonnegative without clipping any variance.
            let beta = if variance_y > 0.0 {
                covariance / variance_y
            } else {
                0.0
            };
            let residual_variance = centered
                .iter()
                .map(|(db, yb)| (db - beta * yb).powi(2))
                .sum::<f64>()
                / divisor;
            let e = d - beta * y;
            (
                beta + e * y / a,
                (q * (residual_variance / a + e * e * variance_y / (a * a))).sqrt(),
            )
        }
    };
    let interval = [center - radius, center + radius];
    result.verdict = interval_verdict(interval);
    result.interval = Some(interval);
    result.reason = "interval inclusion";
    result
}

#[derive(Clone, Copy, Debug)]
enum Ensemble {
    Clustered,
    Dilute,
}

impl Ensemble {
    fn offset(self) -> usize {
        match self {
            Self::Clustered => 0,
            Self::Dilute => HISTORIES,
        }
    }
}

fn fixture(ensemble: Ensemble, mut uniform: impl FnMut() -> f64) -> HistoryInput {
    let records: Vec<_> = Species::ALL
        .into_iter()
        .cycle()
        .take(30)
        .enumerate()
        .map(|(i, species)| {
            let [x, y, z] = std::array::from_fn(|_| uniform() - 0.5);
            let position = match ensemble {
                Ensemble::Clustered => Vec3::new(
                    (i / 10) as f64 * 1200.0 - 1200.0 + 4.0 * x,
                    4.0 * y,
                    4.0 * z,
                ),
                Ensemble::Dilute => Vec3::new(2400.0 * x, 2400.0 * y, 2400.0 * z),
            };
            SpeciesInstance {
                species,
                position,
                time: Time::from_ps(1.0),
            }
        })
        .collect();
    HistoryInput::new(&records).unwrap()
}

fn options(policy: usize) -> HistoryOptions {
    HistoryOptions {
        limits: PairHistoryOptions {
            end_time: Time::from_ps(1e6),
            max_molecules: 100_000,
            max_pending_events: 100_000,
            max_reactions: 100_000,
        },
        search: if policy == 0 {
            CandidateSearch::AllPairs
        } else {
            CandidateSearch::Indexed {
                cutoff_multiplier: [1.0, 2.0, 4.0][policy - 1],
            }
        },
        max_observations: 1_024,
        max_trace_records: None,
    }
}

fn initial_candidates(input: &HistoryInput, policy: usize) -> usize {
    // Empty descriptor list enumerates geometry only: no reactions, sampling,
    // RNG draws or extra chemical histories. All policies visit the same live
    // identity pairs regardless of which descriptors subsequently match them.
    let mut rng = ScriptedRng::new(&[]);
    let history = PairHistory::with_search(
        input,
        &[],
        options(policy).limits,
        options(policy).search,
        &mut rng,
    )
    .unwrap();
    assert_eq!(rng.consumed(), 0);
    history.diagnostics().candidate_pairs
}

#[derive(Clone, Copy, Debug, Default)]
struct Resources {
    candidates: u64,
    channels: u64,
    scheduled: u64,
    executed: u64,
    peak_pending: usize,
    max_arena: usize,
    max_index: usize,
    peak_row: usize,
    peak_payload_bytes: usize,
}

impl Resources {
    fn add(&mut self, d: &HistoryDiagnostics) {
        self.merge(Self {
            candidates: d.candidate_pairs as u64,
            channels: d.sampled_pair_channels as u64,
            scheduled: d.scheduled_events,
            executed: d.executed_reactions as u64,
            peak_pending: d.peak_pending_events,
            max_arena: d.arena_records,
            max_index: d.spatial_entries,
            peak_row: d.peak_candidate_buffer,
            peak_payload_bytes: d.peak_storage_bytes,
        });
    }

    fn merge(&mut self, other: Self) {
        self.candidates += other.candidates;
        self.channels += other.channels;
        self.scheduled += other.scheduled;
        self.executed += other.executed;
        self.peak_pending = self.peak_pending.max(other.peak_pending);
        self.max_arena = self.max_arena.max(other.max_arena);
        self.max_index = self.max_index.max(other.max_index);
        self.peak_row = self.peak_row.max(other.peak_row);
        self.peak_payload_bytes = self.peak_payload_bytes.max(other.peak_payload_bytes);
    }
}

struct Batch {
    sums: [[[u64; 15]; 6]; 4],
    resources: [Resources; 4],
}

fn sample_batch(
    ensemble: Ensemble,
    batch: usize,
    network: &ReactionNetwork,
) -> Result<Batch, String> {
    let streams = RngStreams::new(SEED);
    let mut output = Batch {
        sums: [[[0; 15]; 6]; 4],
        resources: [Resources::default(); 4],
    };
    for local in batch * PER_BATCH..(batch + 1) * PER_BATCH {
        let id = (ensemble.offset() + local) as u64;
        let mut input_rng = streams.stream(id, Stage::PhysChem).unwrap();
        let input = fixture(ensemble, || input_rng.uniform_open01());
        for policy in 0..4 {
            let mut rng = streams.stream(id, Stage::Chemistry).unwrap();
            let result = run_history(
                &input,
                network,
                &TIMES.map(Time::from_ps),
                options(policy),
                &mut rng,
            )
            .map_err(|error| format!("{ensemble:?},history={id},policy={policy}: {error:?}"))?;
            if result.observations.len() != TIMES.len()
                || result.diagnostics.time.ps() != 1e6
                || result.trace.is_some()
            {
                return Err(format!("{ensemble:?},history={id}: malformed output"));
            }
            for (t, observation) in result.observations.iter().enumerate() {
                if observation.time.ps() != TIMES[t]
                    || observation.counts.iter().sum::<usize>() > 100_000
                {
                    return Err(format!("{ensemble:?},history={id}: invalid counts/time"));
                }
                for (sum, count) in output.sums[policy][t].iter_mut().zip(observation.counts) {
                    *sum += count as u64;
                }
            }
            output.resources[policy].add(&result.diagnostics);
        }
    }
    Ok(output)
}

#[test]
#[ignore = "R05-C1 statistical gate: explicitly run; FAIL/INCONCLUSIVE blocks G2"]
fn preregistered_cutoff_convergence_gate() {
    println!(
        "R05-C1 protocol=c7188b35bd06420e1ba914c5f404e0c2c2fcfc5b seed={SEED} batches={BATCHES} histories_per_batch={PER_BATCH}"
    );
    for (name, bytes) in [
        ("species", DEFAULT_SPECIES_TOML.as_bytes()),
        ("reactions", DEFAULT_REACTIONS_TOML.as_bytes()),
        ("test", include_bytes!("history_convergence.rs").as_slice()),
        (
            "Cargo.lock",
            include_bytes!("../../../Cargo.lock").as_slice(),
        ),
    ] {
        println!("SHA256,{name},{:x}", Sha256::digest(bytes));
    }
    let network = ReactionTable::option3()
        .unwrap()
        .compile(&SpeciesTable::option3().unwrap())
        .unwrap();
    assert_eq!(network.iter().count(), 72);
    let mut global = Verdict::Pass;
    let mut categories = [0; 4];
    for ensemble in [Ensemble::Clustered, Ensemble::Dilute] {
        let mut input_rng = RngStreams::new(SEED)
            .stream(ensemble.offset() as u64, Stage::PhysChem)
            .unwrap();
        let first_input = fixture(ensemble, || input_rng.uniform_open01());
        let initial = std::array::from_fn::<_, 4, _>(|p| initial_candidates(&first_input, p));
        println!("INITIAL_CANDIDATES,{ensemble:?},{initial:?}");
        assert!(initial[1] < initial[0], "INVALID: no initial pruning");
        // Independent histories run in parallel; indexed collection and output
        // are batch ordered, with no shared RNG or floating-point reduction.
        let sampled: Vec<_> = (0..BATCHES)
            .into_par_iter()
            .map(|batch| sample_batch(ensemble, batch, &network))
            .collect();
        let mut batches = Vec::with_capacity(BATCHES);
        for (batch, result) in sampled.into_iter().enumerate() {
            match result {
                Ok(data) => batches.push(data),
                Err(error) => panic!("R05-C1 INVALID batch={batch}: {error}"),
            }
        }
        let mut resources = [Resources::default(); 4];
        for (b, batch) in batches.iter().enumerate() {
            for (p, name) in POLICIES.iter().enumerate() {
                resources[p].merge(batch.resources[p]);
                for (t, time) in TIMES.iter().enumerate() {
                    for species in Species::ALL {
                        println!(
                            "BATCH,{ensemble:?},{b},{name},{time},{},{},{}",
                            species.chemistry_name(),
                            batch.sums[p][t][species.index()],
                            PER_BATCH
                        );
                    }
                }
            }
        }
        for (name, resource) in POLICIES.iter().zip(resources) {
            println!("RESOURCE,{ensemble:?},{name},{resource:?}");
        }
        for (left, reference) in COMPARISONS {
            for (t, time) in TIMES.iter().enumerate() {
                for species in Species::ALL {
                    let samples = |p: usize| -> Vec<f64> {
                        batches
                            .iter()
                            .map(|b| b.sums[p][t][species.index()] as f64 / PER_BATCH as f64)
                            .collect()
                    };
                    let result = compare(&samples(left), &samples(reference));
                    categories[result.verdict as usize] += 1;
                    global = global.max(result.verdict);
                    println!(
                        "ENDPOINT,{ensemble:?},{}/{},{time},{},{result:?}",
                        POLICIES[left],
                        POLICIES[reference],
                        species.chemistry_name()
                    );
                }
            }
        }
    }
    assert_eq!(categories.iter().sum::<usize>(), 1080);
    println!(
        "R05-C1 VERDICT={global:?} pass={} inconclusive={} fail={} invalid={}",
        categories[0], categories[1], categories[2], categories[3]
    );
    assert_eq!(
        global,
        Verdict::Pass,
        "R05-C1 blocks G2 unless every endpoint passes"
    );
}

#[test]
fn interval_classification_and_boundaries_are_fail_closed() {
    assert_eq!(interval_verdict([-0.01, 0.01]), Verdict::Pass);
    assert_eq!(interval_verdict([-0.02, 0.0]), Verdict::Inconclusive);
    assert_eq!(interval_verdict([0.01, 0.02]), Verdict::Inconclusive);
    assert_eq!(interval_verdict([0.011, 0.02]), Verdict::Fail);
    assert_eq!(interval_verdict([-0.02, -0.011]), Verdict::Fail);
    assert_eq!(interval_verdict([f64::NAN, 0.0]), Verdict::Invalid);
    assert_eq!(interval_verdict([0.0, f64::INFINITY]), Verdict::Invalid);
    assert_eq!(interval_verdict([0.02, 0.01]), Verdict::Invalid);
}

fn alternating(center: f64, half_width: f64) -> Vec<f64> {
    (0..BATCHES)
        .map(|i| center + if i % 2 == 0 { -half_width } else { half_width })
        .collect()
}

#[test]
fn comparator_distinguishes_equivalence_bias_and_insufficient_precision() {
    let reference = vec![2.0; BATCHES];
    let pass = compare(&alternating(2.0, 0.01), &reference);
    assert_eq!(pass.verdict, Verdict::Pass);
    assert_eq!(pass.scale, Scale::Relative);
    let expected = CRITICAL * 0.01 / (63.0_f64.sqrt() * 2.0);
    let [low, high] = pass.interval.unwrap();
    assert!((low + expected).abs() < 1e-14 && (high - expected).abs() < 1e-14);
    for center in [1.9, 2.1] {
        assert_eq!(
            compare(&alternating(center, 0.01), &reference).verdict,
            Verdict::Fail
        );
    }
    assert_eq!(
        compare(&alternating(2.0, 0.5), &reference).verdict,
        Verdict::Inconclusive
    );
}

#[test]
fn rare_counts_use_absolute_units_and_point_one_is_non_rare() {
    let rare = compare(&alternating(0.05, 0.01), &[0.05; BATCHES]);
    assert_eq!(rare.scale, Scale::Absolute);
    assert_eq!(rare.verdict, Verdict::Pass);
    assert_eq!(
        compare(&alternating(0.08, 0.01), &[0.05; BATCHES]).verdict,
        Verdict::Fail
    );
    let boundary = compare(&alternating(0.1, 0.01), &[0.1; BATCHES]);
    assert_eq!(boundary.scale, Scale::Relative);
    assert_eq!(boundary.verdict, Verdict::Inconclusive);
}

#[test]
fn fieller_retains_covariance_and_rejects_an_unresolved_denominator() {
    let reference = alternating(2.0, 1.0);
    let proportional: Vec<_> = reference.iter().map(|y| y * (1.0 + 1.0 / 256.0)).collect();
    let result = compare(&proportional, &reference);
    assert_eq!(result.verdict, Verdict::Pass);
    assert_eq!(result.interval, Some([1.0 / 256.0; 2]));
    let mut uncertain_reference = vec![0.0; BATCHES];
    uncertain_reference[0] = 64.0;
    let result = compare(&vec![1.0; BATCHES], &uncertain_reference);
    assert_eq!(result.verdict, Verdict::Inconclusive);
    assert_eq!(result.reason, "unbounded Fieller interval");
}

#[test]
fn degenerate_or_malformed_batches_cannot_establish_equivalence() {
    for value in [0.0, 2.0] {
        assert_eq!(
            compare(&[value; BATCHES], &[value; BATCHES]).verdict,
            Verdict::Inconclusive
        );
    }
    assert_eq!(
        compare(&[2.0; BATCHES], &[1.0; BATCHES]).verdict,
        Verdict::Fail
    );
    assert_eq!(compare(&[], &[]).verdict, Verdict::Invalid);
    assert_eq!(
        compare(&[1.0; BATCHES - 1], &[1.0; BATCHES]).verdict,
        Verdict::Invalid
    );
    for bad in [f64::NAN, f64::INFINITY, -1.0, 100_001.0] {
        assert_eq!(
            compare(&[bad; BATCHES], &[1.0; BATCHES]).verdict,
            Verdict::Invalid
        );
    }
}

#[test]
fn fixtures_preserve_species_creation_order_and_exercise_initial_pruning() {
    // Deterministic geometry checks do not draw any campaign RNG stream or
    // inspect count outcomes before the registered statistical invocation.
    for ensemble in [Ensemble::Clustered, Ensemble::Dilute] {
        let mut draw = 0;
        let input = fixture(ensemble, || {
            let u = ((draw * 37) % 97) as f64 / 97.0 + 0.5 / 97.0;
            draw += 1;
            u
        });
        assert_eq!(draw, 90);
        for (i, record) in input.snapshot().iter().enumerate() {
            assert_eq!(record.species, Species::ALL[i % 15]);
            assert_eq!(record.time.ps(), 1.0);
            match ensemble {
                Ensemble::Clustered => {
                    let center = (i / 10) as f64 * 1200.0 - 1200.0;
                    assert!((record.position.x - center).abs() < 2.0);
                    assert!(record.position.y.abs() < 2.0 && record.position.z.abs() < 2.0);
                }
                Ensemble::Dilute => {
                    assert!(record.position.x.abs() < 1200.0);
                    assert!(record.position.y.abs() < 1200.0 && record.position.z.abs() < 1200.0);
                }
            }
        }
        assert_eq!(initial_candidates(&input, 0), 435);
        assert!(initial_candidates(&input, 1) < 435);
        assert_eq!(options(1).max_trace_records, None);
    }
}
