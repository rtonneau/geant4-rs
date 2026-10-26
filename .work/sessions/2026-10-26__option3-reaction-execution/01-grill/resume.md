# Session: option3-reaction-execution

**Date:** 2026-10-26T20:38:59.663Z
**Status:** Grill phase complete

## Problem Statement

The option3 table records 72 reactions, but it cannot yet execute charged partially diffusion-controlled (PDC) pairs, unary/background channels, or Geant4 reaction placement. R03 completes these reaction primitives before a multispecies scheduler is built.

## Context & Constraints

- **Current behavior:** `ReactionTable` validates source-order entries; isolated IRT supports TDC and neutral PDC. Charged PDC returns an unsupported-kinetics error, and first-order/background entries have no clocks or placement primitive.
- **Pain point:** A complete data table can be mistaken for an executable chemistry network, leaving 11 charged PDC and 20 unary/background reactions unavailable.
- **Dependencies:** R02 is merged. Preserve the existing option3 literals, reaction-53 balance exception, 15 mobile species, bulk participants, pair-kernel behavior, and Geant4 11.4.1 provenance. Follow `docs/reviews/proposed-roadmap.md` R03 and inspect the named C++ source directly.
- **Tech stack:** Rust `g4-chemistry`, embedded TOML reaction/species tables, `rand` caller-owned RNG, and the existing typed units/errors; no new dependency is planned.

## Success Metrics

- Compile all 72 default entries into supported immutable execution descriptors: 52 mobile-pair and 20 unary/background; preserve ID, source order, product order, and bulk-reservoir identity. Reject unknown regimes and invalid rates, diffusion, radii, products, or arithmetic before history processing.
- Sample all eleven charged-PDC channels and first-order clocks with source units and formulas. Check contact, inside/outside radii, attraction/repulsion where configured, parameter limits, and no-reaction mass against independent high-precision anchors. For each tested unary competition family, use N >= 100,000 and preregistered family alpha 0.001 to check total hazard and branching fractions.
- Place zero/one/two/three products in source order, including conditional parent placement, zero separation, unequal diffusion, and degenerate cases. Verify atom/charge balance with explicit reservoir exchange for every channel; allow only the exact named reaction-53 anomaly.

## Architecture & Approach

Compile each validated reaction into an internal pair or unary descriptor and retain the immutable network for lookup. Reuse existing TDC/neutral-PDC samplers; add the charged branch of `G4DNAIRT::GetIndependentReactionTime` and `SamplePDC`. Add independent exponential clocks for source first- and pseudo-first-order rates. A zero first-order rate in a custom full table loaded through `ReactionTable::from_toml` compiles to an explicit disabled unary descriptor and never produces an event; `ReactionTable::option3()` remains strict about its embedded literals. Add source-order product and reaction-site placement primitives from `G4DNAIRT::MakeReaction`. Return typed errors for unrepresentable or ambiguous domains rather than silently changing physics. Document each reaction ID's regime and precise upstream source line, plus the numerical validation evidence.

## Assumptions & Trade-offs

Match the Geant4 11.4.1 conditional parent-placement Gaussian expression literally, although its standard-deviation argument appears dimensionally inconsistent; document that limitation. Preserve the existing source literals and product stoichiometry, including reaction 53. Do not add a multispecies event heap, boundaries, arbitrary scavenger configuration, IRT-syn, spin channels, or silent correction of source anomalies. R03 supplies primitives, not a whole chemistry history.

## Open Questions

None for the approved R03 design. Any source ambiguity discovered during implementation must be recorded as an explicit error or limitation rather than resolved by an unreviewed physical assumption.

## Notes

Normative Geant4 11.4.1 paths: `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc` (`Sampling`, `GetIndependentReactionTime`, `SamplePDC`, `MakeReaction`) and `source/physics_lists/constructors/electromagnetic/src/G4EmDNAChemistry_option3.cc` (`ConstructReactionTable`). The source's conditional-placement Gaussian expression is at `G4DNAIRT.cc:433`. Existing mathematical pair evidence must remain valid.
