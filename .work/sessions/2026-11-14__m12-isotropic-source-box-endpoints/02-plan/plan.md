# Implementation Plan

**Session:** m12-isotropic-source-box-endpoints
**Date:** 2026-11-14T08:47:16.343Z
**Estimated effort:** 1 day

## Strategy

First the RNG stage and the isotropic sampler with their statistical tests (g4-core), then the RunConfig option wired into the run layer with determinism tests (g4-sim), then the RR-009 box rows, the run and the recorded verdict.

## Tickets Overview

- **Ticket 1:** `Stage::Source = 3` and an isotropic direction sampler (cos theta = 2u-1, phi = 2 pi u), verified against Geant4 `G4RandomDirection`, with pre-registered KS tests.
- **Ticket 2:** `RunConfig.direction` as an untagged `PrimaryDirection` enum (`[x,y,z]` | `"isotropic"`), wired through the run layer; serialisation, default-bitwise and 1/2/8-thread tests; docs/reproducibility.md.
- **Ticket 3:** Enable the six RR-009 box endpoints with isotropic source, unchanged criteria; run the ignored consumer; record the verdict in docs/07, docs/08, validation/gates.json.

## Sequencing Rationale

The sampler must exist and be validated before the config uses it; the gate run needs the config.

## Risks & Mitigation

- **Risk:** fixed-direction output changes bitwise → **Mitigation:** Source stream only touched when isotropic; existing determinism and regression tests must pass unmodified.
- **Risk:** box rows FAIL or come out INCONCLUSIVE → **Mitigation:** record verdict as is; no criterion change; report to user.
- **Risk:** the consumer run is long (10k histories x 3 energies, 100 keV) → **Mitigation:** release profile, run in background.

## Assumptions

- RR-009 reference CSVs for the box are committed in validation/reference/.
- Reproducing the distribution, not the Geant4 random sequence.
