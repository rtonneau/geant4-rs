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

## M2 Electron transport, option2, E <= 1 MeV, primaries only (no secondary tracking)
Scope: Champion elastic, Born excitation, Born1 ionisation (energy loss, secondary *recorded* not transported), infinite water, energy ledger, `mfp` and `range`-like scorers.
Refs: B.2 files + audit G1, G8.
Tests: V2-V5, V7. Acceptance: V2/V3 pass; V4 for elastic angle, ionisation secondary energy; mfp agrees V5.
Perf: no optimisation; record baseline. Risk: sampling code semantics hidden in `SampleSecondaries`.

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
