//! Fixed chemistry reproducibility workload, registered before execution:
//! seed=20_261_023_106, 1,024 histories, Rayon pools of exactly 1, 2, and 8 threads.
//! Each history obtains its own Stage::Chemistry stream and reuses it for all
//! calls: real option3 reactions 1, 11, 9, each at coincidence, strict overlap,
//! equality, and separation, followed by free diffusion of both molecules for
//! dt=[0,.5,10,100] ps respectively. PDC rejection draws vary by history.
//! This exercises both kernels; it is not a many-species reaction scheduler.
//!
//! Ordered snapshots contain history/reaction/molecule identities, tagged IRT
//! outcomes, and all six diffused coordinates. Floats are encoded as raw u64
//! bits before JSON serialization, retaining signed zero and every mantissa bit.
//! Compare every record and the serialized bytes, not summaries or tolerances.

mod common;

use common::{isolated_pair, option3_parameters, position_bits, reaction_time_bits};
use dna_chemistry::diffusion::diffuse;
use dna_chemistry::irt::sample_independent_reaction_time;
use dna_chemistry::molecule::MoleculePair;
use dna_chemistry::reaction_parameters::TwoBodyReactionParameters as Parameters;
use dna_chemistry::species::SpeciesTable;
use dna_core::rng::{RngStreams, Stage};
use dna_core::units::Time;
use rayon::prelude::*;
use serde::Serialize;

const SEED: u64 = 20_261_023_106;
const HISTORIES: usize = 1024;

#[derive(Debug, PartialEq, Eq, Serialize)]
struct PairSnapshot {
    history_id: u64,
    reaction_id: u16,
    configuration: usize,
    molecule_ids: [u64; 2],
    outcome: (u8, u64),
    diffused_position_bits: [[u64; 3]; 2],
}

fn history(
    history_id: u64,
    streams: &RngStreams,
    parameters: &[Parameters; 3],
    species: &SpeciesTable,
) -> Vec<PairSnapshot> {
    let mut rng = streams.stream(history_id, Stage::Chemistry).unwrap();
    let mut snapshots = Vec::with_capacity(12);
    for (&p, separated) in parameters.iter().zip([0.2, 0.8, 2.0]) {
        let sigma = p.reaction_radius().nm();
        let distances = [0.0, 0.5 * sigma, sigma, separated];
        for (configuration, (distance, dt)) in distances
            .into_iter()
            .zip([0.0, 0.5, 10.0, 100.0])
            .enumerate()
        {
            let molecules = isolated_pair(p, distance, history_id);
            let pair = MoleculePair::new(&molecules[0], &molecules[1]).unwrap();
            let outcome = sample_independent_reaction_time(pair, &p, &mut rng).unwrap();
            let diffused_position_bits = molecules.map(|molecule| {
                let instance = molecule.instance();
                let position = diffuse(
                    instance.position,
                    species.get(instance.species).diffusivity,
                    Time::from_ps(dt),
                    &mut rng,
                )
                .unwrap();
                let bits = position_bits(position);
                if dt == 0.0 {
                    assert_eq!(bits, position_bits(instance.position));
                } else {
                    assert_ne!(bits, position_bits(instance.position));
                }
                bits
            });
            snapshots.push(PairSnapshot {
                history_id,
                reaction_id: p.reaction_id(),
                configuration,
                molecule_ids: molecules.map(|molecule| molecule.id().0),
                outcome: reaction_time_bits(outcome),
                diffused_position_bits,
            });
        }
    }
    snapshots
}

fn run(threads: usize) -> Vec<Vec<PairSnapshot>> {
    let streams = RngStreams::new(SEED);
    let parameters = [1, 11, 9].map(option3_parameters);
    let species = SpeciesTable::option3().unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap()
        .install(|| {
            assert_eq!(rayon::current_num_threads(), threads);
            (0..HISTORIES)
                .into_par_iter()
                .with_min_len(1)
                .map(|id| history(id as u64, &streams, &parameters, &species))
                .collect()
        })
}

#[test]
fn serialized_outcomes_and_diffused_positions_are_identical_with_1_2_8_threads() {
    let baseline = run(1);
    assert_eq!(baseline.len(), HISTORIES);
    let mut kinds = [0; 3];
    for (id, records) in baseline.iter().enumerate() {
        assert_eq!(records.len(), 12);
        for record in records {
            assert_eq!(record.history_id, id as u64);
            kinds[usize::from(record.outcome.0)] += 1;
        }
    }
    assert!(kinds.into_iter().all(|count| count > 0));
    // A repeated/empty stream or no-op workload must not satisfy the comparison.
    assert_ne!(
        baseline[0][3].diffused_position_bits,
        baseline[1][3].diffused_position_bits
    );
    let expected_bytes = serde_json::to_vec(&baseline).unwrap();
    for threads in [2, 8] {
        let actual = run(threads);
        assert_eq!(actual.len(), baseline.len());
        for (id, (expected, observed)) in baseline.iter().zip(&actual).enumerate() {
            assert_eq!(expected, observed, "history {id}, {threads} threads");
        }
        assert!(
            expected_bytes == serde_json::to_vec(&actual).unwrap(),
            "serialized history data differ with {threads} threads"
        );
    }
}
