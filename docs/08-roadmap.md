# H. Implementation roadmap

Each milestone ends compilable with `fmt`, `clippy -D warnings`, `cargo test --workspace` green. One validated milestone at a time.

## M0 Foundations (no physics)
Scope: convert repo to workspace (B), `rust-toolchain.toml`, CI `ci.yml`, units newtypes, `Vec3`, `Rng` streams, error types, provenance metadata struct, `docs/reproducibility.md`, `docs/units.md`.
Refs: none (B.7 audit G7/G8 reading starts).
Tests: unit round trips (proptest), RNG stream disjointness/repeatability/uniformity (KS), thread-independence harness.
Acceptance: all listed tests; A.4 G-value convention frozen in docs after reading chem6 scorer (G4).
Risks: `rand` API churn. DoD: CI green on a fresh clone.

## M1 Data layer
Scope: `G4RS_DATA_DIR` resolution, `data/MANIFEST.toml` (generated locally by `scripts/make_data_manifest.py`) + optional checksum verification, table readers for cross-section and differential-cumulative files named in B.4, log-log and 2-D interpolation exactly as `G4DNACrossSectionDataSet` / `QuadInterpolator`, energy-domain enforcement.
Refs: `D/utils/src/G4DNACrossSectionDataSet.cc` (G2), `D/models/src/G4DNAChampionElasticModel.cc`, `G4DNABornIonisationModel1.cc`.
Tests: V1; interpolation vs Geant4 numbers for a few hand-picked points exported once; proptest monotonic CDFs.
Acceptance: V2 infrastructure ready for any table; licence decision Q7 recorded. Risk: file format surprises (units/scale factors).
**Status: implemented (branch `claude/inspiring-edison-v1kw1a`).** Crate `crates/dna-data`: tolerant parser, `CrossSectionTable` (log-log / lin-lin), `AngularCdfTable` (`Theta`), `IonisationCdfTable` (`TransferedEnergy`), `DnaFile` catalog of the 20 files with units and consumers, `DataDir` (`G4RS_DATA_DIR`, manifest verification). G2 and G8 closed in docs/02 B.7.1-B.7.2; per-file units in docs/10. Not done (M2): sampling, the slow-path `DifferentialCrossSection` lookup, RPWBA `TransferedEnergy`, model-level windows/policies (e.g. Melton 12.3-13 eV).

## M2 Electron transport, option2, E <= 1 MeV, primaries only (no secondary tracking)
Scope: Champion elastic, Born excitation, Born1 ionisation (energy loss, secondary *recorded* not transported), infinite water, energy ledger, `mfp` and `range`-like scorers.
Refs: B.2 files + audit G1, G8.
Tests: V2-V5, V7. Acceptance: V2/V3 pass; V4 for elastic angle, ionisation secondary energy; mfp agrees V5.
Perf: no optimisation; record baseline. Risk: sampling code semantics hidden in `SampleSecondaries`.
**Status: implemented (branch `claude/m2-electron-transport`), Geant4 comparison pending.** Crates `dna-physics` (models, `ProcessTable`, transport, ledger) and `dna-sim` (rayon histories, scorers, JSON output with provenance); docs/03's DAG, so physics has neither rayon nor `serde_json`.
- *Scope as decided:* option2 electrons, primary 7.4 eV - 1 MeV (`UnsupportedEnergy` outside), Champion elastic (7.4 eV-1 MeV), `G4DNABornExcitationModel1` (9 eV-1 MeV), `G4DNABornIonisationModel1` (11 eV-1 MeV, cumulated "fast" tables), infinite water, secondaries recorded (energy, direction, position) not transported, Sanche/Melton/solvation left to M3. All three models were re-read in the C++ (docs/02 B.7.4).
- *Passing (V1/V2/V4/V5-self/V7/V13):* windows and `nextafter` neighbours (V1); macroscopic cross sections at three energies and all table nodes against an independent Python re-implementation of the C++ algorithm (`scripts/hand_cross_sections.py`, 1e-9; the Rust output was not used to produce them) (V2-like); V4 against the raw files with the Bonferroni family pre-registered in the test header: 5 elastic-angle KS, 19 ejected-energy KS (every shell), 7 shell and 5 level chi-square, 2 whole-event mixture KS, 1 selection-order chi-square (all p > 2.5e-5); Born angle formulas and the primary momentum balance analytically; the quirks on synthetic tables; energy ledger over 10^4 histories at 100 eV, 1, 10, 100 keV, 1 MeV with max |residual|/E_in = 3.6e-16, 1.0e-15, 3.9e-15, 1.6e-14, 2.8e-13 (V7 asks 1e-9, A.4 asks 1e-12); first-free-path means within 3 standard errors of the hand-computed 1/Sigma at 100 eV, 4321 eV, 250 keV (z = -0.16, -0.43, +1.67 with seed 2025); 1/2/8-thread bitwise equality of every per-history field and of the scores for any batch size (V13).
- *Sensitivity of the V4 tests (checked once by hand on temporary mutants, reverted):* scaling the sampled elastic polar angle by 1 % is rejected by the elastic-angle KS test, and subtracting 1.1 B instead of B from the energy transfer is rejected by the whole-event secondary-energy KS test.
- *Not done / pending:* V2/V3/V5 **against Geant4** (needs `validation/reference/*.csv` from `validation/geant4/`, tests are `#[ignore]`d; the C++ generator is untested); the Q4 slow-path (rejection sampling) equivalence check; Sanche, Melton, solvation (M3); atomic de-excitation after K-shell ionisation (docs/09 Q12); full slowing-down range and W-value (need M3).
- *Intentional deviations from Geant4 (all counted or documented):* (1) the Champion rotation reproduces Geant4's non-unit `orthogonal()` axis by default (`ElasticRotation::Geant4`), `Exact` available (Q13); (2) no atomic de-excitation: the whole 539 eV of a K-shell ionisation is deposited locally and counted (`k_shell_deexcitation_skipped`) (Q12); (3) termination guard `NoInelasticChannel` for 7.4-9 eV, where only elastic scattering exists and Geant4 relies on Sanche/Melton (Q14); (4) table lookups outside their range are errors, not clamps (M1); (5) one total free path then process selection instead of one number of interaction lengths per process (same Poisson process, different stream); (6) `sample` outside the model window is `UnsupportedEnergy`, Geant4 silently returns.
- *Performance baseline* (release, 1 thread, Intel Xeon 2.1 GHz, `cargo test --release -p dna-sim --test baseline -- --ignored --nocapture`, primaries only): 1.9e6 interactions/s at 100 eV, 1.2e6 at 1 keV, 1.4e6 at 10 keV, 8.6e5 at 100 keV, 7.7e5 at 1 MeV (17016 interactions per 1 MeV primary, 0.022 s). No optimisation was attempted; `[profile.test] opt-level = 3` keeps the 10^4-history ledger test at about one minute.

