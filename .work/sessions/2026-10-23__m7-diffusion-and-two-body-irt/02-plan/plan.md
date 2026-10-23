# Implementation Plan

**Session:** m7-diffusion-and-two-body-irt
**Date:** 2026-10-23T06:26:00.367Z
**Estimated effort:** 7-10 days

## Strategy

Build from the numerical foundation upward: special functions, molecule identity and diffusion, source-faithful runtime parameters, TDC sampling, then PDC sampling. Once the deterministic kernels are complete, run the pre-registered V9 and thread-independence suites and document only the verified scope. Formal closure of M6 is a blocker before this plan is implemented or consumes the 1 ps species records.

## Tickets Overview

- **Ticket 1:** Add high-accuracy `erfc` and inverse-`erfc` functions with committed reference fixtures.
- **Ticket 2:** Add stable molecule identity, validated pair construction, and free Brownian diffusion.
- **Ticket 3:** Derive validated option3 two-body runtime parameters for TDC and PDC reactions.
- **Ticket 4:** Implement neutral and charged TDC independent reaction-time sampling.
- **Ticket 5:** Implement neutral PDC independent reaction-time sampling and convergence errors.
- **Ticket 6:** Run the pre-registered V9 statistical family and prove 1/2/8-thread reproducibility.
- **Ticket 7:** Update architecture, algorithm, roadmap, risk, data, and reproducibility documentation.

## Sequencing Rationale

Tickets 1-3 establish independently tested interfaces used by both samplers. Ticket 4 fixes the public outcome and common IRT behavior before Ticket 5 adds the more complex rejection sampler. Ticket 6 validates the complete kernels against analytic laws and concurrency requirements. Ticket 7 records measured results after the evidence exists.

## Risks & Mitigation

- **Risk:** M6 is treated as closed in prose but its GPS session remains unfinished. → **Mitigation:** Formally finish `2026-10-19__m6-close-gate` before shipping Ticket 1.
- **Risk:** A low-accuracy inverse error function biases reaction-time tails. → **Mitigation:** Validate central and tail fixtures to `1e-14` before using the function in IRT.
- **Risk:** Geant4's default option3 IRT is accidentally mixed with IRT-syn. → **Mitigation:** Cite and test only `G4DNAIRT::GetIndependentReactionTime`; ADR 0003 records the boundary.
- **Risk:** Statistical tests are tuned after observing results. → **Mitigation:** Commit seeds, sample sizes, time points, families, and thresholds in headers before the first run.
- **Risk:** Numerical exhaustion is mistaken for physical non-reaction. → **Mitigation:** Keep `NoReaction` separate from a typed 10,000-attempt convergence error.

## Assumptions

- Geant4 11.4.1 is the normative source version and the configured `DEV_DIR`/`G4_ROOT` paths remain valid.
- No new third-party numerical dependency is introduced.
- M6's Geant4 simulations and comparisons are complete, but its close-gate session must still be formally finished.
- Finite boundaries, products, competing pairs, queues, resampling, SBS, ODE, and the many-species scheduler remain out of scope.
