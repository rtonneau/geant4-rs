//! Random-variate helpers shared by the models.

use dna_core::rng::u64_to_open01;
use rand::Rng;

/// Uniform variate in the open interval (0, 1) from any `rand::Rng`, through the same
/// 52-bit mapping as `dna_core::rng::HistoryRng::uniform_open01` (neither endpoint is returned,
/// so `ln(u)` is finite) so that results never depend on the `rand` distribution API.
pub fn uniform<R: Rng + ?Sized>(rng: &mut R) -> f64 {
    u64_to_open01(rng.next_u64())
}
