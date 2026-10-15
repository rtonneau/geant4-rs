//! Deterministic, history-addressed random-number streams.
//!
//! Design (docs/05-parallelisation.md E.2): one `ChaCha12` key derived from a master seed,
//! and one ChaCha *stream* per `(history_id, stage)`. A history's numbers therefore depend
//! only on its integer id, never on thread identity or scheduling, and there is no shared
//! mutable generator. Streams of different ids/stages are disjoint by construction (ChaCha
//! streams select distinct nonces of the same keystream family).
//!
//! `rand`/`rand_chacha` are used only for the generator and the `Rng` traits; uniform
//! variates are produced by [`u64_to_open01`] so that behaviour does not depend on the
//! `rand` distribution API (risk R10).

use crate::error::DnaError;
use rand::TryRng;
use rand_chacha::ChaCha12Rng;
use rand_chacha::rand_core::SeedableRng;
use std::convert::Infallible;

/// Largest history id: `(id << 2) | stage` must fit in the 64-bit ChaCha stream number.
pub const MAX_HISTORY_ID: u64 = (1 << 62) - 1;

/// Simulation stage owning a sub-stream of a history (stage 3 is reserved).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Physics = 0,
    PhysChem = 1,
    Chemistry = 2,
}

/// Vigna's splitmix64, used only to expand the 64-bit master seed to a 256-bit key.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Map a raw 64-bit word to a double in the open interval (0, 1) using its top 52 bits:
/// `(k + 0.5) / 2^52`, so neither 0 nor 1 is ever returned (safe for `ln`). 52 rather than
/// 53 bits because `1 - 2^-54` is not representable and would round to exactly 1.0.
pub fn u64_to_open01(w: u64) -> f64 {
    ((w >> 12) as f64 + 0.5) * (1.0 / 4_503_599_627_370_496.0)
}

/// Factory of per-history generators for one run.
#[derive(Debug, Clone)]
pub struct RngStreams {
    key: [u8; 32],
}

impl RngStreams {
    pub fn new(master_seed: u64) -> Self {
        let mut state = master_seed;
        let mut key = [0u8; 32];
        for chunk in key.chunks_exact_mut(8) {
            chunk.copy_from_slice(&splitmix64(&mut state).to_le_bytes());
        }
        Self { key }
    }

    /// Generator for `(history_id, stage)`, positioned at the start of its stream.
    pub fn stream(&self, history_id: u64, stage: Stage) -> Result<HistoryRng, DnaError> {
        if history_id > MAX_HISTORY_ID {
            return Err(DnaError::HistoryIdOutOfRange {
                history_id,
                max: MAX_HISTORY_ID,
            });
        }
        let mut rng = ChaCha12Rng::from_seed(self.key);
        rng.set_stream((history_id << 2) | stage as u64);
        Ok(HistoryRng(rng))
    }
}

/// Random generator of one stage of one history.
#[derive(Debug, Clone)]
pub struct HistoryRng(ChaCha12Rng);

impl HistoryRng {
    /// Uniform variate in the open interval (0, 1).
    pub fn uniform_open01(&mut self) -> f64 {
        u64_to_open01(rand::Rng::next_u64(self))
    }
}

