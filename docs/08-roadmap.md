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

- **M2 validation gate closed (2026-10-15, Geant4 11.4.1 reference committed in `validation/reference/`, provenance in `provenance.toml`; generator compiled and run by the project lead on Windows/MSVC).** `cargo test -p dna-sim --test geant4_reference -- --ignored --nocapture`: **4 passed, 0 failed**, no tolerance touched. V2/V3: 1206 comparisons (208 energies x 3 processes, plus the probabilities), largest deviation 0.0 % of the 1e-9/1e-6 tolerance (the cross sections agree to machine precision). V5 `mfp` (seed 1): 36 comparisons, 12 above 50 % of the tolerance, tightest 73.8 % (first-process fraction elastic at 150 eV). V5 `track` (seed 1): 40 comparisons, 3 above 50 %, tightest 81.1 % (excitation interactions per primary at 10 keV). Geant4 seed 1 against Geant4 seed 2: 70 comparisons, 2 above 50 %, tightest 80.7 % (track length at 1 keV): Geant4's own run-to-run scatter reaches 80 % of the V5 tolerance for single quantities, so a Rust-versus-Geant4 comparison at 70-80 % is not evidence of a bias. Rust (other seed) against the seed-2 Geant4 runs: mfp 36 comparisons / 4 above 50 % (tightest 59.5 %), track 40 / 2 (tightest 77.2 %). The gate was **re-run after the M3 code** (Sanche, Melton, solvation, cascade): same counts, because the M2 set-up (secondaries not transported, tracking cut 100 eV, primaries >= 150 eV) never reaches the new processes and the stream of those runs is unchanged.

## M3 Secondary electrons + low-energy electron processes
Scope: secondary stack, Sanche vib., Melton attachment, solvation/thermalisation cut (7.4 eV), full energy accounting incl. binding energy policy (Q5).
Tests: V5-V7 (range 1 keV-1 MeV, deposit spread, secondary spectra). Acceptance: full-track deposited-energy and secondary-spectra V4/V5 vs Geant4 `dnaphysics`/`slowing`. Risk: low-energy secondaries dominate runtime and physics.
**Status: implemented (branch `claude/inspiring-edison-v1kw1a`); M3 validation gate against Geant4 11.4.1 closed 2026-10-15 with two documented test amendments (below).**
- *M3 gate (2026-10-15).* `cargo test -p dna-sim --test slowing_down_reference -- --ignored --nocapture`: 2 passed. First run failed 3 comparisons, all diagnosed as artefacts of the comparison, none a model bug: (1) V2 Melton attachment at 4 eV, rel 1.63e-8 vs 1e-9: the value is the 1e-300 interior-zero floor; Geant4 evaluates it in mm^2 (1e-316, subnormal, spacing 4.9e-324 = 5e-8 relative) and the rounding of 1e-316 to that grid is exactly -1.63e-8; both are physically zero, so values below 1e-290 now compare equal. (2) V5 radial fractions outside 10 nm at 8 eV (z = 4.05) and inside 100 nm at 900 keV (z = 4.16): the error estimated from the Rust sample is invalid for tails carried by a few histories. Evidence: Geant4 seeds give 5.7e-5 / 4.7e-5 (8 eV) and 2.4e-5 / 0.5e-5 (900 keV); Rust with 6 x 5e5 histories gives 2.5e-5 (1.6 sigma from Geant4) and with 4 x 100 histories 1.5e-5. Comparisons with Kish effective sample size < 10 on the minority side are now skipped and logged (8 skipped of the radial family); `Z_CRIT = 4` unchanged. Skipped comparisons are no evidence either way: the radial tails at 8 eV and at 900 keV (<= 100 nm) are untested at the 2e-5 level. All other V2 rows, all 77 V5 means and all 28 spectra chi-squares pass (lowest p = 0.0136 at 900 keV class 4, cut 3.6e-5). The amendments were made after seeing the failing numbers (not pre-registered).
- *What exists.* `dna-physics`: `SancheVibrational` (`G4DNASancheExcitationModel`), `MeltonAttachment` (`G4DNAMeltonAttachmentModel`), `Solvation` (`G4DNAElectronSolvation` + `Meesungnoen2002`), `ProcessKind` with five cross-section processes, `ProcessTable` with solvation, `transport_electron` = LIFO secondary stack over the whole cascade, ledger over the cascade, `PhysChemEvent` (`physchem.rs`). `dna-sim`: scorers (primary and total track length, range, projected range, per-process counts, solvations per 100 eV, W-like ratio, secondary spectra by generation, radial deposit profile, level/shell counts, quirk counts), `PhysChemSink` + CSV writer, JSON schema v2, examples `slowing_down` and `rotation_effect`. Every model was re-read in the C++ (docs/02 B.7.5). Termination guards of M2 removed (Q14 closed); the safety limit `max_steps` returns an error. Process-table window rule `(low, high]` between models of one process was already implemented and unit-tested at exact boundary values and `nextafter` neighbours (`table.rs` tests); the real windows of the five processes and the closed solvation window are tested in `windows.rs`.
- *Discrepancy with the M3 brief, found by reading.* The Sanche "kill and deposit" branch (brief: lines ~249-280) is **commented out** in 11.4.1 (`:248-264`); the live code does nothing when `E - E_vib <= 0`. Reproduced and counted (`vibrational_noop`, unreachable in the window). `G4DNATransformElectronModel` is not used by any physics constructor. Details in docs/02 B.7.5.
- *Tests (all pass; thresholds written in the test headers before the first run).* Hand-computed cross sections (independent Python, `scripts/hand_cross_sections.py`) to 1e-9: Sanche at every node of the window and at 2.5, 31, 97.3 eV and at the closed 100 eV edge (protection `t (1 - 1e-12)`), partial cross sections of three energies including the 1e-18 nm^2 (= 1e-30 mm^2) zero floor, Melton at all interior nodes and at 4.1, 5.9, 6.1, 7.3, 11.7, 12.3, 12.8 eV, `r_mean` polynomial at nine energies. Windows and `nextafter` boundaries of Sanche, Melton, solvation and the assembled table. Sampling (Bonferroni families stated in `low_energy.rs`): Sanche level chi-square at 2, 7.4, 31, 100 eV (4 tests, p = 0.67, 0.68, 0.35, 0.70); Melton absorption at 7.5 eV, 10^6 first interactions (|z| < 0.8 for elastic/vibrational/attachment fractions); solvation displacement: 9 KS tests (D <= 0.0034, p >= 0.20), mean length equal to `r_mean` (|z| <= 1.4), zero-`r_mean` branch isotropy (p = 0.45, 0.41); draw counts and mapping of the uniforms checked with a scripted RNG; Melton construction fails on a non-zero tail (synthetic table). Ledger of the whole cascade at 100 eV, 1, 10, 100 keV, 1 MeV (normal run 10^4/10^4/2000/200/20 histories; `--ignored` run 10^4/10^4/10^4/2000/200 executed once): max |residual|/E_in = 5.7e-16, 9.1e-16, 3.8e-15, 1.4e-14, 2.5e-13 (V7 asks 1e-9, A.4 1e-12); every history ends with E_dep = E_in, no untracked energy, `N_solv + N_att = 1 + N_sec`. 1/2/8-thread bitwise equality of every per-history field incl. spectra, radial profile and physchem events at 100 eV, 1, 10, 100 keV; the physchem CSV is byte-identical for 1/2/8 threads and its row counts equal the interaction counts. The M2 Geant4 gate still passes (see M2 status).
- *Sensitivity of the new statistical tests (checked once by hand on temporary mutants, reverted):* scaling the solvation `sigma_1D` by 1 % is rejected by the mean-displacement-length test (the KS tests alone, p >= 0.03, do not see it); swapping the partial cross sections of Sanche levels 1 and 2 in the level selection is rejected by the chi-square test (chi2 = 4.2e4 for 7 dof at 2 eV).
- *Physical sanity (not a Geant4 comparison; thresholds from the a-priori section below).* Range strictly increasing, each step by > 5 combined standard errors; W-like quantity inside the pre-registered 18-36 eV window; solvations per 100 eV inside 3-6; first-generation spectrum falling.
- *Results (Rust only, seed 20270103, `cargo run --release -p dna-sim --example slowing_down`; files in `validation/rust_results/m3/`; `Geant4` rotation).*

| E | N | primary track [nm] | range [nm] | W = E/N_ion [eV] | solvations /100 eV | secondaries /history | mean 1st-gen. secondary [eV] | deposit share: exc / vib / binding / attach / solv [% of E] |
|---|---|---|---|---|---|---|---|---|
| 100 eV | 20000 | 26.34 +- 0.17 | 4.57 +- 0.02 | 19.85 +- 0.02 | 5.94 | 5.0 | 9.2 | 8.0 / 4.7 / 61.3 / 0.9 / 25.2 |
| 1 keV | 10000 | 89.3 +- 0.3 | 41.0 +- 0.2 | 19.53 +- 0.01 | 5.13 | 51.2 | 21.8 | 6.6 / 4.1 / 66.5 / 0.8 / 22.0 |
| 10 keV | 2000 | 2733 +- 13 | 1857 +- 13 | 20.42 +- 0.02 | 4.82 | 489.7 | 34.4 | 6.2 / 3.9 / 68.4 / 0.7 / 20.8 |
| 100 keV | 200 | 1.455e5 +- 0.015e5 | 9.96e4 +- 0.17e4 | 20.88 +- 0.03 | 4.71 | 4790 | 44.1 | 6.0 / 3.8 / 69.2 / 0.7 / 20.3 |
| 1 MeV | 20 | 4.23e6 +- 0.12e6 | 3.06e6 +- 0.14e6 | 21.01 +- 0.02 | 4.68 | 47606 | 52.3 | 5.9 / 3.8 / 69.4 / 0.7 / 20.3 |

  Per history at 1 MeV: 47606 ionisations, 5706 excitations, 2.6e5 vibrational excitations, 855 attachments, 2.28e6 elastic collisions, 46752 solvations; mean displacement of a solvated electron 11.5 nm. Quirk counters, **totals over the 20 histories** (about 9.5e5 ionisations and 9.4e5 solvations): 6 excitation no-ops, 48 shell-2 redraws, 2357 K-shell ionisations without de-excitation (0.25 % of the ionisations of the cascade: most ionisations are produced by low-energy secondaries), 8033 solvations of electrons with E <= 0.1 eV (the `r_mean = 0` branch, 0.86 % of the solvations; 979 of about 1.2e5, 0.83 %, at 100 eV); no Sanche no-op, no clamped transfer, no ionisation without a secondary.
