//! Test-only independent-history orchestration; production coupling remains R06.
//! Registered workload: seed 20261106016, IDs 37..101, 30 molecules/history,
//! full option3, 1 ps..1 us, pools 1/2/8 and batches 1/7/64/128.
//! Exact comparisons, including every float's bits and RNG continuations.
#![cfg(feature = "chemistry")]

use g4_chemistry::history::{
    CandidateSearch, HistoryInput, HistoryOptions, HistoryResult, run_history,
};
use g4_chemistry::reaction_network::ReactionNetwork;
use g4_chemistry::reactions::ReactionTable;
use g4_chemistry::species::SpeciesTable;
use g4_chemistry::{Species, SpeciesInstance};
use g4_core::rng::{HistoryRng, RngStreams, Stage};
use g4_core::units::Time;
use g4_core::vec3::Vec3;
use rayon::prelude::*;

const SEED: u64 = 20_261_106_016;
const FIRST: u64 = 37;
const HISTORIES: u64 = 64;

#[derive(Debug, PartialEq)]
struct Snapshot {
    id: u64,
    result: HistoryResult,
    // PartialEq alone would miss a signed-zero change in a coordinate.
    float_bits: Vec<u64>,
    continuation: [[u64; 16]; 3],
}

fn continuation(rng: &mut HistoryRng) -> [u64; 16] {
    std::array::from_fn(|_| rng.uniform_open01().to_bits())
}

fn history(id: u64, network: &ReactionNetwork, search: CandidateSearch) -> Snapshot {
    let streams = RngStreams::new(SEED);
    let mut physics = streams.stream(id, Stage::Physics).unwrap();
    let mut physchem = streams.stream(id, Stage::PhysChem).unwrap();
    let mut chemistry = streams.stream(id, Stage::Chemistry).unwrap();
    // Synthetic inputs deliberately consume a separate stage stream; these
    // are not claimed to be physical or physicochemical histories.
    let records: Vec<_> = Species::ALL
        .into_iter()
        .cycle()
        .take(30)
        .map(|species| SpeciesInstance {
            species,
            position: Vec3::new(
                5.0 * physchem.uniform_open01(),
                5.0 * physchem.uniform_open01(),
                5.0 * physchem.uniform_open01(),
            ),
            time: Time::from_ps(1.0),
        })
        .collect();
    let input = HistoryInput::new(&records).unwrap();
    let mut expected_physics = physics.clone();
    let mut expected_physchem = physchem.clone();
    let times = [1.0, 10.0, 100.0, 1_000.0, 100_000.0, 1_000_000.0].map(Time::from_ps);
    let result = run_history(
        &input,
        network,
        &times,
        HistoryOptions {
            search,
            max_trace_records: Some(10_000),
            ..Default::default()
        },
        &mut chemistry,
    )
    .unwrap();
    assert_eq!(input.snapshot(), records);
    let continuation = [
        continuation(&mut physics),
        continuation(&mut physchem),
        continuation(&mut chemistry),
    ];
    assert_eq!(continuation[0], self::continuation(&mut expected_physics));
    assert_eq!(continuation[1], self::continuation(&mut expected_physchem));
    let mut untouched_chemistry = streams.stream(id, Stage::Chemistry).unwrap();
    assert_ne!(
        continuation[2],
        self::continuation(&mut untouched_chemistry)
    );

    let mut floats = vec![
        result.diagnostics.end_time.ps(),
        result.diagnostics.time.ps(),
    ];
    if let Some(cutoff) = result.diagnostics.cutoff {
        floats.push(cutoff.nm());
    }
    if let CandidateSearch::Indexed { cutoff_multiplier } = result.diagnostics.search {
        floats.push(cutoff_multiplier);
    }
    floats.extend(result.observations.iter().map(|o| o.time.ps()));
    for event in result.trace.as_ref().unwrap() {
        floats.push(event.time.ps());
        for position in event
            .placement
            .parent_positions
            .iter()
            .chain(event.placement.products.iter().map(|p| &p.position))
        {
            floats.extend([position.x, position.y, position.z]);
        }
    }
    Snapshot {
        id,
        result,
        float_bits: floats.into_iter().map(f64::to_bits).collect(),
        continuation,
    }
}

fn run(
    threads: usize,
    batch: usize,
    reverse: bool,
    network: &ReactionNetwork,
    search: CandidateSearch,
) -> Vec<Snapshot> {
    let mut ids: Vec<_> = (FIRST..FIRST + HISTORIES).collect();
    if reverse {
        ids.reverse();
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap();
    let mut output = pool.install(|| {
        assert_eq!(rayon::current_num_threads(), threads);
        let mut output = Vec::new();
        for chunk in ids.chunks(batch) {
            let results: Vec<_> = chunk
                .par_iter()
                .with_min_len(1)
                .map(|&id| history(id, network, search))
                .collect();
            output.extend(results);
        }
        output
    });
    output.sort_by_key(|snapshot| snapshot.id);
    output
}

#[test]
fn full_history_outputs_and_streams_are_exact_across_threads_batches_and_submission_order() {
    let network = ReactionTable::option3()
        .unwrap()
        .compile(&SpeciesTable::option3().unwrap())
        .unwrap();
    for search in [CandidateSearch::default(), CandidateSearch::AllPairs] {
        let expected = run(1, HISTORIES as usize, false, &network, search);
        assert_eq!(expected.len(), HISTORIES as usize);
        assert!(
            expected
                .iter()
                .all(|s| s.result.diagnostics.executed_reactions > 0)
        );
        assert_ne!(expected[0].continuation, expected[1].continuation);
        assert_ne!(expected[0].float_bits, expected[1].float_bits);
        for threads in [1, 2, 8] {
            for batch in [1, 7, 64, 128] {
                for reverse in [false, true] {
                    let actual = run(threads, batch, reverse, &network, search);
                    assert_eq!(
                        actual, expected,
                        "threads={threads}, batch={batch}, reverse={reverse}, search={search:?}"
                    );
                }
            }
        }
    }
}