impl TryRng for HistoryRng {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        self.0.try_next_u32()
    }
    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        self.0.try_next_u64()
    }
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
        self.0.try_fill_bytes(dst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use std::collections::HashSet;

    fn words(mut r: HistoryRng, n: usize) -> Vec<u64> {
        (0..n).map(|_| r.next_u64()).collect()
    }

    #[test]
    fn splitmix64_matches_reference_vectors() {
        // Reference outputs of Vigna's splitmix64 seeded with 0.
        let mut s = 0u64;
        assert_eq!(splitmix64(&mut s), 0xE220_A839_7B1D_CDAF);
        assert_eq!(splitmix64(&mut s), 0x6E78_9E6A_A1B9_65F4);
        assert_eq!(splitmix64(&mut s), 0x06C4_5D18_8009_454F);
    }

    #[test]
    fn same_seed_history_stage_repeats_exactly() {
        let a = RngStreams::new(42).stream(7, Stage::Physics).unwrap();
        let b = RngStreams::new(42).stream(7, Stage::Physics).unwrap();
        assert_eq!(words(a, 1000), words(b, 1000));
    }

    #[test]
    fn different_master_seeds_differ() {
        let a = RngStreams::new(1).stream(0, Stage::Physics).unwrap();
        let b = RngStreams::new(2).stream(0, Stage::Physics).unwrap();
        assert_ne!(words(a, 4), words(b, 4));
    }

    #[test]
    fn streams_are_disjoint_across_histories_and_stages() {
        let streams = RngStreams::new(12345);
        let mut seen = HashSet::new();
        for id in 0..10_000u64 {
            for stage in [Stage::Physics, Stage::PhysChem, Stage::Chemistry] {
                let w = words(streams.stream(id, stage).unwrap(), 4);
                assert!(
                    seen.insert(w),
                    "history {id} {stage:?} repeats an earlier stream"
                );
            }
        }
    }

    #[test]
    fn stream_is_independent_of_creation_order() {
        let streams = RngStreams::new(9);
        let first_then_other = {
            let _ = streams.stream(1, Stage::Physics).unwrap();
            words(streams.stream(2, Stage::Chemistry).unwrap(), 16)
        };
        let alone = words(streams.stream(2, Stage::Chemistry).unwrap(), 16);
        assert_eq!(first_then_other, alone);
    }

    #[test]
    fn history_id_beyond_62_bits_is_rejected() {
        let streams = RngStreams::new(0);
        assert!(streams.stream(MAX_HISTORY_ID, Stage::Chemistry).is_ok());
        assert_eq!(
            streams.stream(MAX_HISTORY_ID + 1, Stage::Physics).err(),
            Some(crate::error::DnaError::HistoryIdOutOfRange {
                history_id: MAX_HISTORY_ID + 1,
                max: MAX_HISTORY_ID
            })
        );
    }

    #[test]
    fn open01_excludes_both_endpoints() {
        let lo = u64_to_open01(0);
        let hi = u64_to_open01(u64::MAX);
        assert!(lo > 0.0 && lo < 1.0e-15);
        assert!(hi < 1.0 && hi > 1.0 - 1.0e-15);
    }

    /// Kolmogorov-Smirnov test against U(0,1). Acceptance fixed before looking at results:
    /// D < 1.95 / sqrt(N) (asymptotic critical value at alpha ~ 0.001). Seed is fixed, so the
    /// outcome is deterministic.
    #[test]
    fn uniform_samples_pass_kolmogorov_smirnov() {
        const N: usize = 100_000;
        let mut r = RngStreams::new(2024).stream(0, Stage::Physics).unwrap();
        let mut u: Vec<f64> = (0..N).map(|_| r.uniform_open01()).collect();
        u.sort_by(|a, b| a.total_cmp(b));
        let d = u
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                let lo = x - i as f64 / N as f64;
                let hi = (i + 1) as f64 / N as f64 - x;
                lo.max(hi)
            })
            .fold(0.0, f64::max);
        let critical = 1.95 / (N as f64).sqrt();
        assert!(d < critical, "KS D = {d} >= {critical}");
    }

    #[test]
    fn mean_of_uniform_samples_is_one_half() {
        const N: usize = 100_000;
        let mut r = RngStreams::new(5).stream(3, Stage::PhysChem).unwrap();
        let mean = (0..N).map(|_| r.uniform_open01()).sum::<f64>() / N as f64;
        // sigma of the mean = sqrt(1/12/N) ~ 9.1e-4; accept 5 sigma.
        assert!((mean - 0.5).abs() < 5.0 * (1.0 / 12.0 / N as f64).sqrt());
    }

    /// Pins the generator across `rand`/`rand_chacha` upgrades (risk R10): if this changes,
    /// every fixed-seed regression value in the project changes too. Values were recorded
    /// from this implementation (regression pin, not an independent oracle).
    #[test]
    fn golden_values_pin_generator_choice() {
        let w = words(RngStreams::new(42).stream(0, Stage::Physics).unwrap(), 3);
        assert_eq!(
            w,
            vec![
                2_885_535_749_968_689_682,
                5_597_393_260_974_906_631,
                13_949_277_091_178_962_527
            ]
        );
    }
}