- *Prediction versus result (written before the numbers were looked at, see below).* Identities and range order: as predicted. Solvations per 100 eV (4.7-5.1 for E >= 1 keV): inside the predicted 4-5 for E >= 10 keV, 5.1 at 1 keV. **W: predicted central ~25 eV (20-32 eV), found 19.5-21.0 eV**, i.e. at or slightly below the lower edge of the predicted band and inside the pre-registered test window 18-36 eV only by 1.5 eV at 1 keV. The estimate of the energy per ionisation was too high by about 20 %; a W of 20 eV means 69 % of E_in is booked as binding energy, which the model's heavy weight of the 10.8-16 eV shells makes plausible, and 100/W = 4.8 ionisations per 100 eV is in the range track-structure codes report for liquid water. This is a statement of consistency, not of agreement with Geant4: the Geant4 `slow` reference (below) is the test.
- *Elastic rotation (Q13), both conventions.* Direction never enters a cross section in the infinite medium and both conventions draw the same numbers, so for the same `(seed, history)` path lengths, energies, counts, spectra and W are **bitwise identical**; only positions differ. Paired difference `Exact - Geant4` of the primary's range (`rotation_effect` example): 100 eV -0.3 % (-0.015 +- 0.014 nm), 1 keV -0.48 % (-0.196 +- 0.048 nm, 4 sigma), 10 keV -1.16 % (-21.6 +- 4.7 nm, 4.6 sigma), 100 keV +0.1 % (+98 +- 707 nm), 1 MeV -0.7 % (-1.98e4 +- 6.3e4 nm); the last two are not resolved with 400 and 40 histories. Geant4's shrunken deflection makes tracks straighter, i.e. ranges ~0.5-1 % longer at 1-10 keV. The effect on the radial deposit profile and on G(t) (M7/M8) is not yet quantified. Default stays `Geant4` for comparisons (Q13); every physics result of M3 above is also stored for `Exact`.
- *Performance baseline* (release, 1 thread, Xeon 2.1 GHz class container, `cargo test --release -p dna-sim --test baseline -- --ignored --nocapture`, **whole cascade**): 1.8e6 / 1.8e6 / 1.8e6 / 1.8e6 / 1.7e6 interactions per second at 100 eV, 1, 10, 100 keV, 1 MeV (316, 2843, 26552, 261364, 2.6e6 interactions per history; 1 MeV: 1.5 s per history). The M2 baseline (primaries only) was 1.9e6 -> 7.7e5 /s because high-energy ionisations dominated; the cascade is dominated by cheap low-energy elastic/vibrational steps. No optimisation attempted; cost is linear in energy (about 2.6 interactions per eV). The ledger test at five energies costs about 5 min on 4 cores with full statistics, so the normal run uses fewer histories above 10 keV.
- *Not done / limitations.* **No Geant4 comparison of Sanche, Melton, solvation, the cascade, W, ranges, spectra exists yet.** The Rust side is ready (`slowing_down_reference.rs`, ignored) and the generator extended (`g4rs_ref slow`/`xslow`, syntax-checked against stub headers only: Geant4 is not available in this container). Not reproduced: Auger/fluorescence after K-shell ionisation (Q12; 0.5 % of ionisations; the reference is run with it off), Geant4's Gaussian generator (distribution only), chemistry-on branch of the thermalisation (25 meV proposal, navigator). The solvated-electron `r_mean` polynomial is a fit to data above 0.2 eV; its use up to 7.4 eV and the 1e-3 nm branch below 0.1 eV are Geant4 behaviour reproduced, not validated physics (M6). Energy deposition at the oxygen K shell is local (Q12), which shifts a few eV per ionisation from secondary Auger electrons into the binding term.

### M3 a-priori expectations (written and committed **before** any full-cascade W, solvation-count or range number was looked at)
These are the statements the physical-sanity tests are pre-registered against; they are reasoning, not Geant4 numbers.
- **Identities (exact, not statistical).** Every track ends by solvation or attachment, so `N_solv + N_att = 1 + N_secondaries` per history, and `E_in = E_dep` (infinite medium, everything transported).
- **W-like quantity `E_in / N_ion`** (ratio of totals). Reasoning: the energy per ionisation is the mean ionisation energy booked locally (about 17 eV with this shell mix, 14 % of it from the oxygen K shell, docs/09 Q12) plus the energy that goes into electronic excitations (about 0.4-0.5 excitations per ionisation at 8-14 eV, i.e. 4-6 eV), plus the kinetic energy of sub-ionisation electrons that is lost to vibrations and solvation (a few eV). That gives about 25-30 eV; gas-phase water has 30 eV, track-structure studies of the liquid report somewhat lower values, so I expect **20 eV < W < 32 eV, central ~25 eV, nearly energy independent above 1 keV**. Pre-registered test window: **18 eV <= W <= 36 eV for E in {1, 10, 100 keV, 1 MeV}**; 100 eV is only required to be finite and larger than the lowest ionisation energy (10.79 eV) because 100 eV cascades have 2-4 ionisations.
- **Solvations per 100 eV** `~ 100 (1 + 1/N_ion) / W - attachments`, i.e. about **4-5 per 100 eV** with attachments a few-percent correction (Melton cross section is ~1 % of Sanche at 8-12 eV, but ~1 in 10 of the 8-12 eV electrons are absorbed over the several tens of elastic/vibrational steps they make; I do not trust this estimate to better than a factor of 3).
- **Range.** The primary's full track length and straight-line range increase strictly with energy; the primary track length is shorter than the CSDA range (energy given to secondaries is not spent along the primary path) but of the same order: sub-micrometre at 1 keV, micrometres at 10 keV, 10^2 micrometres at 100 keV, millimetres at 1 MeV. No ESTAR comparison is made (L4 is a report-only item, docs/07 V6).
- **Secondary spectra.** The first-generation (delta electron) spectrum falls steeply with energy (roughly 1/W^2 above the binding energy); mean energy of first-generation secondaries of the order of 10-30 eV for keV primaries.

## M4 (optional) Electron options 4 and 6
Emfietzoglou, Uehara, CPA100; model transition tests at 10 keV / 250 keV. Only if Q2 says so.

## M5 Protons
Scope: p/H charge-state transport, Miller-Green, Rudd (Extended), Born, ion elastic, Dingfelder charge change; RPWBA (100-300 MeV) after Born; secondary electrons via M3. Closes audit G3.
Tests: V2-V7 for protons; stopping power vs PSTAR (L4 report). Acceptance: V5 range/stopping power; secondary electron spectra V4. Risk: charge-exchange cycling, high-energy tails (long runtime).