## M3 Secondary electrons + low-energy electron processes
Scope: secondary stack, Sanche vib., Melton attachment, solvation/thermalisation cut (7.4 eV), full energy accounting incl. binding energy policy (Q5). 
Tests: V5-V7 (range 1 keV-1 MeV, deposit spread, secondary spectra). Acceptance: full-track deposited-energy and secondary-spectra V4/V5 vs Geant4 `dnaphysics`/`slowing`. Risk: low-energy secondaries dominate runtime and physics.

## M4 (optional) Electron options 4 and 6
Emfietzoglou, Uehara, CPA100; model transition tests at 10 keV / 250 keV. Only if Q2 says so.

## M5 Protons
Scope: p/H charge-state transport, Miller-Green, Rudd (Extended), Born, ion elastic, Dingfelder charge change; RPWBA (100-300 MeV) after Born; secondary electrons via M3. Closes audit G3.
Tests: V2-V7 for protons; stopping power vs PSTAR (L4 report). Acceptance: V5 range/stopping power; secondary electron spectra V4. Risk: charge-exchange cycling, high-energy tails (long runtime).

## M6 Physicochemical stage
Scope: dissociation channels, displacer, thermalisation (G6), solvated electron placement, output of `SpeciesInstance` list.
Tests: V8. Acceptance: initial species counts per 100 eV vs Geant4 `chem6` at t = 1 ps (pre-chemistry) within V11 criterion.

## M7 Chemistry: reaction network, diffusion, IRT
Scope: species/reaction TOML loaders (option3 table, script-checked against C++), IRT incl. partially diffusion-controlled sampling, erfc, box boundary, SBS oracle, ODE test harness. Closes G5, G7.
Tests: V9, V10, unit tests for reaction selection and boundaries. Acceptance: V9 for A+B and A+A over three D/R/k sets; IRT vs SBS agreement on a TDC case within 3 sigma.

## M8 G(t) end-to-end
Scope: orchestration, scoring G(t) with uncertainties, TOML config, CLI `dna run config.toml`, JSON output with metadata.
Tests: V11, V13. Acceptance: V11 for e- 1 MeV (or 100 keV and 1 MeV) and p 10 MeV against chem6 reference.

## M9 Performance
Profile, benchmark, optimise within scientific equivalence; scaling study threads 1..N; document.

## M10 Documentation & extensibility hardening
Guides: adding particle / process / reaction; example configs; mapping table C++ -> Rust.

## M11 Extensions (separate plans)
1-10 MeV electrons (Q1, deferred by decision), finite boxes/boundaries (UHDR), scavengers/oxygen, IRT-syn and parallel chemistry (E.4), additional particles.
