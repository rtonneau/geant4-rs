//! A history's result depends only on its id, never on thread count or scheduling
//! (docs/05-parallelisation.md E.2/E.3).

use g4_core::rng::{RngStreams, Stage};
use rayon::prelude::*;

/// Stand-in for a history: draws a data-dependent number of variates over two stages and
/// returns a floating-point result that is sensitive to the exact stream.
fn toy_history(streams: &RngStreams, id: u64) -> f64 {
    let mut phys = streams.stream(id, Stage::Physics).unwrap();
    let n = 10 + (phys.uniform_open01() * 90.0) as usize;
    let mut acc: f64 = (0..n).map(|_| -phys.uniform_open01().ln()).sum();
    let mut chem = streams.stream(id, Stage::Chemistry).unwrap();
    acc += chem.uniform_open01();
    acc
}

fn run_on(threads: usize, streams: &RngStreams, n: u64) -> Vec<f64> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap()
        .install(|| {
            (0..n as usize)
                .into_par_iter()
                .with_min_len(7)
                .map(|id| toy_history(streams, id as u64))
                .collect()
        })
}

#[test]
fn per_history_results_are_bitwise_identical_for_any_thread_count() {
    let streams = RngStreams::new(2026);
    let serial: Vec<f64> = (0..2000).map(|id| toy_history(&streams, id)).collect();
    for threads in [1, 2, 8] {
        let parallel = run_on(threads, &streams, 2000);
        assert_eq!(
            serial.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            parallel.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            "{threads} threads"
        );
    }
}

#[test]
fn fixed_order_reduction_is_reproducible_across_thread_counts() {
    let streams = RngStreams::new(7);
    let sum = |threads| run_on(threads, &streams, 5000).iter().sum::<f64>();
    assert_eq!(sum(1).to_bits(), sum(8).to_bits());
}
