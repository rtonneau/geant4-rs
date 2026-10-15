# I. Open questions and risks

## Open questions (owner: project lead unless stated)
- **Q1 RESOLVED (user):** limit primary energy to <= 1 MeV initially. Electron beams > 1 MeV are rejected with a typed error; the 1-10 MeV question (D.1 a/b/c) moves to M11.
- **Q2** Electron option(s) to support: recommendation option2 first (default in chem6; includes vib/attach), option4/6 only on demand.
- **Q3** Proton secondary-electron sampling fidelity and cross-section data for Rudd Extended (many ion tables, only p/H needed): audit during M5.
- **Q4** "hp/cumulated" fast tables vs on-the-fly integration: choose cumulated (faster, Geant4 option-2 default `fasterCode`) after verifying statistical equivalence with the slow path (M2).
- **Q5 CLARIFIED, awaiting your confirmation:** energy normalisation now defined concretely in 01 section A.4 from the C++ source (E_dep = sum of local deposits; per-event ratio mean as Geant4; we additionally report ratio of totals). Only the solvation deposit and PrimaryKiller details remain to confirm in M0/M2.
- **Q6** Multi-track/boundary chemistry (UHDR prototype, periodic/bounded boxes): out of initial scope; M7 data structures must not preclude it (explicit box, per-species position arrays).
- **Q7 RESOLVED (user, revised):** data stay outside the repo; see `docs/10-data-requirements.md`. Local path via `DNA_DATA_DIR`; manifest with checksums committed, data not. Data-dependent tests run locally only.
- **Q8** Temperature/density dependence: fixed 298 K water for now; confirm in C++ (G7).
- **Q9 RESOLVED (user):** `serde_json` approved for machine-readable output.
- **Q10** Whether to keep package name `geant4-rs` for a facade crate or rename (cosmetic).

## Risks (ranked)
| # | Risk | Severity | Mitigation / investigation |
|---|---|---|---|
| R1 | Final-state sampling semantics are only visible in C++ `SampleSecondaries`; mis-port silently biases deposits and G | High | M0/M2 audit G1, line-by-line notes in each model file, distribution tests V4 against Geant4 |
| R2 | Data format/units/scale-factor mistakes in tables | High | M1 round-trip tests vs Geant4 numbers; V2 |
| R3 | Geant4 reference itself limited/questionable (e.g. chemistry prototype status of UHDR, IRT approximation) so agreement != correctness | High | L4 comparisons, document known Geant4 deviations, do not claim validity from L2/L3 |
| R4 | Energy-normalisation mismatch makes G-values incomparable | High | resolve Q5/G4 in M0 before M8 |
| R5 | Reaction-radius / partially diffusion-controlled implementation drifts from Geant4 | High | transcribe from C++, V9 vs analytic, compare with SBS oracle |
| R6 | 1-10 MeV beams cannot be done within DNA model validity | Medium-High | Q1 decision, explicit errors |
| R7 | Runtime of proton / high-energy electron histories (heavy secondary cascades) | Medium | work-stealing small chunks, M9 profiling; no cutoffs that change physics |
| R8 | Reproducibility claims overreach (float summation, platform) | Medium | fixed-order reductions, claim scoped to same toolchain (E.2) |
| R9 | Licence blocks committing fixtures/data | Medium | fetch script, minimal derived subsets after review |
| R10 | `rand`/crate API churn | Low | pin in `Cargo.lock`, thin wrapper in `dna-core` |
