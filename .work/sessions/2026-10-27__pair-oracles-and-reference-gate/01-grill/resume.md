# Session: pair-oracles-and-reference-gate

**Date:** 2026-10-27T06:27:16.521Z
**Status:** Grill phase complete

## Problem Statement

R03 makes all option3 reaction regimes executable, but pair reaction-time distributions do not yet have the independent, cross-regime evidence needed for G1. R04 must test the Rust option3 IRT sampler against analytic or high-precision laws, a separate physical Brownian pair oracle, and source-matched Geant4 11.4.1 IRT evidence without reinterpreting historical SBS results.

## Context & Constraints

- **Current behavior:** R03 is complete on main, including charged PDC. The repository has isolated IRT tests, a validation-only Geant4-faithful SBS stepper, and the separate RR-003 H+H SBS request; these do not discharge R04's full pair gate.
- **Pain point:** The existing checks do not independently establish pair CDFs and eventual no-reaction probabilities across neutral/charged TDC/PDC and applicable A+B/A+A cases. The known early H+H IRT/SBS discrepancy must remain visible.
- **Dependencies:** R01 and R03 are present. Preserve existing reference files, generator modes, RR-003, fixtures and historical V9 verdicts. Agents may compile-check but never run Geant4; each new run goes in RUN_REQUESTS.md with a unique ID. User-run complete data and provenance are required for a Geant4 verdict.
- **Tech stack:** Rust g4-chemistry tests and validation modules, the Geant4 11.4.1 C++ source and additive g4rs_ref mode, CSV/TOML provenance, and the maintained cargo gates.

## Success Metrics

- Commit a case matrix distinguishing dimensionless cases from literal option3 channels and covering neutral/charged TDC/PDC, applicable A+B/A+A, contact/1.5/3 reaction radii, positive times 0.01/0.1/1/10/100 times a²/(D1+D2), and eventual no-reaction mass.
- Before inspecting new results, fix independent seeds, >=100,000 analytic samples per case, family alpha 0.001 with pointwise z and Bonferroni reporting, and a Brownian schedule of >=10,000 independent pairs per case, at least three timestep levels, and precommitted precision doubling up to 1,000,000. The last two CDF estimates must differ by <0.005 absolute and be compatible within three combined SE; lack of convergence is INCONCLUSIVE.
- Add a Geant4 option3 IRT pair mode, exact user command and strict ignored consumer. Independently seeded, source-matched Geant4 CDF/reaction-fraction differences must pass the predeclared family test and <0.01 absolute effect limit with sufficient precision. A deliberately biased sampler must fail the gate. Keep analytic, independent Brownian and Geant4 verdicts separate; G1 passes only when all three pass.

## Architecture & Approach

Use a test-only Brownian pair oracle separate from the existing source-faithful SBS implementation and from the production IRT sampler. For charged pairs, model the underlying Coulomb interaction and appropriate reaction boundary; report any deviation from Geant4's effective-distance IRT approximation as its own scientific verdict. Use analytic formulas or independent high-precision evaluation for CDF and eventual non-reaction checks. Add an additive Geant4 generator mode that exercises option3 IRT with exactly matched channel parameters and units, plus strict CSV/provenance parsing and independently seeded comparisons. Publish the committed matrix, preregistration and convergence report before interpreting data. Preserve legacy V9 and RR-003 classifications separately.

## Assumptions & Trade-offs

The physical Coulomb oracle may disagree with the Geant4 approximation. Such a discrepancy beyond registered limits leaves G1 open even if Rust agrees with Geant4 IRT. Generator implementation and compile checks may finish with evidence REQUESTED; they cannot claim scientific acceptance before complete user-run data pass. R04 does not add a production SBS engine, arbitrary box chemistry, a scheduler or track simulation, or claim V10 closed-system validation.

## Open Questions

No unresolved design choice. The eventual scientific verdict depends on the preregistered comparisons and user-run Geant4 evidence.

## Notes

The source basis is Geant4 11.4.1 G4DNAIRT.cc GetIndependentReactionTime/SamplePDC, G4DNAMoleculeEncounterStepper.cc and G4DNASmoluchowskiReactionModel.cc, plus the option3 reaction constructor. R03's 72-ID execution matrix identifies applicable literal channels. Document model applicability, numerical approximations, provenance and every intentional deviation from Geant4.