## M6 Physicochemical stage
Scope: dissociation channels, displacer, thermalisation (G6), solvated electron placement, output of `SpeciesInstance` list.
Tests: V8. Acceptance: initial species counts per 100 eV vs Geant4 `chem6` at t = 1 ps (pre-chemistry) within V11 criterion.
**Status: Rust M6 implementation complete on `m6-physchem-b`; stage-matched Geant4 gate (RR-001) run 2026-10-17 and FAILED (16 of 28 species/energy rows exceed z <= 4.5); M6 stays open, see "RR-001 result" below.** **Update (2026-10-17, M6 residual audit): after a source-proven change to the electron-hole recombination schedule the same gate passes (28/28 rows, unchanged criteria); see "M6 residual audit (final record)" below. The Rust means and failure counts in the paragraphs and tables between here and that section describe the engine before the fix.** Part A is unchanged. Part B adds fixed-order two-stage runs (`run_physchem_summary`, JSON schema `geant4-rs/dna-sim/run/v3`), a SHA-256 of the three embedded chemistry inputs, and initial-species scoring at global time 1 ps. The old physical-only v2 output remains available.
- *What exists.* New crate `dna-chemistry` (depends on `dna-core` only): `SpeciesTable` (15 species, D/charge/radius), `ChannelTable` (12 decay states, 23 channels, validated at load: probability sums to 1 within 1e-12, products and displacement names known, orbit = 4 - index, hydrogen/oxygen/charge balance with 0 or 1 extra water), `displacer` (port of `G4DNAWaterDissociationDisplacer`, cases reachable from option2 events), `params` (birth time 1 ps, auto-ionisation electron 1.7 eV, Onsager recombination parameters), `PhysChemEngine::convert_history(events, rng) -> species + stats`. Data in `data/chemistry/*.toml` with C++ line numbers; the shared record types (`PhysChemEvent`, `Species`, `SpeciesInstance`), the Gaussian helpers, the Meesungnoen2002 law and `ScriptedRng` moved to `dna-core` (re-exported by `dna-physics`, no behaviour change; the M3 suite passes unchanged).
- *Species identity.* The 15 TOML tags, extraction fixture keys, and emitted chemistry names are the exact Geant4 `G4MoleculeTable` configuration keys. `Species::OH` and `Species::HO2` are legal Rust identifiers for `°OH` and `HO2°`; the old ASCII spellings are accepted only by the explicit legacy parser. `H2O(B)`, `H3Op(B)`, `OHm(B)`, and `NoneM` are bookkeeping configurations outside `Species`.
- *Scope as decided (prompt A items 1, 2, 4).* Event -> species per history: solvation -> `e_aq` at the stored displaced position; excitation/ionisation/attachment -> channel selection + displacement + mother recoil; **electron-hole recombination of `G4DNAElectronHoleRecombination`** (found in the audit, docs/02 B.7.6 item 5; it is part of option3's pre-chemistry and was not in the brief): an ionised molecule recombines with its nearest alive `e_aq` within 7.12 nm with probability `1 - exp(-r_O/d)`, `r_O = 0.7116 nm`, and then decays through the `H2Ovib` channels. Species are born at **1 ps** (Geant4, not 0). Not done (session B): scoring, G-values, wiring into `dna-sim` runs and JSON, Geant4 `physchem` generator and comparison.
- *Tests (thresholds in the test headers, fixed before the first run).* (1) `cpp_crosscheck`: every number of the Rust data against a machine extraction of the C++ text (`scripts/extract_physchem_from_cpp.py`) and an independent Python evaluation (`scripts/hand_physchem.py`): species, 23 channels in order, probabilities, energies, products, displacement cases, displacer constants, Meesungnoen coefficients and `r_mean(1.7 eV) = 2.8152 nm`, Onsager radius (0.71164 nm, eps 80.0998, probabilities at 6 distances, 1e-12), elemental balance of all channels. (2) `channel_statistics`: Pearson chi-square of the sampled channel for the 6 multi-channel states, N = 400 000, Bonferroni alpha = 1.67e-4 (observed p = 0.057, 0.74, 0.51, 0.39, 0.96, 0.50). (3) `displacement_statistics`: 20 z-tests of the mean-square displacement of every product, separation or component against the analytic hand values, N = 200 000, |z| < 4.1 (largest observed |z| = 2.1), plus exact zeros for the oxygen atom and `OH-`. (4) `history`: draw counts per event type with a scripted generator, recombination probability decisions, nearest-alive rule, search radius, d = 0, switch, auto-ionisation electron availability, creation order, 1 ps, immutability of the event list, bitwise identical species lists for 1, 2, 8 threads. (5) `dna-sim/tests/physchem_a.rs` on real cascades (300 eV, 1 keV, 10 keV; 88 histories, 14 028 events, 1 405 recombinations): ledger conserved (1e-9) and the event deposits equal the ledger per kind, exact hydrogen/oxygen/charge conservation of the species of every history, e_aq balance, 1 ps, thread-count independence. (6) six deliberate mutants (nearest -> farthest, inverted recombination test, H2 fraction -15/18, channel scan without subtraction, mother shift omitted, Onsager at 298 K) were all rejected; reverted.
- *Informational, not asserted, not validated* (10 keV electrons, 20 histories, seed 20261017, E_dep = 2.0e5 eV; species per 100 eV at 1 ps): H3O+ 4.06, OH 5.00, e_aq 3.99, H 0.90, H2 0.23, O 0.17, OH- 0.09; 9 830 ionised molecules, 8 044 of them with an `e_aq` within the search radius, 2 132 (21.7 %) recombined. No Geant4 comparison exists yet: these numbers must not be quoted as results.
- *Intentional deviations / assumptions (docs/09 Q20-Q23, R14-R15).* Processing order of the water molecules (Q21); chemistry-stage decay deposits not added to `E_dep` (Q20); two temperatures reproduced (Q23); Gaussian generator and draw counts differ from `G4RandGauss` (distributions only); infinite medium (no navigator clipping).
- *Part B scores and definitions.* For every species: alive count, count per history, G-value as ratio of totals and mean of per-history ratios, production counts/G-values attributed to excitation/ionisation/attachment/solvation before a later recombination consumes an electron, RMS distance from the primary origin, and RMS displacement from its parent event (solvation excluded because the recorded event position is already displaced). Spatial correlations are the mean and RMS OH--e_aq and OH--H3O+ distances for products of the **same decay**, not an unbounded all-pairs statistic. Chemistry-stage relaxation energy remains a separate diagnostic and never changes the physical ledger.
- *A-priori sanity and result (not a validation).* Before running, the 10 keV bounds in `physchem_b.rs` were fixed to e_aq 2--6, OH 3--8, H3O+ 2--6, H 0.2--2, H2 0.1--1.5 molecules/100 eV: roughly 4.8 ionisations/terminal electrons per 100 eV make OH/H3O+/e_aq dominant, recombination lowers charged yields and adds fragments, and H2 is a minority branch. With 30 histories (seed 20261016): 4.010, 4.980, 4.093, 0.832, 0.237 respectively; all bounds pass. Event-order study (30 x 10 keV, seed 20261018): ionisation recombination fraction 20.84%, 0% with recombination disabled, 21.85% with reversed water-event order; total species changed, confirming Q21.
- *Tests.* Existing V8 channel chi-square, displacement mean-square, channel balance, ledger and atom/charge conservation tests remain green. New independent hand test checks both G estimators and RMS; two-stage summaries are bitwise equal with 1/2/8 threads. `physchem_reference.rs` is ignored until `physchem.csv` and `provenance_m6.toml` exist; its pre-registered count family uses combined z <= 4.5 plus 15% for non-small means, and RMS radius uses 20% (the requested CSV lacks fourth moments for a z test).
- *Performance.* The added stage/scoring test at 10 keV takes about 0.7 s for the three normal tests together on this machine (90 transported histories plus sensitivity and thread repeats); no optimisation was attempted. Memory remains bounded by the physics batch plus one history's species list; reduction is sequential in history order.
- *Geant4 gate failure (first observed).* The reference comparison first fails at exactly `1000 eV H2O2: z = 111.6`. The test stops at its first failed assertion, so this result hides the rest of the comparison; inspect the CSV and diagnostics before drawing conclusions about other energies or observables. The pre-registered count and RMS tolerances above remain unchanged.
- *Next steps (2026-10-16).* M6 closure is split into the seeds `m6-comparison-report`, `m6-diagnose-and-fix` and `m6-close-gate` in `docs/gps-backlog.md`. The reference data exist (Q24 is closed, see docs/09), but `cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture` fails at the first assertion described above; the gate is **open**.

### M6 snapshot audit and full comparison (2026-10-16)

*Pre-fix note (2026-10-17): every Rust number in this comparison, and in the sections up to "M6 residual audit (final record)", comes from the engine before commit `5df0aa1` (recombination schedule). They stay as the record of what was observed then; do not quote them as the current Rust result.*

The existing `physchem.csv` is an **after-zero-time-reactions** snapshot at global time 1 ps. It is not the dissociation-only list returned by Rust M6. The first `dt > 0` **pre-step** callback precedes the selected positive-time step, but follows every earlier zero-time step, including contact reactions. The CSV header, `provenance_m6.toml` and generator comment calling it "before diffusion/reactions" are historical labels contradicted by the source audit below; the evidence files and existing generator mode are preserved unchanged. This establishes a comparison-stage mismatch, not a numerical correction or closure of M6.

Sources were read from Geant4 **11.4.1** (`source/global/management/include/G4Version.hh:46,71-73`, version 1141 / `geant4-11-04-patch-01`). Paths and line numbers below are relative to that source tree:

1. `source/processes/electromagnetic/dna/management/src/G4Scheduler.cc`, `Stepping`: calculate the model time at lines 550-552; calculate the medium-process time at 581-584; choose medium processes on ties at 613-626. The pre-callback is at 663, global time advances at 676, medium `DoIt` runs at 681-682, and `ComputeTrackReaction` runs at 694-696 before the post-callback at 700-701. A zero step keeps global time at 1 ps; a later positive-step pre-callback sees its completed changes.
2. `source/processes/electromagnetic/dna/processes/src/G4DNAMolecularDissociation.cc`: `GetMeanLifeTime` clamps an expired decay to zero at 108-112; `DecayIt` selects a channel at 142-155, displaces products at 165-223 and builds daughter tracks at the mother's global time at 258-268. `source/physics_lists/constructors/electromagnetic/src/G4ChemDissociationChannels_option1.cc:194-465` supplies only H3Op, °OH, OHm, e_aq, H, H2 and Oxy, including the H2Ovib channels after electron-hole recombination. None can produce H2O2, HO2°, Om (O^-), or O2.
3. `source/processes/electromagnetic/dna/models/src/G4DNAIRTMoleculeEncounterStepper.cc:431-479` waits for the stopped water states, then returns the initialization sentinel or earliest reaction time minus current time. `source/processes/electromagnetic/dna/management/src/G4ITModelProcessor.cc:118-131` initializes IRT and obtains that time; `ComputeTrackReaction:186-197,336-354` calls `FindReaction` on collision steps, kills parents and merges daughters. There is no exclusion of zero-time collisions. Medium-process priority therefore finishes zero-time dissociations before subsequent zero-time chemical collisions.
4. `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc`: `Initialize:93-126` samples reaction times; `GetIndependentReactionTime:298-333` explicitly permits zero-time contact reactions (deterministically for type 0 inside the effective radius, probabilistically for type 1). `FindReaction:527-548` applies reactions whose times are <= the current global time, and `MakeReaction:486-508` creates products, samples their reactions and merges them. Thus chemical cascades can occur without elapsed diffusion time; their reaction-site placement can still affect spatial scores.
5. `source/physics_lists/constructors/electromagnetic/src/G4EmDNAChemistry_option3.cc:89-91` selects option1 dissociation; its reaction table includes °OH + °OH -> H2O2 (255-260), °OH + Oxy -> HO2° (170-174), °OH + OHm -> Om (283-288), and Oxy + Oxy -> O2 (183-187). These demonstrate reaction paths, not measured attribution of individual CSV molecules. Lines 722-741 register water dissociation/recombination and omit Brownian transportation for IRT; lines 754-766 select the IRT model. `validation/geant4/g4rs_ref/g4rs_ref.cc:184-210` snapshots the main list in the pre-callback only when the selected time step is positive.

The focused ignored diagnostic `m6_reference_contains_products_requiring_zero_time_chemistry` independently checks the seven-species union against the loaded decay channels (including recombined water) and confirms the positive reference-only species. H2O2, HO2°, Om and O2 occur at all four energies; rarer HO2m, O2m and O3 also occur in the reference. Product presence alone would not prove callback order; the scheduler/IRT trace above supplies that evidence.

The complete rerun used `cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture`, Rust seed **20261017**, and the unchanged Geant4 reference (seed **20261019**, option2 physics / option3 chemistry, IRT, fluo/Auger off). Means and standard errors below are molecules/history; signed z and delta are Rust minus Geant4. N/A radius means zero population. Missing reference rows are represented as zero, as in the diagnostic report. The original gate still fails first at `1000 eV H2O2: z = 111.55722825815035` (its assertion uses absolute z); the report completes all 60 species/energy rows.

**1 keV, 20000 histories**

| Species | Geant4 mean (SE) | Rust mean (SE) | signed z | count delta (%) | RMS nm, Geant4 / Rust |
|---|---:|---:|---:|---:|---:|
| H3Op | 4.0105e1 (2.4694e-2) | 4.1022e1 (2.3400e-2) | +26.940 | +2.285 | 34.842 / 34.836 |
| °OH | 4.9552e1 (3.0288e-2) | 5.1650e1 (2.8274e-2) | +50.625 | +4.233 | 34.930 / 34.906 |
| OHm | 8.3920e-1 (6.3894e-3) | 9.1225e-1 (6.7342e-3) | +7.869 | +8.705 | 35.197 / 35.151 |
| e_aq | 4.0201e1 (2.5444e-2) | 4.1109e1 (2.4380e-2) | +25.778 | +2.260 | 38.881 / 38.780 |
| H | 1.0277e1 (2.4968e-2) | 1.0232e1 (2.4952e-2) | -1.272 | -0.437 | 35.390 / 35.253 |
| H2 | 2.7784e0 (1.1565e-2) | 2.6898e0 (1.1362e-2) | -5.471 | -3.192 | 35.494 / 35.176 |
| H2O2 | 6.8935e-1 (6.1793e-3) | 0.0000e0 (0.0000e0) | -111.557 | -100.000 | 35.792 / N/A |
| HO2° | 2.7885e-1 (3.7169e-3) | 0.0000e0 (0.0000e0) | -75.022 | -100.000 | 36.422 / N/A |
| HO2m | 0.0000e0 (0.0000e0) | 0.0000e0 (0.0000e0) | N/A | N/A | N/A / N/A |
| Oxy | 1.4869e0 (8.3235e-3) | 2.0354e0 (9.8624e-3) | +42.498 | +36.884 | 35.166 / 35.243 |
| Om | 6.4650e-2 (1.8381e-3) | 0.0000e0 (0.0000e0) | -35.172 | -100.000 | 35.286 / N/A |
| O2 | 5.6900e-2 (1.6713e-3) | 0.0000e0 (0.0000e0) | -34.045 | -100.000 | 36.745 / N/A |
| O2m | 3.0000e-4 (1.2246e-4) | 0.0000e0 (0.0000e0) | -2.450 | -100.000 | 29.546 / N/A |
| O3 | 0.0000e0 (0.0000e0) | 0.0000e0 (0.0000e0) | N/A | N/A | N/A / N/A |
| O3m | 0.0000e0 (0.0000e0) | 0.0000e0 (0.0000e0) | N/A | N/A | N/A / N/A |

**10 keV, 2000 histories**

| Species | Geant4 mean (SE) | Rust mean (SE) | signed z | count delta (%) | RMS nm, Geant4 / Rust |
|---|---:|---:|---:|---:|---:|
| H3Op | 4.0274e2 (4.6188e-1) | 4.0768e2 (4.7997e-1) | +7.420 | +1.227 | 1559.751 / 1545.995 |
| °OH | 4.8727e2 (5.5305e-1) | 4.9761e2 (5.7102e-1) | +13.005 | +2.122 | 1562.922 / 1549.232 |
| OHm | 8.3220e0 (6.5397e-2) | 8.6785e0 (6.6455e-2) | +3.824 | +4.284 | 1563.021 / 1551.714 |
| e_aq | 3.9500e2 (4.5883e-1) | 4.0000e2 (4.7620e-1) | +7.560 | +1.266 | 1560.921 / 1547.136 |
| H | 8.6085e1 (2.4419e-1) | 8.5875e1 (2.5264e-1) | -0.596 | -0.243 | 1584.043 / 1564.680 |
| H2 | 2.4123e1 (1.0911e-1) | 2.3150e1 (1.0703e-1) | -6.366 | -4.033 | 1575.650 / 1561.998 |
| H2O2 | 3.9720e0 (4.4250e-2) | 0.0000e0 (0.0000e0) | -89.762 | -100.000 | 1593.522 / N/A |
| HO2° | 1.8380e0 (3.0064e-2) | 0.0000e0 (0.0000e0) | -61.137 | -100.000 | 1617.979 / N/A |
| HO2m | 1.0000e-3 (7.0693e-4) | 0.0000e0 (0.0000e0) | -1.415 | -100.000 | 1683.158 / N/A |
| Oxy | 1.2961e1 (7.9512e-2) | 1.6782e1 (9.1728e-2) | +31.468 | +29.472 | 1583.439 / 1566.622 |
| Om | 4.1150e-1 (1.4686e-2) | 0.0000e0 (0.0000e0) | -28.019 | -100.000 | 1585.336 / N/A |
| O2 | 3.1550e-1 (1.2730e-2) | 0.0000e0 (0.0000e0) | -24.783 | -100.000 | 1610.941 / N/A |
| O2m | 5.0000e-4 (5.0000e-4) | 0.0000e0 (0.0000e0) | -1.000 | -100.000 | 1561.459 / N/A |
| O3 | 0.0000e0 (0.0000e0) | 0.0000e0 (0.0000e0) | N/A | N/A | N/A / N/A |
| O3m | 0.0000e0 (0.0000e0) | 0.0000e0 (0.0000e0) | N/A | N/A | N/A / N/A |

**100 keV, 200 histories**

| Species | Geant4 mean (SE) | Rust mean (SE) | signed z | count delta (%) | RMS nm, Geant4 / Rust |
|---|---:|---:|---:|---:|---:|
| H3Op | 4.0195e3 (5.3236e0) | 4.0802e3 (5.1942e0) | +8.171 | +1.512 | 80848.560 / 81641.134 |
| °OH | 4.8037e3 (6.1877e0) | 4.9188e3 (6.1281e0) | +13.218 | +2.396 | 80905.883 / 81687.232 |
| OHm | 8.0840e1 (6.1466e-1) | 8.5210e1 (6.9212e-1) | +4.721 | +5.406 | 81027.811 / 81553.233 |
| e_aq | 3.9356e3 (5.3110e0) | 3.9960e3 (5.0426e0) | +8.252 | +1.536 | 80853.020 / 81649.863 |
| H | 7.9136e2 (2.3059e0) | 7.9397e2 (2.4673e0) | +0.773 | +0.330 | 81309.074 / 82059.606 |
| H2 | 2.2296e2 (1.0587e0) | 2.1806e2 (1.1225e0) | -3.169 | -2.193 | 81210.656 / 81844.095 |
| H2O2 | 3.4680e1 (4.5830e-1) | 0.0000e0 (0.0000e0) | -75.671 | -100.000 | 82205.105 / N/A |
| HO2° | 1.5335e1 (2.9632e-1) | 0.0000e0 (0.0000e0) | -51.752 | -100.000 | 81218.249 / N/A |
| HO2m | 0.0000e0 (0.0000e0) | 0.0000e0 (0.0000e0) | N/A | N/A | N/A / N/A |
| Oxy | 1.1850e2 (7.5694e-1) | 1.5316e2 (8.8661e-1) | +29.727 | +29.245 | 81399.385 / 82299.285 |
| Om | 4.0300e0 (1.3944e-1) | 0.0000e0 (0.0000e0) | -28.902 | -100.000 | 79063.427 / N/A |
| O2 | 2.9400e0 (1.1875e-1) | 0.0000e0 (0.0000e0) | -24.757 | -100.000 | 81579.610 / N/A |
| O2m | 0.0000e0 (0.0000e0) | 0.0000e0 (0.0000e0) | N/A | N/A | N/A / N/A |
| O3 | 0.0000e0 (0.0000e0) | 0.0000e0 (0.0000e0) | N/A | N/A | N/A / N/A |
| O3m | 0.0000e0 (0.0000e0) | 0.0000e0 (0.0000e0) | N/A | N/A | N/A / N/A |

**900 keV, 20 histories**

| Species | Geant4 mean (SE) | Rust mean (SE) | signed z | count delta (%) | RMS nm, Geant4 / Rust |
|---|---:|---:|---:|---:|---:|
| H3Op | 3.6134e4 (5.4737e1) | 3.6595e4 (5.8884e1) | +5.733 | +1.276 | 1848558.173 / 2045077.826 |
| °OH | 4.3086e4 (5.8528e1) | 4.4017e4 (6.6219e1) | +10.532 | +2.160 | 1849223.894 / 2045154.505 |
| OHm | 7.3995e2 (6.3302e0) | 7.6315e2 (4.1702e0) | +3.061 | +3.135 | 1847656.663 / 2049540.195 |
| e_aq | 3.5359e4 (5.3734e1) | 3.5832e4 (5.9910e1) | +5.884 | +1.339 | 1848535.110 / 2045007.440 |
| H | 6.9565e3 (2.2253e1) | 7.0090e3 (2.8967e1) | +1.439 | +0.755 | 1854213.330 / 2045268.547 |
| H2 | 1.9922e3 (8.2377e0) | 1.9256e3 (6.8704e0) | -6.204 | -3.341 | 1851068.373 / 2044105.558 |
| H2O2 | 2.9670e2 (3.0445e0) | 0.0000e0 (0.0000e0) | -97.455 | -100.000 | 1838265.173 / N/A |
| HO2° | 1.3350e2 (1.9541e0) | 0.0000e0 (0.0000e0) | -68.319 | -100.000 | 1873304.070 / N/A |
| HO2m | 5.0000e-2 (5.0000e-2) | 0.0000e0 (0.0000e0) | -1.000 | -100.000 | 2519987.879 / N/A |
| Oxy | 1.0408e3 (8.1714e0) | 1.3374e3 (9.1664e0) | +24.149 | +28.493 | 1853974.184 / 2041638.622 |
| Om | 3.5650e1 (1.0865e0) | 0.0000e0 (0.0000e0) | -32.813 | -100.000 | 1909806.206 / N/A |
| O2 | 2.5150e1 (1.1033e0) | 0.0000e0 (0.0000e0) | -22.796 | -100.000 | 1863886.838 / N/A |
| O2m | 1.0000e-1 (6.8825e-2) | 0.0000e0 (0.0000e0) | -1.453 | -100.000 | 2424983.188 / N/A |
| O3 | 5.0000e-2 (5.0000e-2) | 0.0000e0 (0.0000e0) | -1.000 | -100.000 | 1185859.632 / N/A |
| O3m | 0.0000e0 (0.0000e0) | 0.0000e0 (0.0000e0) | N/A | N/A | N/A / N/A |

All seven shared species have RMS differences below 11% (the unchanged limit is 20%). H is consistent in count at all four energies; H3Op, °OH and e_aq have small relative excesses but fail the z criterion; Oxy is 28.5-36.9% high and fails both count criteria. H2 and OHm also have z failures at some energies. Reaction-only populations are exactly zero in Rust M6, so their Rust RMS is undefined. The snapshot mismatch explains why a dissociation-only engine cannot reproduce the full reference, but it does not establish that all remaining count or spatial differences come from that mismatch. The count criteria remain **z <= 4.5 and 15% for non-small means**, and the RMS criterion remains **20%**; M6 stays open until a matching observable is validated.

### M6 cascade and scoring audit (2026-10-16)

No transport or scoring defect was established. The following settings were traced in the checked-in generator and the read-only Geant4 **11.4.1** source; they are equivalent for the four committed primary energies (1/10/100/900 keV). This is configuration evidence, not a claim of identical random streams or closure of the snapshot mismatch above.

| Setting | Rust evidence | Geant4 / generator evidence |
|---|---|---|
| Complete electron cascade | `RunConfig::new` enables secondaries without a tracking cut or interaction limit; `transport_electron` drains its LIFO stack, ends tracks by solvation/attachment, and only checks the primary's input range. | `g4rs_ref.cc:404-425` returns `fUrgent` for every physchem secondary and starts chemistry only after the physical stack empties. `gCut` stays zero in this mode, so its primary-only stepping action cannot cut an electron at positive energy. |
| Q12: fluorescence/Auger disabled | `born_ionisation.rs:173` counts skipped K-shell de-excitation; the full binding energy stays local. | `g4rs_ref.cc:689-694,709-715` disables fluo/Auger/cascade/PIXE and removes the atomic de-excitation object after initialization. `source/processes/electromagnetic/dna/models/src/G4DNABornIonisationModel1.cc:525-581` generates de-excitation particles only when that object is non-null, otherwise retaining the binding energy deposit. |
| Q13: Geant4 elastic rotation | The comparison constructs `ProcessTable::option2_electron(..., ElasticRotation::Geant4)`. | `source/physics_lists/constructors/electromagnetic/src/G4EmDNAPhysics_option2.cc:84` selects option 2; `G4EmDNABuilder.cc:336-351` in the same directory installs Champion. `source/processes/electromagnetic/dna/models/src/G4DNAChampionElasticModel.cc:297-308` uses the unnormalised `orthogonal()` axis and normalises the final direction. Both primaries start at zero along +z. |
| Q16: 2 eV primary validity floor | `ProcessTable::primary_energy_range` gives 2 eV; the check is before pushing the primary, never reapplied to a secondary. Sub-floor electrons still solvate. | Geant4 has **no matching 2 eV tracking cut** in this generator. All reference primaries are above the Rust validity floor; imposing it on secondary tracks would break equivalence. This remains a Rust input-domain policy, not a Geant4 physical threshold. |
| Q17: Meesungnoen2002 | `option2_electron` installs `Solvation::new`, whose shared penetration law is Meesungnoen2002. | `G4EmDNABuilder.cc:329-333` installs the factory model. `source/processes/electromagnetic/utils/src/G4EmLowEParameters.cc:78` defaults to Meesungnoen2002; `source/processes/electromagnetic/dna/models/src/G4DNAOneStepThermalizationModel.cc:292-308` resolves that subtype to the liquid-water model. No generator override exists. |

Scoring uses the **alive population at global 1 ps**, not a cumulative production counter or a 1 ps age added to each physical interaction. `source/processes/electromagnetic/dna/utils/src/G4DNAChemistryManager.cc:645-659,680-694` builds water/electron tracks at `picosecond + delayedTime`; this generator has no pulse delay. Rust assigns the fixed birth time to every converted species. `PhysChemScorer` takes counts after electron-hole recombination and radius from the surviving species list. The generator counts the main list at its first positive-step pre-callback; `source/processes/electromagnetic/dna/management/src/G4ITModelProcessor.cc:336-354` ends killed parents, merges daughters and removes killed tracks before the next callback. That list also includes the zero-time chemical changes established above, which Rust M6 does not perform.

Both radius estimators accumulate squared position norms from the **primary origin**, which is zero in both runners (`RunConfig::primary`, `g4rs_ref.cc:340-345,205-207`), and divide by the total alive population before taking the square root. This differs from per-event product displacement, which remains a separate observable, and from averaging per-history RMS values. The deterministic `physchem_counts_and_radius_use_survivors_at_global_one_ps` fixture places an ionisation 100 nm from zero, consumes its nearby electron, and leaves a survivor at (0,6,8) nm: produced e_aq = 2, alive e_aq = 1, RMS = 10 nm. Its H/OH scores compare origin radius and parent displacement separately, while all products remain at global 1 ps. `physchem_cascade_follows_all_electrons_without_a_secondary_floor` checks completed histories, tracks = 1 + secondaries = solvations + attachments, zero abandoned energy, sub-2-eV solvation, and acceptance/rejection at the primary floor. No physics model, tolerance, reference CSV/provenance or generator mode was changed.

Audit verification: `cargo test -p dna-sim physchem` passed (five matching tests, including both new checks). `cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture` completed in 255.40 s: **3 passed, 1 failed**, exit 101. The 60-row comparison and ordering diagnostic completed unchanged; the original asserting gate still failed first at `1000 H2O2: z=111.55722825815035`. This is the expected retained gate result, not M6 closure. Required checks passed: formatting, workspace Clippy with warnings denied, and workspace tests (**268 passed, 12 ignored**). Command outputs are in the session scratch directory (`ticket03-reference.log`, `ticket03-workspace.log`).

### M6 decision after the three audits (2026-10-17): awaiting data

The final rerun of `cargo test -p dna-sim --test physchem_reference m6_initial_species_counts_and_rms_match_geant4 -- --ignored --exact --nocapture` still fails at `1000 H2O2: z=111.55722825815035` (exit 101). **No scientific correction was justified by the conversion, cascade or scoring audits.** The count limits remain combined-standard-error **z <= 4.5** and **15%** when both means exceed 0.1/event; the RMS-radius limit remains **20%**. The complete four-energy comparison above remains the evidence for the other populations, including Oxy's 28.5-36.9% excess. A comparison across different stage boundaries cannot validate M6, even for the shared species whose radii satisfy the numeric limit.

The exact open condition is to compare the surviving species after water dissociation and electron-hole recombination, at global 1 ps, **before IRT chemical reactions**, and pass those same limits. Aggregate post-reaction counts and summed squared radii cannot reconstruct the removed parents, reaction sites or pre-reaction populations. The ordering ablations measure a Rust sensitivity; they cannot identify the Geant4 pre-reaction result or establish the equivalence of its cached recombination candidates (Q21/R14). M6 is not closed, and M7 work that consumes these initial conditions remains gated.

RR-001 in `validation/geant4/RUN_REQUESTS.md` requests new `physchem_prechem.csv` and `provenance_m6_prechem.toml`, produced by the separately named `g4rs_ref physchem_prechem` mode. It keeps option2 physics, option3 chemistry, IRT selection, secondaries, and atomic de-excitation settings unchanged. Its new post-step action stops on the final zero-time water decay, after daughters merge and killed parents are removed (`G4ITStepProcessor2.cc:110-185`), before the next model time selection initializes IRT (`G4DNAIRTMoleculeEncounterStepper.cc:431-467`, `G4ITModelProcessor.cc:118-131`; all under `source/processes/electromagnetic/dna/{management,models}/src/`). It fails the run if a chemical reaction or positive-time step precedes capture, a snapshot is missing, or the captured tracks are not alive at 1 ps. No reaction table, dissociation channel, recombination process, or previous generator mode was changed.

The consuming `m6_prechem_counts_and_rms_match_geant4` test is ignored pending that user-run data. Its 28 comparisons (seven dissociation species at four energies) reuse the exact original count and RMS assertions. It first requires complete requested statistics, valid count moments, provenance and the explicit snapshot marker, and rejects reaction-only species. These checks and criteria were written before any new data exist. A pass validates this boundary; a failure must be investigated with the unchanged criteria, especially Q21/R14. The original CSV, provenance and gate remain historical evidence and are not replaced. The new C++ mode was compiled and linked against installed Geant4 11.4.1 with MSVC; **no Geant4 executable was run by this session**. Build/run instructions and the runtime-unverified status are in `validation/geant4/{README,BUILD_NOTES}.md`.

### RR-001 result (2026-10-17): data committed, gate FAILED

*Pre-fix: the Rust means below (and the 16 failing checks) are those of the engine before commit `5df0aa1`. After that commit the same test passes with 0 failing checks; see "M6 residual audit (final record)". The Geant4 means and the failure list are kept unchanged as the reference evidence.*

The project lead authorised the run in a later session. `g4rs_ref physchem_prechem` (generator commit `ad1aceb`, clean tree, MSVC 19.50.35729, Geant4 11.4.1, seed 20261020) exited 0 with all four energies complete (20000/2000/200/20 events) and 28 data rows. A smoke case (1 keV, 200 events, seed 1) passed first; its CSV was not kept. The runtime guards did not fire, so no chemical reaction precedes the snapshot. Data: `validation/reference/physchem_prechem.csv` (commit `e808b2f`), `provenance_m6_prechem.toml` (`ffe4b48`).

`cargo test -p dna-sim --test physchem_reference m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact --nocapture` passed provenance, boundary and completeness checks and **failed the comparison** with the unchanged criteria (z <= 4.5, 15% count, 20% RMS): 16 failing checks, all in the z test; none in the 15% count or 20% RMS tests. The test was changed (commits `9e03af8`, `0f833fa`) only to parse the provenance as a TOML document and to report every failing row; no criterion or tolerance changed.

| Energy | Failing rows (z; Geant4 vs Rust mean per event) |
|---|---|
| 1 keV | H3O+ 21.36 (40.30 / 41.02); e_aq 20.76 (40.38 / 41.11); H 12.90 (10.69 / 10.23); O 7.26 (2.138 / 2.035); OH 6.63 (51.38 / 51.65); H2 6.25 (2.792 / 2.690) |
| 10 keV | H 7.48 (88.55 / 85.88); H2 5.36 (23.97 / 23.15); e_aq 5.14 (396.6 / 400.0); H3O+ 4.97 (404.4 / 407.7) |
| 100 keV | e_aq 6.27 (3950.8 / 3996.0); H3O+ 6.12 (4035.3 / 4080.2); H 5.26 (812.9 / 794.0); H2 4.55 (225.3 / 218.1) |
| 900 keV | H 4.99 (7190.4 / 7009.1); O 4.66 (1389.4 / 1337.4) |

The 12 other rows pass. Differences are small in absolute terms (H3O+/e_aq up to +1.8% at 1 keV, H 2-4% low, H2 and O up to ~5% low) but exceed the combined standard errors. Rust is below Geant4 for H, H2 and O and above for H3O+ and e_aq. The earlier 111-sigma H2O2 failure was a stage mismatch: with the boundaries matched, the large discrepancies disappear and a small systematic residual in the dissociation/ionisation partition remains. Candidate causes (not yet tested): ionisation/excitation balance of the cascade, dissociation channel fractions, electron-hole recombination and its cached candidates (Q21/R14). The gate is not loosened and M6 is **not closed**; M7 work that consumes these initial conditions stays gated. Next step: a separate seed investigates the residual, comparing the ionisation, excitation and attachment counts per energy with Geant4 before touching any model.

Final verification: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` all exit 0 (**268 passed, 13 ignored**). The original asserting gate separately exits 101 with the same z failure (6.64 s); the new ignored consumer separately exits 101 with `RR-001 awaiting data: physchem_prechem.csv` (0.06 s), and the existing zero-time-chemistry diagnostic passes. These two expected ignored-test failures are reported separately from the passing normal suite. Logs and build artifacts are in `.scratch/tests/2026-10-16__m6-diagnose-and-fix/` (`ticket04-workspace.log`, `ticket04-original-gate.log`, `ticket04-awaiting-data.log`, `ticket04-semantics.log`, `ticket04-geant4-build*.log`). `validation/reference/` has no changes. Session outcome: **awaiting data**, not M6 closure.

### M6 residual audit: source comparison (2026-10-17, ticket 02; recorded in the final section below)

Rust physicochemical stage compared with the Geant4 **11.4.1** `.cc` sources, read 2026-10-17 in the read-only checkout (nothing there was modified). The scope follows the ticket-01 harness (`crates/dna-sim/tests/physchem_audit.rs`, commit `b30beb5`). Charge and H/O conservation hold exactly on both sides, so they do not separate stage families. What remains is a Rust excess of O atoms per event (+0.9 % at 1 keV, z +21), H3O+/e_aq high and H/H2/O low. The rows therefore cover the event-to-state mapping, the channel fractions, dissociative attachment (DA) and electron-hole recombination. Marks: **identical** (same rule and numbers), **differs** (different rule; a candidate only, the verdict belongs to ticket 03), **undecidable** (static reading does not settle it).

Path prefixes. Geant4: `D/` = `source/processes/electromagnetic/dna/`, `PL/` = `source/physics_lists/constructors/electromagnetic/src/`, `CH` = `PL/G4ChemDissociationChannels_option1.cc`, `EHR` = `D/processes/src/G4DNAElectronHoleRecombination.cc`, `WDD` = `D/processes/src/G4DNAWaterDissociationDisplacer.cc`, `MD` = `D/processes/src/G4DNAMolecularDissociation.cc`, `CM` = `D/utils/src/G4DNAChemistryManager.cc`. Rust: `T` = `data/chemistry/dissociation_option3.toml`, `P` = `data/chemistry/physchem_option3.toml`, other paths relative to `crates/`.

**A. Physics event to water state**

| # | Item | Rust file:line | Geant4 file:line | Mark | Note |
|---|---|---|---|---|---|
| A1 | Excitation level `k` from the partial cross sections; molecule created even when `E <= E_exc` (no-op) | `dna-physics/src/models/born_excitation.rs:77-104`, `dna-physics/src/transport.rs:512-518` | `D/models/src/G4DNABornExcitationModel1.cc:191-209` | identical | `CreateWaterMolecule` sits outside the `newEnergy > 0` branch |
| A2 | Level `k` empties orbit `4-k` and fills orbit 5: k = 0..4 -> A^1B_1, B^1A_1, Exci3rdLayer, Exci2ndLayer, Exci1stLayer | `dna-chemistry/src/channels.rs:198-205,312-317,341-345`; `T:27,48,90,111,132` | `CM:636-637`; `D/molecules/management/src/G4MolecularConfiguration.cc:554-564`; `CH:237-240,283-287,312-317,338-342,351-366` | identical | |
| A3 | Ionisation shell `s`; no molecule if `k < B`; shell-2 redraw below 19 eV | `dna-physics/src/models/born_ionisation.rs:125-145`, `transport.rs:519-526` | `D/models/src/G4DNABornIonisationModel1.cc:461-472,589-592` | identical | |
| A4 | Shell `s` empties orbit `4-s` (Ioni5..Ioni1) | `channels.rs:312-317,346-350`; `T:149-217` | `CM:639-640`; `G4MolecularConfiguration.cc:571-598`; `CH:376-408` | identical | all five states share one channel: the shell split cannot change counts |
| A5 | K-shell ionisation with de-excitation off: no extra molecules | `born_ionisation.rs:173-179` | `G4DNABornIonisationModel1.cc:527-569` | identical | the reference runs with fluo/Auger off (Q12) |
| A6 | DA: absorbed electron -> H2O^- (orbit 5 +1) -> `DissociAttachment_ch1` | `dna-physics/src/models/melton.rs:82-97`, `transport.rs:527-533`, `channels.rs:318,351-357`, `T:219-230` | `D/models/src/G4DNAMeltonAttachmentModel.cc:73,255-261`; `CM:633-634`; `CH:413-426` | identical | `fDissociationFlag` defaults to true |
| A7 | Vibrational excitation and elastic scattering create no molecule | `dna-core/src/physchem.rs:17`, `transport.rs:534` | `D/models/src/G4DNASancheExcitationModel.cc:228-286` (no `CreateWaterMolecule`) | identical | |
| A8 | Molecule at the post-step point, born at 1 ps, lifetime 0, products at the mother's time | `transport.rs:467,515`; `P:9`; `dna-chemistry/src/engine.rs:277-282` | `source/tracking/src/G4SteppingManager.cc:198-219`; `CM:655-656`; `D/molecules/types/src/G4H2O.cc:65`; `MD:108-113,258` | identical | |

The per-history counts of excitation, ionisation, attachment and solvation were compared with Geant4 at 1, 10, 100 and 900 keV in the M3 gate (`dna-sim/tests/slowing_down_reference.rs:66-75,407-411`, `validation/reference/slow_sums.csv`; all passed at z < 4). That run had no chemistry, and the per-level and per-shell split of the full cascade was not compared there.

**B. Dissociation channels and selection**

| # | Item | Rust file:line | Geant4 file:line | Mark | Note |
|---|---|---|---|---|---|
| B1 | A^1B_1: relax 0.35 (8.22 eV); OH + H 0.65 (A1B1) | `T:23-42` | `CH:225-242` | identical | |
| B2 | B^1A_1: relax 0.175 (10.00 eV); H2 + 2 OH 0.0325 (B1A1); OH + H3O+ + e_aq 0.50 (AutoIoni); H + OH 0.2535 (A1B1); O + 2 H 0.039 (B1A1_2) | `T:44-84` | `CH:246-292` | identical | same order, which matters for the scan (B10) |
| B3 | Exci3rdLayer: AutoIoni 0.5; relax 0.5 (11.24 eV) | `T:86-105` | `CH:296-319` | identical | |
| B4 | Exci2ndLayer: AutoIoni 0.5; relax 0.5 (12.61 eV) | `T:107-126` | `CH:323-344` | identical | |
| B5 | Exci1stLayer: AutoIoni 0.5; relax 0.5 (13.77 eV) | `T:128-147` | `CH:348-368` | identical | |
| B6 | Ioni1..Ioni5: H3O+ + OH, 1.0 | `T:149-217` | `CH:376-408` | identical | |
| B7 | DA: H2 + OH- + OH, 1.0 | `T:219-230` | `CH:413-426` | identical | |
| B8 | H2Ovib: H2 + 2 OH 0.1365; OH + H 0.3575; O + 2 H 0.156; relax 0.35 (0 eV) | `T:232-264` | `CH:431-465` | identical | |
| B9 | Relax energies = excitation structure; ionisation energies | `T:33,54,103,124,145`; `dna-physics/src/water.rs:18,22` | `D/utils/src/G4DNAWaterExcitationStructure.cc:31-35,45-52`; `D/utils/src/G4DNAWaterIonisationStructure.cc:31-35`; `CH:228,253,308,335,363` | identical | chemistry-stage deposit only, no effect on counts |
| B10 | One uniform per decay; `u < p` selects, else `u -= p`; overrun keeps the last channel | `channels.rs:119-128`, `engine.rs:268-269` | `MD:142-155` | identical | |

**C. Displacements** (product counts do not depend on them; they enter only through recombination distances)

| # | Item | Rust file:line | Geant4 file:line | Mark | Note |
|---|---|---|---|---|---|
| C1 | `radialDistributionOfProducts(R)`: three Gaussians, sigma = R/sqrt(3) | `dna-chemistry/src/displacer.rs:50-55` | `WDD:683-691` | identical | same distribution, different Gaussian generator (docs/09) |
| C2 | Mother shift RMS 2.0 nm for ionisation and auto-ionisation, 0 otherwise, added to every product | `displacer.rs:131-135` | `WDD:182-231`; `MD:175-177,207` | identical | |
| C3 | Ionisation and auto-ionisation: `u < 0.5` -> OH 0.8 nm, else H3O+ 0.8 nm | `displacer.rs:112-126` | `WDD:248-281,379-410` | identical | |
| C4 | Auto-ionisation e_aq: Meesungnoen2002 penetration at 1.7 eV | `displacer.rs:127-129`; `P:14` | `WDD:144,411-413,696-721`; `D/models/src/G4DNAOneStepThermalizationModel.cc:123-147` | identical | |
| C5 | A1B1: 2.4 nm axis, OH -1/18, H +17/18 | `displacer.rs:74-83` | `WDD:282-302` | identical | |
| C6 | B1A1 and DA: 0.8 nm axis, H2 -16/18, each OH +2/18 axis +- 0.5 x (1.1 nm); OH- not displaced | `displacer.rs:84-101` | `WDD:303-346,417-457` | identical | |
| C7 | B1A1_2: O fixed, each H +- 0.5 x (1.6 nm) | `displacer.rs:102-111` | `WDD:347-378` | identical | |
| C8 | Navigator clip to 0.8 x safety | `engine.rs:277-282` (none) | `MD:211-223` | identical | infinite-medium limit; shortens displacements, never changes counts |

**D. Electron-hole recombination**

| # | Item | Rust file:line | Geant4 file:line | Mark | Note |
|---|---|---|---|---|---|
| D1 | Only charge +1 water (ionised states) can recombine | `engine.rs:357` | `EHR:217-220` | identical | excited states have charge 0, DA -1 |
| D2 | Onsager radius `e^2/(4 pi eps0 k_B T eps(rho,T))`, T = 293.15 K, rho = 1 g/cm^3 | `dna-chemistry/src/params.rs:88-110`; `P:16-38` | `EHR:51,57-104,222-229` | identical | numbers cross-checked to 1e-12 at M6 (`cpp_crosscheck`) |
| D3 | Search radius 10 r_O (inclusive); `p = 1 - exp(-r_O/d)`, 1 at d = 0 | `params.rs:110,114-120`; `engine.rs:198,360-364` | `EHR:280-306`; `D/management/include/G4KDTree.icc:172,411-418` | identical | |
| D4 | Candidates sorted by distance (KD tree; the octree branch is used only by IRT_syn) | `engine.rs:175-208` | `EHR:238,280-293`; `D/management/src/G4KDTreeResult.cc:69,112`; `D/models/src/G4DNAIndependentReactionTimeModel.cc:85`; `PL/G4EmDNAChemistry_option3.cc:757-767`; `validation/geant4/g4rs_ref/g4rs_ref.cc:795` | identical | IRT selects `G4DNAMolecularIRTModel` |
| D5 | Recombination is tried before the decay of an ionised molecule | `engine.rs:357-375` | `PL/G4EmDNAChemistry_option3.cc:732-741`; `D/management/src/G4ITStepProcessor.cc:561-575`; `EHR:336-344`; `MD:108-113` | identical | both lifetimes 0; the first strict minimum of the at-rest loop wins (`P:18`, docs/02 B.7.6) |
| D6 | Recombined molecule -> H2Ovib, partner e_aq killed | `engine.rs:364-368`; `T:232-264` | `EHR:183-201`; `CH:459-465` | identical | |
| D7 | Candidate set at the first test | solvated e_aq plus the auto-ionisation e_aq of excitations earlier in event order (`engine.rs:331-343,401-406`) | KD tree rebuilt from the main list at each zero-time step (`D/models/src/G4DNAIRTMoleculeEncounterStepper.cc:80-87`; `D/management/src/G4ITModelProcessor.cc:95-107,146-161`; `D/management/src/G4VITStepModel.cc:69-72`; `D/management/include/G4ITFinder.icc:213-273`); every `FindReactant` of a step runs before its DoIts (`D/management/src/G4Scheduler.cc:550-552,584`); decay products join after the step (`D/management/src/G4ITStepProcessor2.cc:184-185`) | differs | in Geant4's first step only solvated e_aq are candidates. Rust can pair an ion with an auto-ionisation e_aq at once, which raises the Rust count |
| D8 | Nearest e_aq already consumed in the same pass | nearest alive e_aq, tested with the ion's own uniform (`engine.rs:358-364`) | `FindReactant` draws `u` and tests the cached nearest (`EHR:235,290-308`); `MakeReaction` tests the first still-alive cached candidate with the same `u` (`EHR:164-181`) | identical | in both, the in-pass probability is p(nearest alive among the candidates); the candidate sets differ (D7) |
| D9 | Second test after a failed `MakeReaction` | none: one test, then the Ioni channel (`engine.rs:357-375`) | the ion stays `fStopButAlive` in its ionised state (`EHR:205-208`); at the next zero-time step `GetMeanLifeTime` calls `FindReactant` again with a new uniform on the rebuilt tree, which now holds the auto-ionisation e_aq and lacks the consumed ones (`EHR:235,336-344`; `G4DNAIRTMoleculeEncounterStepper.cc:80-87`; `G4ITStepProcessor2.cc:184-185`) | differs | possible only when p_alive < u < p_nearest. This raises the Geant4 count. Inferred from the at-rest selection, not traced at run time |
| D10 | Processing order within a pass | event order (`engine.rs:345-348`) | reverse main-list order (`G4ITStepProcessor2.cc:122-156`); how the main-list order relates to the physical event order is not established (docs/09 Q21) | undecidable | decides which ion takes which e_aq |

**Thermalisation follow-up** (the original residual-audit relations did not point here; N1 was subsequently audited below)

| # | Item | Rust file:line | Geant4 file:line | Status | Note |
|---|---|---|---|---|---|
| N1 | Thermalisation: electron at or below 7.4 eV -> e_aq at a Meesungnoen2002-displaced position | `dna-physics/src/models/solvation.rs:107-143`, `transport.rs:427-450` | `D/models/include/G4DNAOneStepThermalizationModel.hpp:166-247`; `CM:668-695` | audited 2026-10-17 | same sampled displacement law and event position; active `safety <= displacementMag` clip is inapplicable to infinite water and operationally unreachable in the committed reference workload (quantified below). It cannot explain the 900 keV RMS excess |
| N2 | Solvation count and e_aq creation before the water decays | `engine.rs:360-371` | `CM:677-695` | not audited; out of scope | `n_solvation` passed the M3 gate (see A); the existing terminal-track identity is checked, without extending this audit to N2 |

**Reading of the RR-001 residual** (consistency check for ticket 03, not a verdict). Suppose Rust performs Delta fewer recombinations per event than Geant4. The channel tables (B6, B8) then fix the shift caused by each missing recombination: H3O+ +1, e_aq +1, OH +1 - (2 x 0.1365 + 0.3575) = +0.3695, H -(0.3575 + 2 x 0.156) = -0.6695, H2 -0.1365, O -0.156, O atoms +1.2135. The RR-001 shifts at 1 keV (Rust - Geant4 per event) are H3O+ +0.72, e_aq +0.73, OH +0.27, H -0.46, H2 -0.102 and O -0.103. With Delta = 0.72 the tables predict OH +0.27, H -0.48, H2 -0.098, O -0.112 and +0.87 O atoms per event, which is the +0.9 % found in ticket 01. A shift in the ionisation count alone would move OH as much as H3O+ (+0.72) and leave H unchanged, which the data do not show. At higher energies the fit is looser: at 10 keV, H3O+ +3.3 predicts H -2.2 and H2 -0.45 against -2.67 and -0.82 observed; at 100 keV, H3O+ +44.9 predicts H -30 against -18.9 observed. D7, D9 and D10 are therefore the first candidates for ticket 03. Caveat: D7 and D9 act in opposite directions, and the source alone does not give the sign of the net effect. No source difference was found in the mapping (A), the channel fractions (B), DA (A6, B7) or the displacer (C).

### M6 residual audit: verdict and fix (2026-10-17, ticket 03)

**Verdict: the electron-hole recombination schedule (D7, D9, D10) is the cause, proven from the source and confirmed by the data. It is fixed.** The ticket-01 relations hold on both sides, so they cannot name a stage. The verdict instead rests on the source trace below and on a quantitative test of the three rows.
- **D9** (proven by tracing the selection code; no run-time trace): a failed `MakeReaction` leaves the ion `fStopButAlive` and ionised (`EHR:205-208`). The next zero-time step selects its rest process again (`D/management/src/G4ITStepProcessor.cc:535-586,968-974`). `AtRestGPIL` then calls `GetMeanLifeTime`, which calls `FindReactant` with a new uniform (`D/management/src/G4VITRestDiscreteProcess.cc:106-131`, `EHR:336-344`).
- **D7**: decay products join after the step (`MD:258-268`, `G4ITStepProcessor2.cc:184-185`), and the KD tree is rebuilt at every zero-time step (`G4DNAIRTMoleculeEncounterStepper.cc:80-87`).
- **D10, now decided**: the main list holds one sub-list per configuration ID, added in ascending ID at the start of each event (`D/management/src/G4ITTrackHolder.cc:288-356,401-443,620-658,725-739`, `D/management/include/G4ManyFastLists.hh:91-94`, `D/molecules/management/include/G4Molecule.hh:130-133`). The IDs follow the creation order (`D/molecules/management/src/G4MolecularConfiguration.cc:198-203`) of `CH:237-459`, which is the state order of `T`. DoIt walks the list backwards (`G4ITStepProcessor2.cc:122-156`). Inside a sub-list the molecules keep their creation order (`CM:627-662`). This order matches the Rust event order if the Rust LIFO stack (`transport.rs`) follows Geant4's tracking order, as that file states.

The test re-converted the same Rust physics histories with each candidate schedule, using a scratch prototype that is not committed. It scored the 28 RR-001 rows with the unchanged gate criteria at the gate seed 20261017 and at the five ticket-01 seeds.

| Schedule | Failing checks (6 seeds) | RMS of the 28 z (6 seeds) | Recombinations/event at 1 keV |
|---|---|---|---|
| Previous Rust (sequential, event order) | 11-16 | 7.06-7.89 | 12.547 |
| D7 only (pass structure, no retest) | 13-15 | 7.06-7.69 | 12.518 |
| D10 only (Geant4 order, sequential) | 12-16 | 7.26-7.77 | 12.555 |
| D7 + D9, event order | 0-2 | 1.60-1.99 | 13.084 |
| D7 + D9, reverse event order | 0 | 0.91-1.37 | 13.255 |
| **D7 + D9 + D10 (Geant4 order)** | **0** | **0.93-1.10** | 13.213 |

D9 dominates. At 1 keV, 4.4 ions per event are tested again, and they add 0.70 recombinations per event, close to the 0.72 fitted above. D7 alone moves the count by -0.03. An RMS z near 1 is what statistical noise alone gives.

**Fix** (the only production change): `PhysChemEngine::convert_history` (`crates/dna-chemistry/src/engine.rs`) now runs this schedule in passes; the module rustdoc gives the source lines. The docs/reproducibility.md draw order changed with it. **Gate rerun**, with the criteria unchanged (z <= 4.5, 15 %, 20 %): `cargo test -p dna-sim --test physchem_reference m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact --nocapture` **passed** (exit 0, 34 s). All 28 rows pass, and the largest |z| is 2.04 (100 keV H). At 1 keV the recombination fraction rises from 24.50 % to 25.86 %. Rust baselines that changed: in `crates/dna-chemistry/tests/history.rs`, `the_nearest_alive_electron_is_consumed_first` has a new script for the DoIt order. `competing_holes_demonstrate_the_event_order_assumption` became `competing_holes_follow_the_geant4_doit_order`, and its outcome is reversed. `an_auto_ionisation_electron_is_a_candidate_for_later_holes` became `..._is_not_a_candidate_in_the_pass_that_creates_it`. The new tests are `a_hole_whose_electron_was_taken_is_tested_again_in_the_next_pass` (D9) and `water_states_follow_the_geant4_configuration_order`. The three changed tests fail on the previous engine. The Rust means quoted above (RR-001 table, docs/04 D.2.1, ticket-01 output) describe the previous engine. No file in `validation/reference/` and no tolerance changed. Ticket 04 records the M6 status, Q21/R14 and the limits: one Geant4 seed, and the assumed tracking order.

### M6 residual audit (final record, 2026-10-17)

Session `m6-residual-rust-audit`, seed of `docs/gps-backlog-m6-residual.md`, started from the RR-001 failure (16 of 28 rows, all in the z test). It used only data that already existed: the committed `physchem_prechem.csv` (Geant4 11.4.1, seed 20261020) and Rust reruns. No Geant4 program was run, no file in `validation/reference/` and no tolerance changed. Commits: harness `b30beb5` (ticket 01), source comparison `bfe2a73` (ticket 02), fix `5df0aa1` (ticket 03).

**Outcome.** The stage is **electron-hole recombination scheduling** in `PhysChemEngine::convert_history`. It was proven from the Geant4 source and confirmed by a prototype scored on the RR-001 rows. The follow-up Geant4 request `m6-process-counts-geant4-request` (RR-002) is therefore not needed. After the fix the stage-matched gate `m6_prechem_counts_and_rms_match_geant4` **passes** (exit 0, 0 failing checks, was 16; the largest |z| of the 28 rows is 2.04, 100 keV H), with the criteria unchanged: count z <= 4.5, 15 % for non-small means, 20 % for the RMS radius.

**What this pass is and is not.** It is a pass on the stage-matched gate only: seven dissociation species, four energies, one Geant4 run, at global 1 ps before any IRT reaction. It does not by itself close M6, and nothing in this record unblocks M7 or M8 beyond what the gating rules already in this file say ("Consuming M6 species requires M6 closed", and "M6 closes only after their provenance/boundary checks and the unchanged numerical criteria pass" in the RR-001 paragraph above). Passing the criteria is a necessary condition in those rules; this file does not define the closure as automatic. **Lifting the M6-closed gate, and the M6 status, are decisions for the project lead.**

**1. Conservation relations (ticket 01, `crates/dna-sim/tests/physchem_audit.rs`).** Definitions and the 4.5 sigma bar were fixed in the test header before the results were read.

| Relation | Geant4 CSV (1, 10, 100, 900 keV) | Rust, 5 seeds x 4 energies | Can it name a family? |
|---|---|---|---|
| R-Q: charge of the alive species + 1 (thermalised primary e_aq) = 0 | integer residual 0 at all four energies | 0 on every seed | No: holds on both sides |
| R-H: H atoms - 2 x O atoms = 0 | integer residual 0 at all four energies | 0 on every seed | No: holds on both sides |
| R-Ch: every one of the 23 channels balances H, O and charge; run-level budget per event kind (excitation, ionisation, attachment, solvation) | n/a (the CSV has no event kinds) | exact on every kind, every energy | No: holds by construction of the table |
| R-W: O atoms per event (reported, not asserted) | 94.7355 / 927.903 / 9176.51 / 82481.2 | pre-fix 95.611 / 932.79 / 9227.0 / 82609.5, z +21.2 / +5.8 / +5.2 / +1.1 | Only a hint (+0.9 % O atoms at 1 keV) |

The relations hold exactly on both sides, so they cannot separate stage families: a missing or extra recombination moves species while keeping H, O and charge balanced, which is what the residual looked like. The Rust-side passes are close to identities of the channel table, so they are weak evidence on their own. **Geant4-side variance limitation:** the CSV has means, standard errors and second moments but no per-event covariance, so the Geant4 sigma of a combined quantity (R-W, any z of a relation) is the sum of the single-species variances as if uncorrelated. It is used as a bound on the order of the noise, not as an exact error. The integer-residual relations R-Q and R-H do not depend on it.

**2. Seed spread (ticket 01, pre-fix engine).** Five Rust master seeds (20261101-20261105), 20000/2000/200/20 histories per seed at 1/10/100/900 keV. z is (mean of 5 seeds - Geant4) over the combined error, from the harness output:

| Species | 1 keV | 10 keV | 100 keV | 900 keV | z after the fix: 1 / 10 / 100 / 900 keV |
|---|---:|---:|---:|---:|---|
| H3Op | +27.72 | +7.94 | +6.35 | +2.60 | +1.21 / -0.33 / +0.58 / -1.51 |
| °OH | +9.11 | +2.43 | +2.51 | -0.19 | +1.10 / +0.29 / +0.69 / -1.44 |
| OHm | -1.30 | -1.73 | -0.37 | -0.04 | -1.30 / -1.73 / -0.37 / -0.04 |
| e_aq | +26.65 | +8.18 | +6.36 | +2.62 | +1.56 / -0.10 / +0.62 / -1.52 |
| H | -16.12 | -8.94 | -6.19 | -8.18 | -0.05 / +1.16 / +1.59 / -1.29 |
| H2 | -8.13 | -6.01 | -5.67 | -4.40 | -0.74 / +0.06 / -0.71 / +0.37 |
| Oxy | -9.86 | -5.04 | -4.36 | -5.77 | -0.39 / +0.46 / +0.97 / -0.67 |

The seed-to-seed spread of the Rust means is small against the offset: at 1 keV the offset H3O+ 0.72, e_aq 0.73, H -0.46 molecules/event is about 40-80 times the seed spread (0.009, 0.010, 0.012). The pre-fix offsets keep their sign on all five seeds, so they are not a seed artefact. After the fix every one of these 28 z values is within |z| <= 1.73 (OHm, which the recombination does not touch, is unchanged by construction).

**3. Source comparison (ticket 02, Geant4 11.4.1 `.cc`; the file:line tables are in "M6 residual audit: source comparison" above).** Originally 36 items compared, 2 more not audited. The later N1 audit below closes the thermalisation-boundary item; N2 remains out of scope.

| Group | Items | Result |
|---|---|---|
| A. Physics event to water state (excitation level, ionisation shell, DA, vibration and elastic, birth time) | A1-A8 | all identical |
| B. Dissociation channels, probabilities, order, selection rule | B1-B10 | all identical |
| C. Displacements | C1-C8 | all identical (they enter only through recombination distances) |
| D. Electron-hole recombination | D1-D6, D8 identical | D7 candidate set at the first test: **differs**; D9 retest of a failed ion in the next zero-time step: **differs**; D10 processing order: undecidable in ticket 02, decided in ticket 03 from the track-holder source |
| N. Follow-up | N1 thermalisation boundary; N2 solvation count | N1 audited below: clip inapplicable to infinite water and operationally unreachable for this reference. N2 not audited, out of scope |

**4. Verdict and fix (ticket 03).** D9 (a failed recombination is retried at the next zero-time step with a new uniform, on a tree that now holds the auto-ionisation e_aq) dominates; D7 (auto-ionisation e_aq are not candidates in the pass that creates them) and D10 (Geant4 DoIt order: reverse main list, one sub-list per configuration ID in creation order) complete the schedule. The full trace is in "M6 residual audit: verdict and fix" above. The prototype re-converted identical Rust physics histories with each candidate schedule and scored the 28 RR-001 rows with the unchanged criteria, on the gate seed 20261017 and the five ticket-01 seeds:

| Schedule | Failing checks (6 seeds) | RMS of the 28 z (6 seeds) | Recombinations/event at 1 keV |
|---|---|---|---|
| Previous Rust (sequential, event order) | 11-16 | 7.06-7.89 | 12.547 |
| D7 only (pass structure, no retest) | 13-15 | 7.06-7.69 | 12.518 |
| D10 only (Geant4 order, sequential) | 12-16 | 7.26-7.77 | 12.555 |
| D7 + D9, event order | 0-2 | 1.60-1.99 | 13.084 |
| D7 + D9, reverse event order | 0 | 0.91-1.37 | 13.255 |
| **D7 + D9 + D10 (Geant4 order), implemented** | **0** | **0.93-1.10** | 13.213 |

An RMS z near 1 is what noise alone gives. The recombination fraction at 1 keV rose from 24.50 % to 25.86 % of the ionisations. The implemented change is the only production change (`crates/dna-chemistry/src/engine.rs`); three tests in `crates/dna-chemistry/tests/history.rs` were re-baselined and two added, and `docs/reproducibility.md` records the new draw order.

**Verification at the end of the audit** (2026-10-17, logs in `.scratch/tests/2026-10-17__m6-residual-rust-audit/`, files `04-*.log`). `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` all exit 0: **270 passed, 0 failed, 16 ignored**. The stage-matched ignored gate `m6_prechem_counts_and_rms_match_geant4` passes (exit 0, 0 failing checks; run by ticket 03 and by the supervising session). The *original* post-reaction gate `m6_initial_species_counts_and_rms_match_geant4` is an ignored test that fails by design (it compares different stages, Q25/R17): rerun now with `-- --ignored --exact`, it exits 101, and since commit `0f833fa` collects all failing rows it reaches `rms_radius_nm.unwrap()` (`physchem_reference.rs:186`) on the first reaction-only species with zero Rust population and panics there, instead of asserting at `1000 H2O2: z=111.56` as in the earlier text. The failure is the same stage mismatch; the test was not changed in this audit.

**5. Limits of this result.**
- **One Geant4 seed.** All Rust runs are compared with the single Geant4 run (seed 20261020). A Geant4 fluctuation of order 1-2 sigma on a row is not excluded, and the Geant4 seed-to-seed spread of this observable was not measured. A Rust-side pass at |z| <= 2.04 against one Geant4 run is consistent with agreement; it does not prove it.
- **No per-event covariance** in the reference CSV (see item 1): combined z values of correlated species use an uncorrelated bound. The gate itself tests species one at a time with the criteria fixed before the data existed.
- **Assumed tracking order.** D10 holds if the order of Rust cascade events (LIFO stack in `transport.rs`) corresponds to the order in which Geant4 creates the molecules of a sub-list. This is argued from the source, not traced at run time. The prototype cannot discriminate between the order variants: D7 + D9 in event order, reverse order and Geant4 order give 0-2, 0 and 0 failing checks with close recombination counts (13.084, 13.255, 13.213 per event at 1 keV). Evidence for D10 is therefore the source reading and not the data.
- **D9 was inferred** from the at-rest selection code, not traced in a running Geant4. The match of the recombination count (+0.70 per event from retests against 0.72 fitted from the species shifts) supports it.
- **N1 audited subsequently (2026-10-17).** The active Geant4 safety clip is inapplicable to infinite water and operationally unreachable in the committed reference workload; it cannot explain the 900 keV RMS excess. The source trace, scale argument and limits are below. N2 remains explicitly out of scope.
- **900 keV RMS radius still +12 to +13 % high** (seven species, +12.1 % to +13.5 %, 20 histories, gate seed). It is inside the 20 % criterion, it is not explained, and the seed-to-seed spread of a 20-history RMS was not measured here.
- The fix selected a schedule among six on the RR-001 rows, which were also the rows that failed before. The five audit seeds are new Rust streams but compare with the same Geant4 run, so they do not provide independent Geant4 evidence.
- The stage-matched reference stops before IRT. The RMS of species created by reactions, chemistry-stage G-values and anything after 1 ps are not touched by this result.

**Status of the open questions.** Q21/R14 are updated in docs/09. The ablation tables in docs/04 D.2.1 describe the pre-fix engine.

### N1 thermalisation boundary audit (2026-10-17)

**Verdict: N1 is audited.** Rust preserves the default Meesungnoen2002 displacement and its handoff
to the solvated electron. Geant4's geometry clip is active, but inapplicable to the Rust infinite-water
model and operationally unreachable in the committed M6 reference workload. It cannot explain the
900 keV RMS excess. **The excess remains unresolved; N2 is not audited and remains out of scope.**
Formal M6 closure remains the project lead's decision.

**Source trace.** The read-only Geant4 source is version 11.4.1
(`source/global/management/include/G4Version.hh:46,71-73`, version number 1141,
`geant4-11-04-patch-01`). Paths below use `D = source/processes/electromagnetic/dna`;
line numbers are the original files, including comments.

| Boundary | Geant4 11.4.1 evidence | Rust evidence |
|---|---|---|
| Kill and physical deposit | `D/models/include/G4DNAOneStepThermalizationModel.hpp:180-190`: for `k <= HighEnergyLimit()`, kill and deposit all `k`; chemistry activates the displacement draw | `crates/dna-physics/src/models/solvation.rs:107-143` returns the full deposit and sampled displacement; `crates/dna-physics/src/transport.rs:427-432,450` deposits it once and ends the track |
| Penetration law | `hpp:158-163` delegates to the model; `D/models/src/G4DNAOneStepThermalizationModel.cc:51-57,95-107,123-148` defines the polynomial, three independent Gaussian components with sigma `0.62665706865775006 * r_mean`, and the `r_mean == 0` isotropic `1e-3 nm` branch | `crates/dna-core/src/penetration.rs:10-43` has the same coefficients and polynomial; `solvation.rs:122-139` samples those two branches. Different Gaussian generators reproduce the distribution, not the Geant4 random stream |
| Active geometry correction | `hpp:193-203,225-239` first forms `position + displacement`, sets the navigator world, locates the incoming track, calls `ComputeStep(position, unit(displacement), displacementMag, safety)` and, if `safety <= displacementMag`, replaces the final position by `position + unit(displacement) * 0.80 * safety` | No geometry/navigator is supplied to `Solvation::sample`; infinite-water transport uses `position + displacement` directly |
| Product handoff | `hpp:241-244` passes the final position to `CreateSolvatedElectron` and proposes 25 meV on the killed track; `D/utils/src/G4DNAChemistryManager.cc:668-695` pushes one `e_aq` at that position at `picosecond + delayedTime` | `transport.rs:436-449` reports the thermalisation origin separately from the displaced `PhysChemEvent.position`; `crates/dna-chemistry/src/engine.rs:359-371` creates one `e_aq` at that event position, with no new displacement draw |

The old `CheckNextStep` alternative at `hpp:210-222` is commented out; the `ComputeStep` and
`safety <= displacementMag` block is **not**. Safety is isotropic distance to the nearest surface,
so this condition is broader than a directional segment-crossing test. For normal navigation the
mother safety starts from `DistanceToOut(localPoint)` and is returned as `newSafety`
(`source/geometry/navigation/src/G4NormalNavigation.cc:86-95,325-326`). For this full sphere,
`source/geometry/solids/CSG/src/G4Sphere.cc:2614-2616,2634-2642` gives `R - |position|`.

**Geometry and displacement scale.** `validation/geant4/g4rs_ref/g4rs_ref.cc:357-365` constructs
a single `G4_WATER` world sphere of radius **1 m**, placed at zero, with no daughters;
`:396-410` starts each primary at its centre along +z. Both committed M6 modes use this geometry.
The 900 keV physical track scale is millimetres (about 4 mm, as recorded in
`validation/geant4/README.md`), and `physchem_prechem.csv` gives an `e_aq` origin RMS of
`sqrt(sum_r2_nm2 / sum_count) = 1.810788 mm`. Those are scales, **not maximum-radius bounds**.
For comparison, the existing hand-polynomial checks (`low_energy.rs:438-464`) give mean
displacements 19.102655 nm at 7 eV and 18.860116 nm at 7.4 eV; the latter has component sigma
11.819 nm and displacement RMS 20.471 nm. One metre is about **53 million** of those mean
displacements. The low-energy zero-mean branch is only `1e-3 nm`.

A conservative analytic check avoids treating a typical displacement as a cutoff. On the whole
solvation window, the coefficient triangle inequality gives
`|r_mean(E)| <= sum(abs(gCoeff[12-i]) * 7.4^i) < 695306.507 nm`, hence
`sigma < 435719 nm` (an intentionally loose bound, much larger than the tested values).
Conditional on a thermalisation point remaining within the central 0.5 m, safety is at least
`s = 0.5 m`; the Gaussian union bound is `P(|d| >= s) <= 6 exp(-s^2 / (6 sigma^2))`.
Even with this loose sigma bound it is below `10^-95313` per draw, or below `10^-95304` for an
illustrative billion draws. The central-half-metre condition is far looser than the observed
millimetre track scale, but is not a measured maximum of the M6 histories. These facts support
**effective workload unreachability**, not mathematical impossibility: the Gaussian has unbounded
support, and the aggregate CSVs contain neither a clip counter nor maximum track radius, so they
cannot directly prove zero activations. No new physical cutoff is imposed. The conclusion does
not apply to primaries near surfaces, smaller volumes or finite-volume chemistry generally.

**Coverage and verification.** Existing tests retain their statistical criteria:
`low_energy::meesungnoen_r_mean_matches_the_hand_polynomial`,
`solvation_displacement_is_a_gaussian_with_sigma_proportional_to_the_mean_radius` and
`solvation_below_0_1_ev_displaces_by_1e_minus_3_nm_in_an_isotropic_direction` cover the law.
`crates/dna-physics/tests/transport.rs::energy_ledger_is_conserved_for_the_whole_cascade_at_five_energies`
checks `N_solv + N_att = N_tracks = 1 + N_sec` (`:133-138`).
`crates/dna-chemistry/tests/history.rs:35-50::solvation_creates_an_e_aq_at_the_event_position_without_random_numbers`
checks one electron at the event position and 1 ps, consuming zero uniforms.
`crates/dna-sim/tests/physchem_a.rs::species_conservation_and_untouched_ledger_on_real_cascades`
checks the cascade's deposits and `e_aq = N_solv + N_auto - N_recombined`.
The only missing invariant was the exact transport-to-event position handoff: the new deterministic
`low_energy::transport_solvation_event_keeps_the_sampled_displacement_and_local_deposit`
checks two valid off-origin primaries, including 7.4 eV, against the same scripted sampler draw,
with the deposit remaining at the thermalisation point and no extra draw or step. No statistical
test is duplicated. No production behavior, reference data, tolerance, generator or run request changes.

Verification on 2026-10-17: `cargo test -p dna-physics --test low_energy` exits 0 (21 passed);
the named `history` and `physchem_a` tests each pass with `-- --exact`.
`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and
`cargo test --workspace` all exit 0 (**271 passed, 0 failed, 16 ignored**), including the existing
cascade count identity. Logs `01-low-energy.log` through `06-workspace.log` and all build/test
artifacts are under `.scratch/tests/2026-10-17__m6-n1-thermalisation-audit/`.

### N2 lifecycle audit and spatial diagnostic contract (2026-10-19)

**N2 source audit complete; the 900 keV spatial residual remains unresolved.**
The direct Geant4 11.4.1 source trace in docs/02 B.7.9 finds one alive e_aq per
terminal solvation, at the displaced position and global 1 ps, with the physical
electron killed and its full kinetic energy deposited. Rust preserves those
semantics. Water states enter at their physical interaction positions; initial
molecules enter delayed lists, while dissociation daughters enter secondary
lists and join after the zero-time pass. Recombination kills an electron before
survivor scoring. The source proves these local operations and the reverse
main-list processing rule; it does **not** prove a runtime mapping from Rust's
LIFO cascade event order to Geant4's physical-event/molecule-insertion order.

Geant4 retains physical and molecular parent/track IDs; current Rust records
retain history IDs and internal event/vector indices only. Diagnostic IDs and
creation/processing ordinals will expose each implementation's lifecycle, without
claiming eventwise pairing between different RNG engines. Geant4's molecular
`BuildTrack` consumes direction uniforms during creation; Rust's conversion of
an already displaced solvation event consumes no RNG. This is a stream/record
representation difference, not source evidence for a spatial defect.

This verdict supersedes the earlier historical statements that N2 was outside
the audit. Q21/R14 retain the untraced ordering assumption. No physical model,
chemistry schedule, reference data, generator mode or M6 criterion is changed by
this audit; the stage-matched gate's recorded pass and project-lead closure
decision remain as before.

The preregistered investigation in docs/09 observes chemistry-producing physical
sites, solvated-electron creation, pre-decay water states, dissociation products
at birth, and final post-recombination/pre-IRT survivors. It compares independent
900 keV samples with five seeds x 40 histories, 20,000 hierarchical bootstrap
replicates, and a precision-only uniform expansion to 80 then 160 histories per
seed (at most 800 histories per implementation). The sole primary estimand is
the ratio of pooled origin RMS across all seven shared final survivor species;
simultaneous boundary intervals identify the **first detected divergence**, not
its cause. Diagnostic implementation and the new user-run Geant4 request are
subsequent tickets; no new run or spatial inference is claimed here.

**RR-002 conclusion: the 900 keV pooled RMS residual was not reproduced at the required precision
(2026-10-23).** The preregistered five-boundary diagnostic was run independently in Geant4 11.4.1 and
Rust with seeds 20261021--20261025. The initial 40-histories-per-seed level did not meet the precision
stopping rule, so both implementations were rerun with new, complete 80-histories-per-seed artifacts:
400 histories per implementation and 800 histories across the comparison. The scientific model,
simulation seeds, analysis seed 20261026, 20,000 hierarchical bootstrap replicates, primary estimand
and acceptance rule were unchanged.

Both runs exited 0 with exact five-seed coverage and complete-output markers. The primary
Rust/Geant4 pooled seven-species final-survivor RMS ratio was **0.981740457**, with a 95% interval of
**[0.949564151, 1.015007983]**. Its relative interval half-width was **3.333%**, satisfying the
preregistered <= 4% precision target; the 160-histories-per-seed expansion was therefore not run.
Because the interval includes 1.0, RR-002 found no statistically resolved primary RMS difference.
This is not an equivalence proof: the frozen contract defines interval exclusion as detection and
interval inclusion as inconclusive. It does show that RR-001's common +12.1% to +13.5% Rust excess
was not reproduced in the larger independent sample and provides no evidence for changing the
scientific model, M6 tolerances or reference data.

The 80-history CSVs and matching provenance were intentionally retained as local-only artifacts under
`.scratch/rr-002-80/`; they were not promoted to `validation/reference/` and are not durable committed
reference evidence. RR-002 records diagnostic evidence only and neither reopens nor closes M6. The N2
source audit remains complete, while Q21/R14's broader runtime-order correspondence remains a separate
documented limitation.

## M7 Chemistry: reaction network, diffusion, IRT
Scope: species/reaction TOML loaders (option3 table, script-checked against C++), free unbounded diffusion and an isolated two-body option3 IRT kernel with TDC and neutral PDC. The many-species scheduler, SBS oracle, ODE test harness, reaction products/site placement and finite boundaries are deferred. M7 is not closed; G5/G7 closure criteria remain outstanding.
Tests: ticket 07 delivers V9 for three isolated regimes and tests explicit edge/outcome policies. The V9 acceptance here covers its pre-registered analytic curves only; IRT vs SBS, V10 ODE and Geant4 comparison have not been delivered.
Seeds (`docs/gps-backlog.md`): `m7-option3-reaction-table`, `m7-diffusion-and-two-body-irt`, `m7-sbs-oracle-and-ode`, `m7-sbs-close-gate`, `m7-multispecies-irt-scheduler`, `m7-scheduler-close-gate`. Consuming M6 species requires M6 closed.

**Reaction-table seed implemented.** `reactions_option3.toml` contains all 72 executable Geant4 11.4.1 option3 entries with stable source-order IDs. Machine extraction, an independent hand evaluator, strict Rust validation and an all-field cross-check cover participants, products, source class/line, observed-rate order/value, kinetic intent and `SetReactionType`. The four bulk configurations remain outside `Species`; exact reaction 53 is the only source anomaly allowlisted. At delivery this seed supplied validated data only; the isolated diffusion and pair-kinetics work is recorded in the following ticket-07 entry. Scheduling and the M6-consumption gate remain for later seeds.

**Isolated diffusion/two-body IRT seed delivered (ticket 07, 2026-10-23).** Added zero-draw identity-preserving free diffusion for zero time/diffusivity; stable molecule identity/self-pair rejection; validated TDC/PDC parameter derivation; and `Immediate`/`After(elapsed)`/`NoReaction` sampling. Neutral PDC rejection exhaustion, invalid numeric domains and unsupported charged PDC use typed errors. V9 used N=100,000 pairs in each of three registered regimes, six time points each, 18 CDF/survival comparisons and three no-reaction masses; the independently calculated 80-digit oracle and absolute 3-sigma limits were fixed in the test header before the first statistical run. The test passed. A separate fixed workload of 1,024 histories compared exact snapshots and serialized bytes at 1, 2 and 8 Rayon threads and passed. These are isolated-kernel validation results, not evidence for a scheduler or end-to-end M6 chemistry. Remaining risks include the charged effective-distance law being a Geant4 prescription rather than a general electrostatic solution, and no cross-implementation Geant4 run. Deferred: many-species scheduling, SBS, ODE, reaction products/sites, finite boundaries, and Geant4 comparison.

## M8 G(t) end-to-end
Scope: orchestration, scoring G(t) with uncertainties, TOML config, CLI `dna run config.toml`, JSON output with metadata.
Tests: V11, V13. Acceptance: V11 for e- 1 MeV (or 100 keV and 1 MeV) and p 10 MeV against chem6 reference.
Seeds (`docs/gps-backlog.md`): `m8-gt-scoring-and-chem6-reference`, `m8-gt-close-gate`, `m8-config-and-cli`, `m8-reproducibility-v13`. **Deviation:** the proton 10 MeV acceptance is deferred until M5 exists; the seeds cover e- 100 keV and 1 MeV.

## M9 Performance
Profile, benchmark, optimise within scientific equivalence; scaling study threads 1..N; document.

## M10 Documentation & extensibility hardening
Guides: adding particle / process / reaction; example configs; mapping table C++ -> Rust.

## M11 Extensions (separate plans)
1-10 MeV electrons (Q1, deferred by decision), finite boxes/boundaries (UHDR), scavengers/oxygen, IRT-syn and parallel chemistry (E.4), additional particles.
