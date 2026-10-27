//! Independent first- and pseudo-first-order clocks for option3.
//!
//! Geant4 11.4.1 `G4DNAIRT::Sampling` (G4DNAIRT.cc:250-281) draws one
//! exponential time per nonzero scavenging/first-order channel and chooses
//! the earliest. Rates are the observed source literals in s^-1; the bulk
//! partner is a reservoir descriptor, never a second mobile molecule.
//! This module returns elapsed times and leaves event-window filtering and
//! absolute-time scheduling to a later stage.

use crate::ChemistryError;
use crate::irt::ReactionTime;
use crate::reaction_network::UnaryReactionParameters;
use g4_core::sampling::uniform;
use g4_core::units::Time;
use rand::Rng;

/// Sample one channel's independent elapsed clock from the caller's RNG.
///
/// A disabled channel returns [`ReactionTime::NoReaction`] without a draw.
/// Enabled channels consume exactly one open uniform and return a finite,
/// strictly positive delay in ps, or a typed error if it cannot be represented.
pub fn sample_unary_reaction_time(
    parameters: &UnaryReactionParameters,
    rng: &mut impl Rng,
) -> Result<ReactionTime, ChemistryError> {
    if parameters.is_disabled() {
        return Ok(ReactionTime::NoReaction);
    }

    let u = uniform(rng);
    // Geant4 uses -log(1-u)/k. ln_1p retains accuracy at small u;
    // multiplying before division retains subnormal ps delays at large k.
    let delay_ps = -(-u).ln_1p() * 1.0e12 / parameters.rate_per_s();
    if !delay_ps.is_finite() || delay_ps <= 0.0 {
        return Err(ChemistryError::UnaryClockDomain {
            reaction_id: parameters.reaction_id(),
        });
    }
    Ok(ReactionTime::After(Time::from_ps(delay_ps)))
}
